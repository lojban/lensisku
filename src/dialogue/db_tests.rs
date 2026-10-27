//! Dialogue cards against a real DB: per-line audio cache, per-question answers, one review.
//! Needs `FLASHCARDS_TEST_DATABASE_URL` (disposable DB with the lensisku schema).
#![allow(clippy::unwrap_used)]

use super::service::{
    answer, generate_audio, get_view, DialogueAnswerRequest, GenerateAudioRequest,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const SCRIPT: &str = "# coi\nDjan(M): coi .marian.\n~ hi Marian\nMarian(F): lo mi ckiku cu zvati ma\n---\n? fill: Marian(F): lo mi [ckiku] cu zvati ma\n? choice: Who lost the keys?  + Marian  - Djan\n";

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn dialogue_audio_cache_and_per_question_scores() {
    let url = crate::test_db_guard::disposable_database_url("FLASHCARDS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(4)
        .build()
        .unwrap();
    let client = pool.get().await.unwrap();
    let name = format!("dlg_{}_{}", std::process::id(), rand::random::<u32>());
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
            "INSERT INTO collections (user_id, name, is_public) VALUES ($1, 'dlg', false) RETURNING collection_id",
            &[&user_id],
        )
        .await
        .unwrap()
        .get(0);
    let item_id: i32 = client
        .query_one(
            "INSERT INTO collection_items (collection_id, free_content_front, free_content_back, position)
             VALUES ($1, $2, 'notes', 0) RETURNING item_id",
            &[&collection_id, &SCRIPT],
        )
        .await
        .unwrap()
        .get(0);
    let fid: i32 = client
        .query_one(
            "INSERT INTO flashcards (collection_id, item_id, position, direction)
             VALUES ($1, $2, 0, 'dialogue') RETURNING id",
            &[&collection_id, &item_id],
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

    let calls = Arc::new(AtomicUsize::new(0));
    let synth = {
        let calls = calls.clone();
        move |text: String, voice: String| {
            calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { Ok(format!("{voice}:{text}").into_bytes()) })
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>,
                >
        }
    };
    let req = GenerateAudioRequest::default();
    let v = generate_audio(&pool, item_id, user_id, &req, synth.clone())
        .await
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(v.lines.iter().all(|l| l.has_audio));
    assert_eq!(
        (v.voices["Djan"].as_str(), v.voices["Marian"].as_str()),
        ("Martin", "Victoria")
    );
    // Unchanged: nothing re-synthesized. Edited line 2 + new line 3: two calls. Forced one line: one.
    generate_audio(&pool, item_id, user_id, &req, synth.clone())
        .await
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    let edited = SCRIPT.replace(
        "lo mi ckiku cu zvati ma\n---",
        "ma stuzi lo mi ckiku\nDjan: lo jubme\n---",
    );
    client
        .execute(
            "UPDATE collection_items SET free_content_front = $2 WHERE item_id = $1",
            &[&item_id, &edited],
        )
        .await
        .unwrap();
    // A saved voice is kept even if a speaker's first appearance changes.
    let v = generate_audio(&pool, item_id, user_id, &req, synth.clone())
        .await
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    assert_eq!(v.lines.len(), 3);
    assert_eq!(v.voices["Marian"], "Victoria");
    let one = GenerateAudioRequest {
        line: Some(0),
        force: true,
        ..Default::default()
    };
    generate_audio(&pool, item_id, user_id, &one, synth.clone())
        .await
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 5);
    // Fewer lines: stale clips are deleted.
    client
        .execute(
            "UPDATE collection_items SET free_content_front = $2 WHERE item_id = $1",
            &[&item_id, &SCRIPT],
        )
        .await
        .unwrap();
    generate_audio(&pool, item_id, user_id, &req, synth)
        .await
        .unwrap();
    let n: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM dialogue_line_sounds WHERE item_id = $1",
            &[&item_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(n, 2);
    assert!(
        get_view(&pool, item_id, None).await.is_err(),
        "private collection"
    );

    // Two questions = two rows with their own scores; finish commits one review = lowest.
    let attempt = uuid::Uuid::new_v4();
    let a1 = answer(
        &pool,
        user_id,
        fid,
        &DialogueAnswerRequest {
            attempt_id: attempt,
            question_index: Some(0),
            answer: "ckiku".into(),
            hint_used: true,
            finish: false,
        },
    )
    .await
    .unwrap();
    assert!(a1.correct);
    assert_eq!((a1.rating, a1.card_rating), (2, None));
    let a2 = answer(
        &pool,
        user_id,
        fid,
        &DialogueAnswerRequest {
            attempt_id: attempt,
            question_index: Some(1),
            answer: "Marian".into(),
            hint_used: false,
            finish: true,
        },
    )
    .await
    .unwrap();
    assert_eq!((a2.correct, a2.rating, a2.card_rating), (true, 3, Some(2)));
    let rows = client
        .query(
            "SELECT question_index, correct, rating FROM dialogue_question_answers WHERE attempt_id = $1 ORDER BY question_index",
            &[&attempt],
        )
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].get::<_, i32>(2), 2);
    assert_eq!(rows[1].get::<_, i32>(2), 3);
    let (reviews, last): (i32, Option<chrono::DateTime<chrono::Utc>>) = {
        let r = client
            .query_one(
                "SELECT review_count, last_reviewed_at FROM user_flashcard_progress WHERE flashcard_id = $1",
                &[&fid],
            )
            .await
            .unwrap();
        (r.get(0), r.get(1))
    };
    assert_eq!(reviews, 1);
    assert!(last.is_some());

    for sql in [
        "DELETE FROM dialogue_question_answers WHERE flashcard_id IN (SELECT id FROM flashcards WHERE collection_id = $1)",
        "DELETE FROM flashcard_review_history WHERE flashcard_id IN (SELECT id FROM flashcards WHERE collection_id = $1)",
        "DELETE FROM user_flashcard_progress WHERE flashcard_id IN (SELECT id FROM flashcards WHERE collection_id = $1)",
        "DELETE FROM flashcards WHERE collection_id = $1",
        "DELETE FROM collection_items WHERE collection_id = $1",
        "DELETE FROM collections WHERE collection_id = $1",
    ] {
        client.execute(sql, &[&collection_id]).await.unwrap();
    }
}
