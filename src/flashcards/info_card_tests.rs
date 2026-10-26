//! Information cards (`just_information`) are sticky and shown once: no challenge mix or
//! Auto conversion may retype them, and an acknowledged one is never due again.
//!
//! Needs `FLASHCARDS_TEST_DATABASE_URL` (disposable DB with the lensisku schema).
#![allow(clippy::unwrap_used)]

use super::dto::ReviewRequest;
use super::service::{apply_challenge_mix_to_level, convert_level_to_auto, review_flashcard};

struct Fixture {
    pool: deadpool_postgres::Pool,
    user_id: i32,
    level_id: i32,
    info_id: i32,
}

async fn fixture(tag: &str) -> Fixture {
    let url = crate::test_db_guard::disposable_database_url("FLASHCARDS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(4)
        .build()
        .unwrap();
    let client = pool.get().await.unwrap();
    let name = format!("info_{tag}_{}", std::process::id());
    let user_id: i32 = client
        .query_one(
            "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'x', $2, now(), 'user') RETURNING userid",
            &[&name, &format!("{name}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);
    let collection_id: i32 = client
        .query_one(
            "INSERT INTO collections (user_id, name, is_public) VALUES ($1, 'info', false) RETURNING collection_id",
            &[&user_id],
        )
        .await
        .unwrap()
        .get(0);
    let level_id: i32 = client
        .query_one(
            "INSERT INTO flashcard_levels (collection_id, name, min_cards, min_success_rate, position)
             VALUES ($1, 'l', 1, 0.5, 0) RETURNING level_id",
            &[&collection_id],
        )
        .await
        .unwrap()
        .get(0);
    let mut info_id = 0;
    for (pos, (front, back, dir)) in [
        ("About this level", "Greetings.", "just_information"),
        ("coi", "hi", "both"),
        ("co'o", "bye", "both"),
    ]
    .into_iter()
    .enumerate()
    {
        let pos = pos as i32;
        let item_id: i32 = client
            .query_one(
                "INSERT INTO collection_items (collection_id, free_content_front, free_content_back, position)
                 VALUES ($1, $2, $3, $4) RETURNING item_id",
                &[&collection_id, &front, &back, &pos],
            )
            .await
            .unwrap()
            .get(0);
        let fid: i32 = client
            .query_one(
                &format!(
                    "INSERT INTO flashcards (collection_id, item_id, position, direction)
                     VALUES ($1, $2, $3, '{dir}') RETURNING id"
                ),
                &[&collection_id, &item_id, &pos],
            )
            .await
            .unwrap()
            .get(0);
        client
            .execute(
                "INSERT INTO user_flashcard_progress (user_id, flashcard_id, card_side, status, next_review_at)
                 VALUES ($1, $2, 'direct', 'new', now())",
                &[&user_id, &fid],
            )
            .await
            .unwrap();
        client
            .execute(
                "INSERT INTO flashcard_level_items (level_id, flashcard_id, position) VALUES ($1, $2, $3)",
                &[&level_id, &fid, &pos],
            )
            .await
            .unwrap();
        if pos == 0 {
            info_id = fid;
        }
    }
    Fixture {
        pool,
        user_id,
        level_id,
        info_id,
    }
}

async fn direction(f: &Fixture) -> String {
    f.pool
        .get()
        .await
        .unwrap()
        .query_one(
            "SELECT direction::text FROM flashcards WHERE id = $1",
            &[&f.info_id],
        )
        .await
        .unwrap()
        .get(0)
}

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn challenge_mix_skips_info_cards() {
    let f = fixture("mix").await;
    apply_challenge_mix_to_level(&f.pool, f.user_id, f.level_id)
        .await
        .unwrap();
    assert_eq!(direction(&f).await, "just_information");
}

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn convert_to_auto_skips_info_cards() {
    let f = fixture("auto").await;
    let r = convert_level_to_auto(&f.pool, f.user_id, f.level_id)
        .await
        .unwrap();
    assert_eq!(r.updated_count, 2);
    assert_eq!(direction(&f).await, "just_information");
}

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn acknowledged_info_card_is_never_due_again() {
    let f = fixture("ok").await;
    let req = ReviewRequest {
        flashcard_id: f.info_id,
        rating: 4,
        card_side: "direct".into(),
        hint_used: false,
        presented_type: None,
    };
    review_flashcard(&f.pool, f.user_id, &req).await.unwrap();
    let row = f
        .pool
        .get()
        .await
        .unwrap()
        .query_one(
            "SELECT status::text, next_review_at > now() + INTERVAL '50 years'
             FROM user_flashcard_progress WHERE user_id = $1 AND flashcard_id = $2 AND NOT archived",
            &[&f.user_id, &f.info_id],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, String>(0), "graduated");
    assert!(row.get::<_, bool>(1));
}
