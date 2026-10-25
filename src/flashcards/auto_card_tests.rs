//! `auto` cards (V204): presented-type penalty, ladder stage, production-gated status,
//! 10-minute floor, single `direct` schedule, and a fixed card keeping the >= 3 cutoff.
//!
//! Needs `FLASHCARDS_TEST_DATABASE_URL` (disposable DB with the lensisku schema + V204).
#![allow(clippy::unwrap_used)]

use super::dto::ReviewRequest;
use super::service::review_flashcard;

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn auto_card_penalty_stage_status_and_floor() {
    let url = crate::test_db_guard::disposable_database_url("FLASHCARDS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(2)
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
            &[&format!("ac_{sfx}"), &format!("ac_{sfx}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);
    let cid: i32 = c
        .query_one(
            "INSERT INTO collections (user_id, name, is_public) VALUES ($1, 'ac', true) RETURNING collection_id",
            &[&uid],
        )
        .await
        .unwrap()
        .get(0);
    let mut fids = Vec::new();
    for (pos, dir) in [(0i32, "auto"), (1, "quiz_direct")] {
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
                &format!("INSERT INTO flashcards (collection_id, item_id, position, direction) VALUES ($1, $2, $3, '{dir}') RETURNING id"),
                &[&cid, &item, &pos],
            )
            .await
            .unwrap()
            .get(0);
        c.execute(
            "INSERT INTO user_flashcard_progress (user_id, flashcard_id, card_side) VALUES ($1, $2, 'direct')",
            &[&uid, &fid],
        )
        .await
        .unwrap();
        fids.push(fid);
    }
    let (auto, fixed) = (fids[0], fids[1]);
    let review =
        |fid: i32, rating: u32, side: &'static str, t: Option<&'static str>, hint: bool| {
            let pool = &pool;
            async move {
                review_flashcard(
                    pool,
                    uid,
                    &ReviewRequest {
                        flashcard_id: fid,
                        rating,
                        card_side: side.to_string(),
                        hint_used: hint,
                        presented_type: t.map(str::to_string),
                    },
                )
                .await
                .unwrap()
            }
        };
    let state = |fid: i32| {
        let c = &c;
        async move {
            let r = c
                .query_one(
                    "SELECT p.status::text, p.auto_stage,
                            (SELECT rating FROM flashcard_review_history WHERE user_id = $1 AND flashcard_id = $2 ORDER BY id DESC LIMIT 1),
                            (SELECT presented_type FROM flashcard_review_history WHERE user_id = $1 AND flashcard_id = $2 ORDER BY id DESC LIMIT 1),
                            p.next_review_at > now() + interval '9 minutes'
                     FROM user_flashcard_progress p WHERE p.user_id = $1 AND p.flashcard_id = $2 AND p.card_side = 'direct'",
                    &[&uid, &fid],
                )
                .await
                .unwrap();
            (
                r.get::<_, String>(0),
                r.get::<_, i16>(1),
                r.get::<_, i32>(2),
                r.get::<_, Option<String>>(3),
                r.get::<_, bool>(4),
            )
        }
    };

    // Correct word bank: stored 2 (on-screen penalty), new -> learning, stage 0 -> 1, floor.
    review(auto, 3, "direct", Some("wordbank"), false).await;
    let s = state(auto).await;
    assert_eq!(
        s,
        ("learning".into(), 1, 2, Some("wordbank".into()), true),
        "{s:?}"
    );
    // Another passing word bank: no production answer yet -> stays learning.
    review(auto, 3, "direct", Some("wordbank"), false).await;
    assert_eq!(state(auto).await.0, "learning");
    assert_eq!(state(auto).await.1, 2);
    // Correct reverse (production) on the reverse display side: rating kept, -> review,
    // scheduled on the single direct side.
    review(auto, 3, "reverse", Some("reverse"), false).await;
    let s = state(auto).await;
    assert_eq!((s.0.as_str(), s.1, s.2), ("review", 3, 3), "{s:?}");
    // Hint on a fill-in: -1 (hint only), stage unchanged.
    review(auto, 3, "direct", Some("fillin"), true).await;
    let s = state(auto).await;
    assert_eq!((s.1, s.2), (3, 2), "{s:?}");
    // Quiz + hint: penalty capped at 2 -> stored max(1, 4 - 2) = 2.
    review(auto, 4, "direct", Some("quiz_direct"), true).await;
    assert_eq!(state(auto).await.2, 2);
    // Lapse: stored 1 -> stage drops by 2, status learning.
    review(auto, 1, "direct", Some("fillin"), false).await;
    let s = state(auto).await;
    assert_eq!((s.0.as_str(), s.1, s.2), ("learning", 1, 1), "{s:?}");
    // Invalid presented type counts as on-screen (penalized) and is not stored as-is.
    review(auto, 3, "direct", Some("bogus"), false).await;
    let s = state(auto).await;
    assert_eq!((s.2, s.3.clone()), (2, None), "{s:?}");
    let sides: i64 = c
        .query_one(
            "SELECT count(*) FROM user_flashcard_progress WHERE user_id = $1 AND flashcard_id = $2",
            &[&uid, &auto],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(sides, 1, "auto cards are scheduled on one side");

    // Fixed quiz: correct (3) stored as 3 (no type penalty), new -> learning; a stored 2
    // (hint) does not advance learning (fixed cutoff stays >= 3).
    review(fixed, 3, "direct", None, false).await;
    let s = state(fixed).await;
    assert_eq!((s.0.as_str(), s.2), ("learning", 3), "{s:?}");
    review(fixed, 3, "direct", None, true).await;
    let s = state(fixed).await;
    assert_eq!((s.0.as_str(), s.2), ("learning", 2), "{s:?}");
    review(fixed, 3, "direct", None, false).await;
    assert_eq!(state(fixed).await.0, "review");

    c.execute("DELETE FROM collections WHERE collection_id = $1", &[&cid])
        .await
        .ok();
}
