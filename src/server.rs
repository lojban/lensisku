use crate::auth::permissions::PermissionCache;
use crate::flashcards;
use crate::middleware::limiter::EmailConfirmationLimiter;
use crate::{
    assistant, auth, collections, comments,
    config::AppConfig,
    error::{AppError, AppResult},
    export, jbovlaste, language,
    mailarchive::{self},
    middleware::{self, cache::RedisCache, limiter::{LoginLimiter, PasswordResetLimiter}, panic_handler::CatchPanicWithMessage},
    sessions, subscriptions, users,
    versions::{self},
    waves,
};
use actix_cors::Cors;
use actix_limitation::RateLimiter;
use actix_web::{http::header, middleware::Logger, web, App, HttpServer};
use camxes_rs::peg::grammar::Peg;
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

    let redis_cache_data = web::Data::from(redis_cache);

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
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .wrap(CatchPanicWithMessage)
            .wrap(Logger::default())
            .wrap(cors)
            .wrap(RateLimiter::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(worker_parsers_data.clone()) // Pass the worker-specific parser map
            .app_data(perm_cache.clone())
            .app_data(general_limiter.clone())
            .app_data(password_reset_limiter.clone())
            .app_data(email_confirmation_limiter.clone())
            .app_data(login_limiter.clone())
            .app_data(redis_cache_data.clone())
            .configure(auth::configure)
            .configure(users::configure)
            .configure(language::configure)
            .configure(mailarchive::configure)
            .configure(jbovlaste::configure)
            .configure(comments::configure)
            .configure(waves::configure)
            .configure(versions::configure)
            .configure(export::configure)
            .configure(subscriptions::configure)
            .configure(collections::configure)
            .configure(flashcards::configure)
            .configure(crate::openapi::configure)
            .configure(crate::payments::configure)
            .configure(sessions::controller::init_routes)
            .configure(assistant::configure)
    })
    .workers(num_workers)
    .bind("0.0.0.0:8080")
    .map_err(AppError::Io)?
    .run()
    .await
    .map_err(AppError::Io)
}
