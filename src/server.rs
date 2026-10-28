use crate::auth::permissions::PermissionCache;
use crate::flashcards;
use crate::middleware::limiter::EmailConfirmationLimiter;
use crate::{
    assistant, auth, collections, comments,
    config::AppConfig,
    error::{AppError, AppResult},
    export, jbovlaste, language,
    mailarchive::{self},
    messaging,
    middleware::{
        self,
        cache::RedisCache,
        limiter::{KokoroTtsLimiter, LoginLimiter, PasswordResetLimiter},
        panic_handler::CatchPanicWithMessage,
    },
    sessions, subscriptions, users,
    versions::{self},
    waves, wiki,
};
use actix::Actor;
use actix_cors::Cors;
use actix_limitation::RateLimiter;
use actix_web::{dev::Service, http::header, middleware::Logger, web, App, HttpServer};
use camxes_rs::camxes::peg::grammar::Peg;
use log::{error, info};
use std::{collections::HashMap, env, sync::Arc};

pub async fn start_server(
    config: AppConfig,
    redis_cache: Arc<RedisCache>,
    grammar_texts: Arc<HashMap<i32, String>>,
) -> AppResult<()> {
    let num_workers = num_cpus::get();
    info!("Using {} worker threads", num_workers);

    let pool = config.db_pools.app_pool;
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Configure rate limiters
    let general_limiter =
        web::Data::new(middleware::configure_rate_limiter(&redis_url).map_err(|e| {
            AppError::ExternalService(format!("Failed to initialize general rate limiter: {}", e))
        })?);

    let password_reset_limiter =
        web::Data::new(PasswordResetLimiter::new(&redis_url).map_err(|e| {
            AppError::ExternalService(format!(
                "Failed to initialize password reset limiter: {}",
                e
            ))
        })?);

    let email_confirmation_limiter =
        web::Data::new(EmailConfirmationLimiter::new(&redis_url).map_err(|e| {
            AppError::ExternalService(format!(
                "Failed to initialize email confirmation limiter: {}",
                e
            ))
        })?);

    let login_limiter = web::Data::new(LoginLimiter::new(&redis_url).map_err(|e| {
        AppError::ExternalService(format!("Failed to initialize login rate limiter: {}", e))
    })?);

    let kokoro_tts_limiter = web::Data::new(KokoroTtsLimiter::new(&redis_url).map_err(|e| {
        AppError::ExternalService(format!(
            "Failed to initialize Kokoro TTS rate limiter: {}",
            e
        ))
    })?);

    let redis_cache_data = web::Data::from(redis_cache);
    let cloudflare_cache_purger =
        web::Data::new(middleware::cloudflare_cache::CloudflareCachePurger::from_env());

    // Initialize messaging service and WebSocket broadcast server
    let chat_server = messaging::websocket::ChatServer::new().start();
    let messaging_service = web::Data::new(messaging::MessagingService::new(
        pool.clone(),
        Some(chat_server.clone()),
    ));

    let perm_cache = web::Data::from(PermissionCache::new(pool.clone()));
    perm_cache
        .load_permissions()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to load permissions: {}", e)))?;

    HttpServer::new(move || {
        // Create parsers for this specific worker thread
        let mut worker_parsers = HashMap::new();
        for (lang_id, grammar_text) in grammar_texts.iter() {
            match Peg::new("text", grammar_text) {
                Ok(parser) => {
                    worker_parsers.insert(*lang_id, parser);
                    info!(
                        "Worker {:?} initialized parser for language ID {}",
                        std::thread::current().id(),
                        lang_id
                    );
                }
                Err(e) => {
                    error!(
                        "Worker {:?} failed to initialize parser for language ID {}: {}",
                        std::thread::current().id(),
                        lang_id,
                        e
                    );
                    // Optionally skip this parser or handle the error differently
                }
            }
        }
        #[allow(clippy::arc_with_non_send_sync)]
        let worker_parsers_data = web::Data::new(Arc::new(worker_parsers)); // Wrap in Arc for sharing

        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
                header::UPGRADE,
                header::CONNECTION,
                header::SEC_WEBSOCKET_KEY,
                header::SEC_WEBSOCKET_VERSION,
                header::SEC_WEBSOCKET_PROTOCOL,
            ])
            .expose_headers(vec![
                header::UPGRADE,
                header::CONNECTION,
                header::SEC_WEBSOCKET_ACCEPT,
            ])
            // Any origin may read the public API, but never with ambient credentials:
            // the SPA is same-origin (/api) and authenticates with a bearer header.
            .max_age(3600);

        App::new()
            .wrap(CatchPanicWithMessage)
            .wrap(actix_web::middleware::from_fn(
                middleware::error_sanitizer::hide_server_error_details,
            ))
            .wrap(Logger::default())
            .wrap_fn(|req, service| {
                let tags = if req.method() == actix_web::http::Method::GET {
                    middleware::cloudflare_cache::tags_for_get(req.path(), req.uri().query())
                } else {
                    Vec::new()
                };
                let future = service.call(req);
                async move {
                    let mut response = future.await?;
                    if !tags.is_empty() {
                        if let Ok(value) = header::HeaderValue::from_str(&tags.join(",")) {
                            response
                                .response_mut()
                                .headers_mut()
                                .insert(header::HeaderName::from_static("cache-tag"), value);
                        }
                    }
                    Ok(response)
                }
            })
            .wrap(cors)
            .wrap(RateLimiter::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(worker_parsers_data.clone()) // Pass the worker-specific parser map
            .app_data(perm_cache.clone())
            .app_data(general_limiter.clone())
            .app_data(password_reset_limiter.clone())
            .app_data(email_confirmation_limiter.clone())
            .app_data(login_limiter.clone())
            .app_data(kokoro_tts_limiter.clone())
            .app_data(redis_cache_data.clone())
            .app_data(cloudflare_cache_purger.clone())
            .app_data(messaging_service.clone())
            .app_data(web::Data::new(chat_server.clone()))
            .configure(auth::configure)
            .configure(users::configure)
            .configure(language::configure)
            .configure(mailarchive::configure)
            .configure(jbovlaste::configure)
            .configure(comments::configure)
            .configure(waves::configure)
            .configure(wiki::configure)
            .configure(versions::configure)
            .configure(export::configure)
            .configure(subscriptions::configure)
            .configure(collections::configure)
            .configure(flashcards::configure)
            .configure(crate::dialogue::configure)
            .configure(crate::lingo::configure)
            .configure(crate::openapi::configure)
            .configure(crate::payments::configure)
            .configure(sessions::controller::init_routes)
            .configure(messaging::configure)
            .configure(assistant::configure)
            // These broad page routes must follow API routes such as /mail/message/{id}.
            .service(mailarchive::seo::thread_page)
            .service(mailarchive::seo::message_page)
            .service(jbovlaste::seo::valsi_page)
            .service(wiki::seo::wiki_by_id_page)
            .service(wiki::seo::wiki_page)
    })
    .workers(num_workers)
    .bind("0.0.0.0:8080")
    .map_err(AppError::Io)?
    .run()
    .await
    .map_err(AppError::Io)
}
