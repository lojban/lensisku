//! Level completion must be judged on the counts *after* the review that crossed the
//! threshold (V201). Previously the trigger read the pre-update row, so the last needed
//! review never completed the level and, with nothing left due, the next level stayed locked.
//!
//! Needs `FLASHCARDS_TEST_DATABASE_URL` (disposable DB with the lensisku schema + V201).
#![allow(clippy::unwrap_used)]

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn crossing_review_completes_level_and_unlocks_next() {
    let url = crate::test_db_guard::disposable_database_url("FLASHCARDS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(1)
        .build()
        .unwrap();
    let c = pool.get().await.unwrap();
    let sfx = format!(
        "{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_micros()
    );
    let uid: i32 = c
        .query_one(
            "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'x', $2, now(), 'guest') RETURNING userid",
            &[&format!("lc_{sfx}"), &format!("lc_{sfx}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);
    let cid: i32 = c
        .query_one(
            "INSERT INTO collections (user_id, name, is_public) VALUES ($1, 'lc', true) RETURNING collection_id",
            &[&uid],
        )
        .await
        .unwrap()
        .get(0);
    let level = |name: &'static str, pos: i32| {
        let c = &c;
        async move {
            c.query_one(
                "INSERT INTO flashcard_levels (collection_id, name, min_cards, min_success_rate, position)
                 VALUES ($1, $2, 2, 0.5, $3) RETURNING level_id",
                &[&cid, &name, &pos],
            )
            .await
            .unwrap()
            .get::<_, i32>(0)
        }
    };
    let l1 = level("L1", 0).await;
    let l2 = level("L2", 1).await;
    c.execute(
        "INSERT INTO level_prerequisites (level_id, prerequisite_id) VALUES ($1, $2)",
        &[&l2, &l1],
    )
    .await
    .unwrap();
    let mut cards = Vec::new();
    for pos in 0..2 {
        let item: i32 = c
            .query_one(
                "INSERT INTO collection_items (collection_id, free_content_front, free_content_back, position)
                 VALUES ($1, 'coi', 'hi', $2) RETURNING item_id",
                &[&cid, &pos],
            )
            .await
            .unwrap()
            .get(0);
        let fid: i32 = c
            .query_one(
                "INSERT INTO flashcards (collection_id, item_id, position, direction) VALUES ($1, $2, $3, 'direct') RETURNING id",
                &[&cid, &item, &pos],
            )
            .await
            .unwrap()
            .get(0);
        c.execute(
            "INSERT INTO flashcard_level_items (level_id, flashcard_id, position) VALUES ($1, $2, $3)",
            &[&l1, &fid, &pos],
        )
        .await
        .unwrap();
        cards.push(fid);
    }
    let review = |fid: i32| {
        let c = &c;
        async move {
            c.execute(
                "INSERT INTO user_flashcard_progress (user_id, flashcard_id, card_side) VALUES ($1, $2, 'direct')
                 ON CONFLICT DO NOTHING",
                &[&uid, &fid],
            )
            .await
            .unwrap();
            c.execute(
                "INSERT INTO flashcard_review_history (user_id, flashcard_id, rating, elapsed_days, scheduled_days, state)
                 VALUES ($1, $2, 3, 0, 0, '{}'::jsonb)",
                &[&uid, &fid],
            )
            .await
            .unwrap();
        }
    };
    let completed = || {
        let c = &c;
        async move {
            c.query_one(
                "SELECT completed_at IS NOT NULL FROM user_level_progress WHERE user_id = $1 AND level_id = $2",
                &[&uid, &l1],
            )
            .await
            .unwrap()
            .get::<_, bool>(0)
        }
    };
    review(cards[0]).await;
    assert!(!completed().await, "one card is below min_cards = 2");
    review(cards[1]).await;
    assert!(
        completed().await,
        "the review reaching min_cards must complete the level"
    );
    let unlocked: bool = c
        .query_one("SELECT check_level_prerequisites($1, $2)", &[&uid, &l2])
        .await
        .unwrap()
        .get(0);
    assert!(unlocked, "next level unlocks once level 1 is complete");

    c.execute("DELETE FROM collections WHERE collection_id = $1", &[&cid])
        .await
        .ok();
}
