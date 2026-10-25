//! Course settings, batched item-levels, and the collection `comment_count` regression
//! (badge must count only the collection's own discussion thread, like the thread list).
//!
//! Needs `FLASHCARDS_TEST_DATABASE_URL` (disposable DB with the lensisku schema).
#![allow(clippy::unwrap_used)]

use super::dto::UpdateCourseSettingsRequest;
use super::service::{get_collection_item_levels, get_course_settings, update_course_settings};

async fn pool() -> deadpool_postgres::Pool {
    let url = crate::test_db_guard::disposable_database_url("FLASHCARDS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    deadpool_postgres::Pool::builder(manager)
        .max_size(4)
        .build()
        .unwrap()
}

async fn seed(client: &deadpool_postgres::Client, tag: &str) -> (i32, i32) {
    let suffix = format!("{tag}_{}", std::process::id());
    let user_id: i32 = client
        .query_one(
            "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'x', $2, now(), 'user') RETURNING userid",
            &[&suffix, &format!("{suffix}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);
    let collection_id: i32 = client
        .query_one(
            "INSERT INTO collections (user_id, name, is_public) VALUES ($1, 'c', true) RETURNING collection_id",
            &[&user_id],
        )
        .await
        .unwrap()
        .get(0);
    (user_id, collection_id)
}

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn course_settings_roundtrip_and_owner_only() {
    let pool = pool().await;
    let client = pool.get().await.unwrap();
    let (owner, cid) = seed(&client, "cs_o").await;
    let (other, _) = seed(&client, "cs_x").await;
    assert!(
        !get_course_settings(&pool, cid, None)
            .await
            .unwrap()
            .auto_challenge_mix
    );
    let req = UpdateCourseSettingsRequest {
        auto_challenge_mix: Some(true),
    };
    assert!(update_course_settings(&pool, other, cid, &req)
        .await
        .is_err());
    update_course_settings(&pool, owner, cid, &req)
        .await
        .unwrap();
    assert!(
        get_course_settings(&pool, cid, None)
            .await
            .unwrap()
            .auto_challenge_mix
    );
    let none = UpdateCourseSettingsRequest {
        auto_challenge_mix: None,
    };
    update_course_settings(&pool, owner, cid, &none)
        .await
        .unwrap();
    assert!(
        get_course_settings(&pool, cid, Some(owner))
            .await
            .unwrap()
            .auto_challenge_mix
    );
}

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn item_levels_batched() {
    let pool = pool().await;
    let client = pool.get().await.unwrap();
    let (_owner, cid) = seed(&client, "il").await;
    let item_id: i32 = client
        .query_one(
            "INSERT INTO collection_items (collection_id, free_content_front, free_content_back, position)
             VALUES ($1, 'coi', 'hi', 0) RETURNING item_id",
            &[&cid],
        )
        .await
        .unwrap()
        .get(0);
    let fid: i32 = client
        .query_one(
            "INSERT INTO flashcards (collection_id, item_id, position, direction) VALUES ($1, $2, 0, 'both') RETURNING id",
            &[&cid, &item_id],
        )
        .await
        .unwrap()
        .get(0);
    let level_id: i32 = client
        .query_one(
            "INSERT INTO flashcard_levels (collection_id, name, min_cards, min_success_rate, position)
             VALUES ($1, 'L1', 1, 0.5, 0) RETURNING level_id",
            &[&cid],
        )
        .await
        .unwrap()
        .get(0);
    client
        .execute(
            "INSERT INTO flashcard_level_items (level_id, flashcard_id, position) VALUES ($1, $2, 0)",
            &[&level_id, &fid],
        )
        .await
        .unwrap();
    let rows = get_collection_item_levels(&pool, cid, None).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].item_id, item_id);
    assert_eq!(rows[0].level_id, level_id);
    assert_eq!(rows[0].level_name, "L1");
}

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn comment_count_matches_collection_thread_only() {
    let pool = pool().await;
    let client = pool.get().await.unwrap();
    let (uid, cid) = seed(&client, "cc").await;
    // Flashcard thread (collection + definition link): must NOT count toward the badge.
    let t_card: i32 = client
        .query_one(
            "INSERT INTO threads (collection_id, definition_link_id) VALUES ($1, 999999) RETURNING threadid",
            &[&cid],
        )
        .await
        .unwrap()
        .get(0);
    // Collection's own discussion thread: counts.
    let t_coll: i32 = client
        .query_one(
            "INSERT INTO threads (collection_id) VALUES ($1) RETURNING threadid",
            &[&cid],
        )
        .await
        .unwrap()
        .get(0);
    for (t, n) in [(t_card, 1), (t_card, 2), (t_coll, 1)] {
        client
            .execute(
                "INSERT INTO comments (threadid, userid, commentnum, \"time\", subject) VALUES ($1, $2, $3, 0, 's')",
                &[&t, &uid, &n],
            )
            .await
            .unwrap();
    }
    let c = crate::collections::service::get_collection(&pool, cid, Some(uid))
        .await
        .unwrap();
    assert_eq!(c.comment_count, 1);
}
