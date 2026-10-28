//! Pins `delete_bulk_definitions`: definitions tagged with a client_id are deleted with their
//! dependent rows, except those with comments, which are reported as skipped.
//! Needs `COLLECTIONS_TEST_DATABASE_URL` (disposable DB with the lensisku schema).
#![allow(clippy::unwrap_used)]

use super::service::delete_bulk_definitions;

async fn count(client: &deadpool_postgres::Client, sql: &str, ids: &[i32]) -> i64 {
    client.query_one(sql, &[&ids]).await.unwrap().get(0)
}

#[tokio::test]
#[ignore = "requires COLLECTIONS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn delete_bulk_definitions_current_behaviour() {
    let url = crate::test_db_guard::disposable_database_url("COLLECTIONS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(2)
        .build()
        .unwrap();
    let client = pool.get().await.unwrap();
    let suffix = std::process::id();
    let client_id = format!("bulk-client-{suffix}");

    let user_id: i32 = client
        .query_one(
            "INSERT INTO users (username, password, email, created_at, role) VALUES ($1, 'x', $2, now(), 'user') RETURNING userid",
            &[&format!("bulk_t_{suffix}"), &format!("bulk_t_{suffix}@example.invalid")],
        )
        .await
        .unwrap()
        .get(0);
    let valsi_id: i32 = client
        .query_one(
            "INSERT INTO valsi (word, typeid, userid, time) VALUES ($1, (SELECT min(typeid) FROM valsitypes), $2, 0) RETURNING valsiid",
            &[&format!("zzbulk{suffix}"), &user_id],
        )
        .await
        .unwrap()
        .get(0);
    let mut defs = Vec::new();
    for (n, tag) in [
        (1, &client_id),
        (2, &client_id),
        (3, &client_id),
        (4, &"other".to_string()),
    ] {
        defs.push(
            client
                .query_one(
                    "INSERT INTO definitions (langid, valsiid, definitionnum, definition, userid, time, metadata)
                     VALUES ((SELECT min(langid) FROM languages), $1, $2, 'd', $3, 0, jsonb_build_object('client_id', $4::text))
                     RETURNING definitionid",
                    &[&valsi_id, &n, &user_id, tag],
                )
                .await
                .unwrap()
                .get::<_, i32>(0),
        );
    }
    let (plain, commented, bare, other) = (defs[0], defs[1], defs[2], defs[3]);

    // Dependent rows for `plain` (deleted) and `commented` (kept).
    for def in [plain, commented] {
        client
            .execute(
                "INSERT INTO definitionvotes (valsiid, langid, definitionid, value, userid, time)
                 SELECT valsiid, langid, definitionid, 1, $2, 0 FROM definitions WHERE definitionid = $1",
                &[&def, &user_id],
            )
            .await
            .unwrap();
        client
            .execute(
                "INSERT INTO definition_versions (definition_id, langid, valsiid, definition, user_id, message)
                 SELECT definitionid, langid, valsiid, definition, $2, 'v' FROM definitions WHERE definitionid = $1",
                &[&def, &user_id],
            )
            .await
            .unwrap();
    }
    let thread_id: i32 = client
        .query_one(
            "INSERT INTO threads (definitionid) VALUES ($1) RETURNING threadid",
            &[&commented],
        )
        .await
        .unwrap()
        .get(0);
    client
        .execute(
            "INSERT INTO comments (threadid, userid, commentnum, \"time\", subject) VALUES ($1, $2, 1, 0, 's')",
            &[&thread_id, &user_id],
        )
        .await
        .unwrap();

    let (mut deleted, skipped) = delete_bulk_definitions(&pool, &client_id).await.unwrap();
    deleted.sort_unstable();
    assert_eq!(deleted, vec![plain, bare]);
    assert_eq!(skipped, vec![commented]);

    let all = [plain, commented, bare, other];
    let left = "SELECT count(*) FROM definitions WHERE definitionid = ANY($1)";
    assert_eq!(count(&client, left, &all).await, 2);
    let votes = "SELECT count(*) FROM definitionvotes WHERE definitionid = ANY($1)";
    assert_eq!(count(&client, votes, &[plain]).await, 0);
    assert_eq!(count(&client, votes, &[commented]).await, 1);
    let versions = "SELECT count(*) FROM definition_versions WHERE definition_id = ANY($1)";
    assert_eq!(count(&client, versions, &[plain]).await, 0);
    assert_eq!(count(&client, versions, &[commented]).await, 1);

    // Nothing left to delete: a second run deletes nothing and skips the commented one again.
    let again = delete_bulk_definitions(&pool, &client_id).await.unwrap();
    assert_eq!(again, (vec![], vec![commented]));
}
