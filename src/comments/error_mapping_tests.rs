//! Typed errors from the comments service keep the messages the handlers match on.
//! Needs `COLLECTIONS_TEST_DATABASE_URL` (disposable lensisku DB).
#![allow(clippy::unwrap_used)]

use super::service;
use crate::error::AppError;

#[tokio::test]
#[ignore = "requires COLLECTIONS_TEST_DATABASE_URL (disposable DB with the lensisku schema)"]
async fn comments_error_mapping() {
    let url = crate::test_db_guard::disposable_database_url("COLLECTIONS_TEST_DATABASE_URL");
    let config = url.parse::<tokio_postgres::Config>().unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(2)
        .build()
        .unwrap();

    let e = service::delete_comment(&pool, i32::MAX, 1)
        .await
        .unwrap_err();
    assert!(matches!(e, AppError::NotFound(_)), "{e:?}");
    assert_eq!(e.detail(), "Comment not found");

    let e = service::toggle_reaction(&pool, 1, 1, &"x".repeat(40))
        .await
        .unwrap_err();
    assert!(matches!(e, AppError::BadRequest(_)), "{e:?}");
    assert!(e.detail().contains("too long"));
}
