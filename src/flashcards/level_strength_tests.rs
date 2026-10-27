//! `level_strengths`: FSRS-style recall average per level, due count, watering flag; information
//! cards and never-reviewed cards are ignored.
//!
//! Needs `FLASHCARDS_TEST_DATABASE_URL` (disposable DB with the lensisku schema).
#![allow(clippy::unwrap_used)]

use super::service::{level_strengths, LevelStrength};

/// (level, direction, Some((days since review, stability, next review in days))).
type Card<'a> = (i32, &'a str, Option<(f64, f64, f64)>);

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn level_strength_formula_due_and_watering() {
    let url = crate::test_db_guard::disposable_database_url("FLASHCARDS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(2)
        .build()
        .unwrap();
    let mut client = pool.get().await.unwrap();
    let tx = client.transaction().await.unwrap();
    let suffix = format!("{}_{}", std::process::id(), rand::random::<u32>());
    let user_id: i32 = tx
        .query_one(
            "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'x', $2, now(), 'user') RETURNING userid",
            &[&format!("lst_{suffix}"), &format!("lst_{suffix}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);
    let collection_id: i32 = tx
        .query_one(
            "INSERT INTO collections (user_id, name, is_public) VALUES ($1, 'lst', false) RETURNING collection_id",
            &[&user_id],
        )
        .await
        .unwrap()
        .get(0);
    let level = |name: &'static str, pos: i32| {
        let tx = &tx;
        async move {
            tx.query_one(
                "INSERT INTO flashcard_levels (collection_id, name, position) VALUES ($1, $2, $3) RETURNING level_id",
                &[&collection_id, &name, &pos],
            )
            .await
            .unwrap()
            .get::<_, i32>(0)
        }
    };
    let fresh = level("fresh", 0).await;
    let wilted = level("wilted", 1).await;
    let untouched = level("untouched", 2).await;

    // (level, direction, Some((days since review, stability, next_review in days)) or None)
    let cards: [Card; 7] = [
        (fresh, "direct", Some((0.0, 10.0, 10.0))),
        (fresh, "direct", Some((10.0, 10.0, 5.0))), // R = 0.9 at t = S
        (fresh, "just_information", Some((365.0, 1.0, -300.0))), // ignored
        (wilted, "direct", Some((36.0, 4.0, -30.0))), // R = 0.5, due
        (wilted, "dialogue", Some((36.0, 4.0, -30.0))), // dialogues count: R = 0.5, due
        (wilted, "direct", None),                   // never reviewed: ignored
        (untouched, "direct", None),
    ];
    for (pos, (lvl, dir, prog)) in cards.into_iter().enumerate() {
        let pos = pos as i32;
        let item_id: i32 = tx
            .query_one(
                "INSERT INTO collection_items (collection_id, free_content_front, free_content_back, position)
                 VALUES ($1, 'f', 'b', $2) RETURNING item_id",
                &[&collection_id, &pos],
            )
            .await
            .unwrap()
            .get(0);
        let fid: i32 = tx
            .query_one(
                "INSERT INTO flashcards (collection_id, item_id, position, direction)
                 VALUES ($1, $2, $3, $4::text::flashcard_direction) RETURNING id",
                &[&collection_id, &item_id, &pos, &dir],
            )
            .await
            .unwrap()
            .get(0);
        tx.execute(
            "INSERT INTO flashcard_level_items (level_id, flashcard_id, position) VALUES ($1, $2, $3)",
            &[&lvl, &fid, &pos],
        )
        .await
        .unwrap();
        if let Some((ago, stability, next)) = prog {
            tx.execute(
                "INSERT INTO user_flashcard_progress
                   (user_id, flashcard_id, stability, interval, review_count, status,
                    last_reviewed_at, next_review_at)
                 VALUES ($1, $2, $3, 1, 1, 'review',
                         now() - make_interval(secs => $4::float8 * 86400),
                         now() + make_interval(secs => $5::float8 * 86400))",
                &[&user_id, &fid, &stability, &ago, &next],
            )
            .await
            .unwrap();
        }
    }

    let s = level_strengths(&tx, collection_id, user_id).await.unwrap();
    // avg(1.0, 0.9) = 0.95
    assert_eq!(
        s.get(&fresh).copied(),
        Some(LevelStrength {
            strength: Some(95),
            due_count: 0,
            needs_watering: false
        })
    );
    // R = 1 / (1 + 36 / 36) = 0.5, the direct card and the dialogue are due
    assert_eq!(
        s.get(&wilted).copied(),
        Some(LevelStrength {
            strength: Some(50),
            due_count: 2,
            needs_watering: true
        })
    );
    assert!(!s.contains_key(&untouched));
    tx.rollback().await.unwrap();
}
