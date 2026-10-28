//! Pins `import_from_collection`: one flashcard per collection item not yet carded, at
//! position `max + 1 + index-in-collection`, each with direct and reverse progress rows.
//! Needs `FLASHCARDS_TEST_DATABASE_URL` (disposable DB with the lensisku schema).
#![allow(clippy::unwrap_used)]

use std::fmt::Write;

use super::service::import_from_collection;

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn flashcard_import_current_behaviour() {
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
            "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'x', $2, now(), 'user') RETURNING userid",
            &[&format!("ifc_{sfx}"), &format!("ifc_{sfx}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);
    let cid: i32 = c
        .query_one(
            "INSERT INTO collections (user_id, name, is_public) VALUES ($1, 'ifc', false) RETURNING collection_id",
            &[&uid],
        )
        .await
        .unwrap()
        .get(0);
    let mut items = Vec::new();
    for pos in 0..4 {
        items.push(
            c.query_one(
                "INSERT INTO collection_items (collection_id, free_content_front, free_content_back, position)
                 VALUES ($1, $2, 'b', $3) RETURNING item_id",
                &[&cid, &format!("f{pos}"), &pos],
            )
            .await
            .unwrap()
            .get::<_, i32>(0),
        );
    }
    // Item 1 already has a card at position 5, so new cards start at 6.
    c.execute(
        "INSERT INTO flashcards (collection_id, item_id, position) VALUES ($1, $2, 5)",
        &[&cid, &items[1]],
    )
    .await
    .unwrap();

    let res = import_from_collection(&pool, uid, cid).await.unwrap();
    assert_eq!((res.imported_count, res.skipped_count), (3, 1));

    let mut got = String::new();
    for r in c
        .query(
            "SELECT ci.position, f.position, f.direction::text,
                    (SELECT string_agg(p.card_side::text, ',' ORDER BY p.card_side::text)
                     FROM user_flashcard_progress p WHERE p.flashcard_id = f.id AND p.user_id = $2)
             FROM flashcards f JOIN collection_items ci ON ci.item_id = f.item_id
             WHERE f.collection_id = $1 ORDER BY ci.position",
            &[&cid, &uid],
        )
        .await
        .unwrap()
    {
        writeln!(
            got,
            "item={} card={} dir={} progress={:?}",
            r.get::<_, i32>(0),
            r.get::<_, i32>(1),
            r.get::<_, String>(2),
            r.get::<_, Option<String>>(3)
        )
        .unwrap();
    }
    let expected = "\
item=0 card=6 dir=both progress=Some(\"direct,reverse\")
item=1 card=5 dir=direct progress=None
item=2 card=8 dir=both progress=Some(\"direct,reverse\")
item=3 card=9 dir=both progress=Some(\"direct,reverse\")
";
    assert_eq!(got, expected);

    // Re-import: everything is skipped.
    let again = import_from_collection(&pool, uid, cid).await.unwrap();
    assert_eq!((again.imported_count, again.skipped_count), (0, 4));
}
