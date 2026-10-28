//! Pins `chat_store::import_sessions`: returned ids follow the request order and every
//! session is stored as sent. Needs `COLLECTIONS_TEST_DATABASE_URL` (disposable lensisku DB).
#![allow(clippy::unwrap_used)]

use super::chat_store::{import_sessions, ImportBody};

#[tokio::test]
#[ignore = "requires COLLECTIONS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn chat_import_current_behaviour() {
    let url = crate::test_db_guard::disposable_database_url("COLLECTIONS_TEST_DATABASE_URL");
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
            &[&format!("chimp_{sfx}"), &format!("chimp_{sfx}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);

    let body: ImportBody = serde_json::from_value(serde_json::json!({
        "sessions": [
            {"title": "first", "messages": [{"role": "user", "content": "coi"}], "primaryModelId": "m1", "scrollTop": 12.5},
            {"messages": []},
            {"title": "third", "messages": [{"role": "assistant", "content": "doi"}], "scrollTop": 3.0}
        ]
    }))
    .unwrap();
    let res = import_sessions(&pool, uid, &body).await.unwrap();
    assert_eq!(res.chats.len(), 3);

    let mut got = Vec::new();
    for item in &res.chats {
        let r = c
            .query_one(
                "SELECT title, messages::text, primary_model_id, scroll_top FROM assistant_chats WHERE id = $1 AND user_id = $2",
                &[&item.id, &uid],
            )
            .await
            .unwrap();
        got.push(format!(
            "{}|{}|{:?}|{}",
            r.get::<_, String>(0),
            r.get::<_, String>(1),
            r.get::<_, Option<String>>(2),
            r.get::<_, f64>(3)
        ));
    }
    assert_eq!(
        got,
        vec![
            r#"first|[{"role": "user", "content": "coi"}]|Some("m1")|12.5"#.to_string(),
            "|[]|None|0".to_string(),
            r#"third|[{"role": "assistant", "content": "doi"}]|None|3"#.to_string(),
        ]
    );

    // Empty import is a no-op.
    let empty: ImportBody = serde_json::from_value(serde_json::json!({"sessions": []})).unwrap();
    assert!(import_sessions(&pool, uid, &empty)
        .await
        .unwrap()
        .chats
        .is_empty());
}
