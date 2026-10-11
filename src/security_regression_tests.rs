//! Run explicitly against disposable services; never a production database.
#![allow(clippy::unwrap_used)]
use crate::{auth, collections, comments, flashcards};

#[tokio::test]
#[ignore = "requires disposable PostgreSQL and Redis (SECURITY_TEST_DB_PORT and SECURITY_TEST_REDIS_URL)"]
async fn security_boundaries_against_disposable_services() {
    let port: u16 = std::env::var("SECURITY_TEST_DB_PORT")
        .unwrap()
        .parse()
        .unwrap();
    let redis_url = std::env::var("SECURITY_TEST_REDIS_URL").unwrap();
    let mut config = deadpool_postgres::Config::new();
    config.host = Some("127.0.0.1".into());
    config.port = Some(port);
    config.user = Some("postgres".into());
    config.password = Some("security-test-only".into());
    config.dbname = Some("security_fixture".into());
    let pool = config
        .create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )
        .unwrap();
    let client = pool.get().await.unwrap();
    client
        .batch_execute(include_str!("security_regression_fixture.sql"))
        .await
        .unwrap();
    client
        .batch_execute(include_str!(
            "../migrations/V184__oauth_browser_transactions.sql"
        ))
        .await
        .unwrap();
    client
        .batch_execute(include_str!(
            "../migrations/V185__comment_visibility_policy.sql"
        ))
        .await
        .unwrap();
    client
        .batch_execute(include_str!("../migrations/V18__password_reset.sql"))
        .await
        .unwrap();

    // Real migration protects a legacy verifier without requiring a new password.
    client
        .execute(
            "UPDATE users SET password = $1 WHERE userid = 1",
            &[&format!("{:x}", md5::compute(b"byq cnffjbeq"))],
        )
        .await
        .unwrap();
    client
        .batch_execute(include_str!(
            "../migrations/V186__protect_legacy_password_verifiers.sql"
        ))
        .await
        .unwrap();
    let stored: String = client
        .query_one("SELECT password FROM users WHERE userid = 1", &[])
        .await
        .unwrap()
        .get(0);
    assert!(stored.starts_with("legacy-bcrypt:"));
    assert!(auth::service::verify_password("old password", &stored).unwrap());
    let pair = auth::service::login(
        &pool,
        &auth::dto::LoginRequest {
            username_or_email: "owner".into(),
            password: "old password".into(),
        },
        "127.0.0.1".into(),
        "test".into(),
    )
    .await
    .unwrap();
    let claims = auth::models::authenticate_token(&pool, &pair.access_token)
        .await
        .unwrap();
    let stored: String = client
        .query_one("SELECT password FROM users WHERE userid = 1", &[])
        .await
        .unwrap()
        .get(0);
    assert!(stored.starts_with("$2"));
    assert!(auth::service::verify_password("old password", &stored).unwrap());
    client
        .execute("UPDATE users SET role = 'user' WHERE userid = 1", &[])
        .await
        .unwrap();
    let fresh = auth::models::authenticate_token(&pool, &pair.access_token)
        .await
        .unwrap();
    assert_eq!(fresh.role, "user");
    assert!(fresh.authorities.is_empty());
    let mut forged_subject = claims.clone();
    forged_subject.sub = 2;
    assert!(auth::models::validate_claims(&pool, forged_subject)
        .await
        .is_err());
    let rotated = auth::service::refresh_tokens(
        &pool,
        &pair.refresh_token,
        "127.0.0.1".into(),
        "test".into(),
    )
    .await
    .unwrap();
    assert!(auth::models::authenticate_token(&pool, &pair.access_token)
        .await
        .is_err());
    assert!(auth::service::refresh_tokens(
        &pool,
        &pair.refresh_token,
        "127.0.0.1".into(),
        "test".into()
    )
    .await
    .is_err());
    assert!(
        auth::models::authenticate_token(&pool, &rotated.access_token)
            .await
            .is_ok()
    );
    client.execute("INSERT INTO password_reset_requests(email,session_id,token,token_expiry,created_at) VALUES ('owner@example.test','test-session','test-token',$1,0)", &[&(chrono::Utc::now().timestamp()+60)]).await.unwrap();
    let reset: auth::dto::PasswordRestoreRequest = serde_json::from_value(serde_json::json!({"session_id":"test-session", "token":"test-token", "new_password":"New Password 123!"})).unwrap();
    auth::service::restore_password(&pool, &reset)
        .await
        .unwrap();
    assert!(
        auth::models::authenticate_token(&pool, &rotated.access_token)
            .await
            .is_err()
    );
    assert!(auth::service::refresh_tokens(
        &pool,
        &rotated.refresh_token,
        "127.0.0.1".into(),
        "test".into()
    )
    .await
    .is_err());
    assert!(auth::service::restore_password(&pool, &reset)
        .await
        .is_err());

    // Shared privacy policy: anonymous, owner, outsider, public, unknown IDs.
    for (thread, viewer, expected) in [
        (1, None, false),
        (1, Some(1), true),
        (1, Some(2), false),
        (2, None, true),
        (2, Some(2), true),
        (999, None, false),
    ] {
        let allowed: bool = client
            .query_one("SELECT can_read_comment_thread($1,$2)", &[&thread, &viewer])
            .await
            .unwrap()
            .get(0);
        assert_eq!(allowed, expected);
    }
    let params = |viewer| comments::dto::ThreadParams {
        thread_id: None,
        comment_id: Some(1),
        current_user_id: viewer,
        valsi_id: None,
        natlang_word_id: None,
        definition_id: None,
        definition_link_id: None,
        target_user_id: None,
        collection_id: None,
        scroll_to: None,
        page: None,
        per_page: None,
    };
    assert!(comments::service::get_thread_comments(&pool, params(None))
        .await
        .is_err());
    assert!(
        comments::service::get_thread_comments(&pool, params(Some(2)))
            .await
            .is_err()
    );
    assert_eq!(
        comments::service::get_thread_comments(&pool, params(Some(1)))
            .await
            .unwrap()
            .total,
        1
    );
    let reply = comments::dto::NewCommentParams {
        pool: pool.clone(),
        user_id: 2,
        parent_id: Some(1),
        valsi_id: None,
        natlang_word_id: None,
        definition_id: None,
        definition_link_id: None,
        target_user_id: None,
        collection_id: None,
        subject: "injected".into(),
        content: "injected".into(),
    };
    assert!(comments::service::add_comment(reply)
        .await
        .unwrap_err()
        .to_string()
        .contains("Access denied"));
    assert_eq!(
        comments::service::list_comments(&pool, 1, 20, "desc", None)
            .await
            .unwrap()
            .total,
        1
    );
    assert_eq!(
        comments::service::list_comments(&pool, 1, 20, "desc", Some(1))
            .await
            .unwrap()
            .total,
        2
    );
    assert!(comments::service::get_comment_stats(&pool, 1, None)
        .await
        .is_err());
    assert!(comments::service::get_comment_stats(&pool, 1, Some(1))
        .await
        .is_ok());

    let redis =
        crate::middleware::cache::RedisCache::new(&redis_url, std::time::Duration::from_secs(60))
            .unwrap();
    assert!(collections::service::clone_collection(&pool, &redis, 1, 2)
        .await
        .unwrap_err()
        .to_string()
        .contains("Access denied"));
    assert_eq!(
        collections::service::clone_collection(&pool, &redis, 3, 2)
            .await
            .unwrap()
            .item_count,
        1
    );
    assert!(
        collections::service::get_item_image(&pool, 1, "front", None)
            .await
            .is_err()
    );
    assert!(
        collections::service::get_item_image(&pool, 1, "front", Some(2))
            .await
            .is_err()
    );
    assert!(
        collections::service::get_item_image(&pool, 1, "front", Some(1))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        collections::service::get_item_image(&pool, 3, "front", None)
            .await
            .unwrap()
            .is_none()
    );
    assert!(collections::service::get_item_sound(&pool, 1, None)
        .await
        .is_err());
    assert!(collections::service::get_item_sound(&pool, 3, None)
        .await
        .unwrap()
        .is_none());
    let update: collections::dto::UpdateItemRequest =
        serde_json::from_value(serde_json::json!({"notes":"changed"})).unwrap();
    assert!(
        collections::service::update_item_images(&pool, 2, 1, 2, &update)
            .await
            .unwrap_err()
            .to_string()
            .contains("does not belong")
    );
    collections::service::update_item_images(&pool, 2, 2, 2, &update)
        .await
        .unwrap();
    let query: flashcards::dto::FlashcardListQuery =
        serde_json::from_value(serde_json::json!({"collection_id":1})).unwrap();
    assert!(flashcards::list_flashcards(&pool, 2, query)
        .await
        .unwrap_err()
        .to_string()
        .contains("Access denied"));
    let cards: flashcards::dto::AddCardsRequest =
        serde_json::from_value(serde_json::json!({"flashcard_ids":[1]})).unwrap();
    assert!(flashcards::add_cards_to_level(&pool, 1, 2, &cards)
        .await
        .unwrap_err()
        .to_string()
        .contains("does not belong"));

    // Concurrent quota reservations cannot pass the cap or the backoff timer.
    let limiter = crate::middleware::limiter::LoginLimiter::new(&redis_url).unwrap();
    let results = futures::future::join_all(
        (0..32).map(|_| limiter.reserve_login("test-ip", "canonical-user")),
    )
    .await;
    assert_eq!(
        results
            .into_iter()
            .filter(|result| matches!(result, Ok(0)))
            .count(),
        1
    );
    limiter.clear_success("canonical-user").await.unwrap();
    assert_eq!(
        limiter
            .reserve_login("test-ip", "canonical-user")
            .await
            .unwrap(),
        0
    );
    let results = futures::future::join_all((0..40).map(|i| {
        let limiter = &limiter;
        async move {
            limiter
                .reserve_login("one-ip", &format!("account-{i}"))
                .await
                .unwrap()
        }
    }))
    .await;
    assert_eq!(
        results.into_iter().filter(|seconds| *seconds == 0).count(),
        20
    );

    // Import and ordinary creation share one transaction lock and aggregate cap.
    client
        .execute(
            "INSERT INTO assistant_chats(user_id) SELECT 2 FROM generate_series(1,99)",
            &[],
        )
        .await
        .unwrap();
    let import: crate::assistant::chat_store::ImportBody =
        serde_json::from_value(serde_json::json!({"sessions":[{"messages":[]},{"messages":[]}]}))
            .unwrap();
    assert!(
        crate::assistant::chat_store::import_sessions(&pool, 2, &import)
            .await
            .is_err()
    );
    let (a, b) = tokio::join!(
        crate::assistant::chat_store::create_chat(&pool, 2),
        crate::assistant::chat_store::create_chat(&pool, 2)
    );
    assert_ne!(a.is_ok(), b.is_ok());
    let count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM assistant_chats WHERE user_id = 2",
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(count, 100);
}
