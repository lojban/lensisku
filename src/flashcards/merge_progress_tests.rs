//! Pins the current behaviour of `merge_anonymous_progress` before anyone batches its loop.
//!
//! Needs a disposable database with the full lensisku schema, e.g.
//! a `pg_dump -s` of the dev DB restored into `flashcards_test` (then
//! `REFRESH MATERIALIZED VIEW user_reaction_counts`), in `FLASHCARDS_TEST_DATABASE_URL`.
#![allow(clippy::unwrap_used)]

use super::dto::{MergeProgressLevelItem, MergeProgressRequest};
use super::service::merge_anonymous_progress;

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn merge_anonymous_progress_current_behaviour() {
    let url = crate::test_db_guard::disposable_database_url("FLASHCARDS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(2)
        .build()
        .unwrap();
    let client = pool.get().await.unwrap();

    let suffix = std::process::id();
    let user_id: i32 = client
        .query_one(
            "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'x', $2, now(), 'user') RETURNING userid",
            &[
                &format!("merge_t_{suffix}"),
                &format!("merge_t_{suffix}@example.invalid"),
            ],
        )
        .await
        .unwrap()
        .get(0);
    let collection = |name: &'static str| {
        let client = &client;
        async move {
            client
        .query_one(
          "INSERT INTO collections (user_id, name, is_public) VALUES ($1, $2, true) RETURNING collection_id",
          &[&user_id, &name],
        )
        .await
        .unwrap()
        .get::<_, i32>(0)
        }
    };
    let own = collection("merge own").await;
    let other = collection("merge other").await;
    let level = |collection_id: i32, position: i32| {
        let client = &client;
        async move {
            client
        .query_one(
          "INSERT INTO flashcard_levels (collection_id, name, min_cards, min_success_rate, position)
           VALUES ($1, 'l', 2, 0.5, $2) RETURNING level_id",
          &[&collection_id, &position],
        )
        .await
        .unwrap()
        .get::<_, i32>(0)
        }
    };
    let a = level(own, 0).await;
    let b = level(own, 1).await;
    let foreign = level(other, 0).await;
    client
        .execute(
            "INSERT INTO level_prerequisites (level_id, prerequisite_id) VALUES ($1, $2)",
            &[&b, &a],
        )
        .await
        .unwrap();

    let item = |level_id, cards_completed, correct_answers, total_answers| MergeProgressLevelItem {
        level_id,
        cards_completed,
        correct_answers,
        total_answers,
    };

    // First merge: A meets its thresholds, B does not; the foreign level is skipped.
    // A repeated id is processed twice, so it counts twice and keeps the maxima.
    let request = MergeProgressRequest {
        collection_id: own,
        level_progress: vec![
            item(a, 3, 2, 3),
            item(b, 1, 0, 1),
            item(foreign, 9, 9, 9),
            item(a, 1, 1, 1),
        ],
    };
    let merged = merge_anonymous_progress(&pool, user_id, &request)
        .await
        .unwrap();
    assert_eq!(merged.merged_levels, 3);

    let row = |level_id: i32| {
        let client = &client;
        async move {
            client
                .query_opt(
                    "SELECT cards_completed, correct_answers, total_answers,
                  completed_at IS NOT NULL, unlocked_at IS NOT NULL
           FROM user_level_progress WHERE user_id = $1 AND level_id = $2",
                    &[&user_id, &level_id],
                )
                .await
                .unwrap()
                .map(|r| {
                    (
                        r.get::<_, i32>(0),
                        r.get::<_, i32>(1),
                        r.get::<_, i32>(2),
                        r.get::<_, bool>(3),
                        r.get::<_, bool>(4),
                    )
                })
        }
    };
    assert_eq!(row(a).await, Some((3, 2, 3, true, true)));
    assert_eq!(row(b).await, Some((1, 0, 1, false, true)));
    assert_eq!(row(foreign).await, None);

    // Second merge raises B over its thresholds; completion is judged on the merged values,
    // so B becomes complete.
    let request = MergeProgressRequest {
        collection_id: own,
        level_progress: vec![item(b, 5, 5, 5)],
    };
    merge_anonymous_progress(&pool, user_id, &request)
        .await
        .unwrap();
    assert_eq!(row(b).await, Some((5, 5, 5, true, true)));

    // An unknown level id fails the whole request and rolls back.
    let request = MergeProgressRequest {
        collection_id: own,
        level_progress: vec![item(a, 7, 7, 7), item(i32::MAX, 1, 1, 1)],
    };
    assert!(merge_anonymous_progress(&pool, user_id, &request)
        .await
        .is_err());
    assert_eq!(row(a).await, Some((3, 2, 3, true, true)));

    client
        .execute(
            "DELETE FROM user_level_progress WHERE user_id = $1",
            &[&user_id],
        )
        .await
        .unwrap();
    client
        .execute(
            "DELETE FROM collections WHERE collection_id = ANY($1)",
            &[&vec![own, other]],
        )
        .await
        .unwrap();
    client
        .execute("DELETE FROM user_balances WHERE user_id = $1", &[&user_id])
        .await
        .unwrap();
    client
        .execute("DELETE FROM users WHERE userid = $1", &[&user_id])
        .await
        .unwrap();
}
