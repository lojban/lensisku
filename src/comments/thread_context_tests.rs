//! Run only against a guarded disposable database with the lensisku schema.
#![allow(clippy::unwrap_used)]

use super::{dto::ThreadParams, service::get_thread_comments};

#[tokio::test]
#[ignore = "requires FLASHCARDS_TEST_DATABASE_URL (disposable lensisku database)"]
async fn definition_discussion_includes_all_threads_but_not_other_definitions() {
    let url = crate::test_db_guard::disposable_database_url("FLASHCARDS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let pool = deadpool_postgres::Pool::builder(deadpool_postgres::Manager::new(
        config,
        tokio_postgres::NoTls,
    ))
    .max_size(4)
    .build()
    .unwrap();
    let client = pool.get().await.unwrap();
    let suffix = format!("discussion_test_{}", uuid::Uuid::new_v4());
    let uid: i32 = client.query_one(
        "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'fake-password', 'fake@example.invalid', now(), 'user') RETURNING userid",
        &[&suffix],
    ).await.unwrap().get(0);
    let vid: i32 = client.query_one(
        "INSERT INTO valsi (word, typeid, userid, time) VALUES ($1, 1, $2, 0) RETURNING valsiid",
        &[&suffix, &uid],
    ).await.unwrap().get(0);
    let mut definitions = Vec::new();
    for n in 1..=2i32 {
        definitions.push(client.query_one(
            "INSERT INTO definitions (valsiid, langid, definitionnum, definition, userid, time) VALUES ($1, 2, $2, 'fake definition', $3, 0) RETURNING definitionid",
            &[&vid, &n, &uid],
        ).await.unwrap().get::<_, i32>(0));
    }
    let mut threads = Vec::new();
    let mut comments = Vec::new();
    // Two populated threads and an empty thread for this definition; another definition
    // and an entry-level discussion must not affect its badge or discussion results.
    for did in [
        Some(definitions[0]),
        Some(definitions[0]),
        Some(definitions[0]),
        Some(definitions[1]),
        None,
    ] {
        let tid: i32 = client
            .query_one(
                "INSERT INTO threads (valsiid, definitionid) VALUES ($1, $2) RETURNING threadid",
                &[&vid, &did],
            )
            .await
            .unwrap()
            .get(0);
        threads.push(tid);
        if threads.len() != 3 {
            comments.push(client.query_one(
                "INSERT INTO comments (threadid, userid, commentnum, time, subject, content) VALUES ($1, $2, 1, 0, 'fake subject', '[]'::jsonb) RETURNING commentid",
                &[&tid, &uid],
            ).await.unwrap().get::<_, i32>(0));
        }
    }
    let params = |thread_id, definition_id, scroll_to, per_page| ThreadParams {
        thread_id,
        valsi_id: Some(vid),
        definition_id,
        scroll_to,
        per_page,
        natlang_word_id: None,
        definition_link_id: None,
        target_user_id: None,
        collection_id: None,
        comment_id: None,
        current_user_id: None,
        page: None,
    };
    let result = get_thread_comments(&pool, params(None, Some(definitions[0]), None, None))
        .await
        .unwrap();
    assert_eq!(result.total, 2);
    assert_eq!(
        result
            .comments
            .iter()
            .map(|c| c.comment_id)
            .collect::<Vec<_>>(),
        comments[..2]
    );
    let result = get_thread_comments(
        &pool,
        params(Some(threads[0]), Some(definitions[0]), None, None),
    )
    .await
    .unwrap();
    assert_eq!(result.total, 1);
    let result = get_thread_comments(
        &pool,
        params(None, Some(definitions[0]), Some(comments[1]), Some(1)),
    )
    .await
    .unwrap();
    assert_eq!(result.page, 2);
    assert_eq!(result.comments[0].comment_id, comments[1]);
    let result = get_thread_comments(&pool, params(None, None, None, None))
        .await
        .unwrap();
    assert_eq!(result.total, 1);
    let details = crate::jbovlaste::service::get_definitions_by_entry(
        &pool,
        &vid.to_string(),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(
        details
            .iter()
            .find(|d| d.definitionid == definitions[0])
            .unwrap()
            .comment_count,
        Some(2)
    );
}
