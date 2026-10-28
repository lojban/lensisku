//! SQLSTATE P0002 (no_data_found) maps to `AppError::NotFound`, which the users and
//! versions handlers turn into their old 404 responses.
//! Needs `COLLECTIONS_TEST_DATABASE_URL` (disposable lensisku DB).
#![allow(clippy::unwrap_used)]

use crate::error::AppError;

#[tokio::test]
#[ignore = "requires COLLECTIONS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn p0002_error_mapping() {
    let url = crate::test_db_guard::disposable_database_url("COLLECTIONS_TEST_DATABASE_URL");
    let (client, conn) = tokio_postgres::connect(&url, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(conn);
    let err = client
        .batch_execute("DO $$ BEGIN RAISE no_data_found USING MESSAGE = 'row missing'; END $$")
        .await
        .unwrap_err();
    let e = AppError::from(err);
    assert!(matches!(e, AppError::NotFound(_)), "{e:?}");
    assert_eq!(e.detail(), "row missing");
}
