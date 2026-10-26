//! Information cards (`just_information`) are prose: their text must never become a quiz option
//! (collection distractors, image distractors, or replayed past wrong answers).
//!
//! Needs `FLASHCARDS_TEST_DATABASE_URL` (disposable DB with the lensisku schema).
#![allow(clippy::unwrap_used)]

use super::service::get_quiz_options_for_listing;

const INFO_FRONT: &str = "About this level: INFO FRONT";
const INFO_BACK: &str = "INFO BACK prose that explains grammar";

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn info_card_text_is_never_a_quiz_option() {
    let url = crate::test_db_guard::disposable_database_url("FLASHCARDS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(4)
        .build()
        .unwrap();
    let mut client = pool.get().await.unwrap();
    let suffix = format!("{}_{}", std::process::id(), rand::random::<u32>());
    let user_id: i32 = client
        .query_one(
            "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'x', $2, now(), 'user') RETURNING userid",
            &[&format!("icd_{suffix}"), &format!("icd_{suffix}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);
    let collection_id: i32 = client
        .query_one(
            "INSERT INTO collections (user_id, name, is_public) VALUES ($1, 'icd', false) RETURNING collection_id",
            &[&user_id],
        )
        .await
        .unwrap()
        .get(0);

    let mut quiz_id = 0;
    let cards: [(&str, &str, &str); 5] = [
        (INFO_FRONT, INFO_BACK, "just_information"),
        ("coi", "hi", "quiz_direct"),
        ("co'o", "bye", "direct"),
        ("ki'e", "thanks", "direct"),
        ("je'e", "roger", "direct"),
    ];
    for (pos, (front, back, dir)) in cards.into_iter().enumerate() {
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
        if dir == "quiz_direct" {
            quiz_id = fid;
        }
    }
    client
        .execute(
            "INSERT INTO flashcard_quiz_options (flashcard_id, correct_answer_text) VALUES ($1, 'hi')",
            &[&quiz_id],
        )
        .await
        .unwrap();
    // A past wrong pick that happens to be info-card text must not be replayed as a distractor.
    client
        .execute(
            "INSERT INTO user_quiz_answer_history
                 (user_id, flashcard_id, selected_option_text, is_correct_selection, answered_at)
             VALUES ($1, $2, $3, false, now()), ($1, $2, $4, false, now())",
            &[&user_id, &quiz_id, &INFO_BACK, &INFO_FRONT],
        )
        .await
        .unwrap();

    let tx = client.transaction().await.unwrap();
    for _ in 0..40 {
        for side in ["direct", "reverse"] {
            let (_, options) = get_quiz_options_for_listing(&tx, quiz_id, user_id, side)
                .await
                .unwrap();
            assert!(
                !options.iter().any(|o| o == INFO_FRONT || o == INFO_BACK),
                "info-card text offered as a {side} option: {options:?}"
            );
        }
    }
    let (_, direct) = get_quiz_options_for_listing(&tx, quiz_id, user_id, "direct")
        .await
        .unwrap();
    // The three regular backs still fill the distractor slots (no info text, no fallbacks).
    for want in ["hi", "bye", "thanks", "roger"] {
        assert!(
            direct.iter().any(|o| o == want),
            "{want} missing: {direct:?}"
        );
    }
    tx.rollback().await.unwrap();
    for sql in [
        "DELETE FROM user_quiz_answer_history WHERE flashcard_id IN (SELECT id FROM flashcards WHERE collection_id = $1)",
        "DELETE FROM flashcard_quiz_options WHERE flashcard_id IN (SELECT id FROM flashcards WHERE collection_id = $1)",
        "DELETE FROM flashcards WHERE collection_id = $1",
        "DELETE FROM collection_items WHERE collection_id = $1",
        "DELETE FROM collections WHERE collection_id = $1",
    ] {
        client.execute(sql, &[&collection_id]).await.unwrap();
    }
}
