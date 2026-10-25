//! `apply_challenge_mix_to_level` must leave the owner with active progress on every side the
//! new directions use (the progress table is unique per side over archived rows too).
//!
//! Needs `FLASHCARDS_TEST_DATABASE_URL` (disposable DB with the lensisku schema).
#![allow(clippy::unwrap_used)]

use super::service::apply_challenge_mix_to_level;

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn apply_mix_keeps_owner_progress_active() {
    let url = crate::test_db_guard::disposable_database_url("FLASHCARDS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(4)
        .build()
        .unwrap();
    let client = pool.get().await.unwrap();
    let suffix = std::process::id();
    let user_id: i32 = client
        .query_one(
            "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'x', $2, now(), 'user') RETURNING userid",
            &[&format!("mix_t_{suffix}"), &format!("mix_t_{suffix}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);
    let collection_id: i32 = client
        .query_one(
            "INSERT INTO collections (user_id, name, is_public) VALUES ($1, 'mix', false) RETURNING collection_id",
            &[&user_id],
        )
        .await
        .unwrap()
        .get(0);
    let level_id: i32 = client
        .query_one(
            "INSERT INTO flashcard_levels (collection_id, name, min_cards, min_success_rate, position)
             VALUES ($1, 'l', 2, 0.5, 0) RETURNING level_id",
            &[&collection_id],
        )
        .await
        .unwrap()
        .get(0);
    let mut cards = Vec::new();
    for (pos, (front, back)) in [("coi", "hi"), ("coi do", "hi you"), ("co'o", "bye")]
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
                "INSERT INTO flashcards (collection_id, item_id, position, direction)
                 VALUES ($1, $2, $3, 'both') RETURNING id",
                &[&collection_id, &item_id, &pos],
            )
            .await
            .unwrap()
            .get(0);
        client
            .execute(
                "INSERT INTO user_flashcard_progress (user_id, flashcard_id, card_side, status, next_review_at)
                 VALUES ($1, $2, 'direct', 'new', now()), ($1, $2, 'reverse', 'new', now())",
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
        cards.push(fid);
    }

    let active = |fid: i32| {
        let client = &client;
        async move {
            client
                .query_one(
                    "SELECT COALESCE(string_agg(card_side, ',' ORDER BY card_side), '')
                     FROM user_flashcard_progress WHERE user_id = $1 AND flashcard_id = $2 AND NOT archived",
                    &[&user_id, &fid],
                )
                .await
                .unwrap()
                .get::<_, String>(0)
        }
    };

    let res = apply_challenge_mix_to_level(&pool, user_id, level_id)
        .await
        .unwrap();
    assert_eq!(res.updated_count, 2); // card 2 stays `both` (no audio)
    assert_eq!(active(cards[0]).await, "direct"); // quiz_direct
    assert_eq!(active(cards[1]).await, "reverse"); // wordbank_reverse
    assert_eq!(active(cards[2]).await, "direct,reverse");

    // Switching back restores the archived side instead of leaving it without progress.
    client
        .execute(
            "UPDATE flashcards SET direction = 'both' WHERE id = $1",
            &[&cards[0]],
        )
        .await
        .unwrap();
    client
        .execute(
            "UPDATE flashcards SET direction = 'listen_wordbank_both' WHERE id = $1",
            &[&cards[1]],
        )
        .await
        .unwrap();
    apply_challenge_mix_to_level(&pool, user_id, level_id)
        .await
        .unwrap();
    assert_eq!(active(cards[0]).await, "direct");
    assert_eq!(active(cards[1]).await, "reverse");

    // The due list carries quiz options, and a quiz card that cannot build them (no stored
    // answer) does not take the options of the other cards with it.
    client
        .batch_execute(&format!(
            "UPDATE flashcards SET direction = 'quiz_direct' WHERE id IN ({c0}, {c2});
             DELETE FROM flashcard_quiz_options WHERE flashcard_id IN ({c0}, {c2});
             INSERT INTO flashcard_quiz_options (flashcard_id, correct_answer_text) VALUES ({c2}, 'bye');",
            c0 = cards[0],
            c2 = cards[2]
        ))
        .await
        .unwrap();
    let due = crate::flashcards::service::list_flashcards(
        &pool,
        user_id,
        crate::flashcards::dto::FlashcardListQuery {
            collection_id,
            page: None,
            per_page: Some(50),
            status: None,
            due: Some(true),
            flashcard_id: None,
            level_id: Some(level_id),
        },
    )
    .await
    .unwrap();
    let options = |fid: i32| {
        due.flashcards
            .iter()
            .find(|c| c.flashcard.id == fid)
            .and_then(|c| c.flashcard.quiz_options.clone())
    };
    assert_eq!(options(cards[0]), None);
    let opts = options(cards[2]).expect("quiz card in the due list has options");
    assert_eq!(opts.len(), 4);
    assert!(opts.contains(&"bye".to_string()));

    let _ = client
        .execute(
            "DELETE FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await;
    let _ = client
        .execute("DELETE FROM users WHERE userid = $1", &[&user_id])
        .await;
}
