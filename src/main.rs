#![deny(dead_code)]

use dotenvy::dotenv;
use env_logger::Env;
use log::{error, info, warn};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

pub mod error;
pub use error::{AppError, AppResult};
pub mod api_docs;
mod auth;
pub mod auth_utils;
mod background;
mod collections;
mod comments;
mod config;
mod db;
pub mod embeddings;
mod export;
mod flashcards;
mod jbovlaste;
mod language;
mod mailarchive;
mod middleware;
mod muplis;
mod notifications;
mod openapi;
mod payments;
mod server;
pub mod sessions;
mod subscriptions;
mod users;
mod utils;
mod tersmu;
mod versions;


#[actix_web::main]
async fn main() -> AppResult<()> {
    dotenv().ok();

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting the Lojban Lens Search API");

    // Use ? directly as create_app_config now returns AppResult
    let config = config::create_app_config()?;

    // Enable database extensions and run migrations using import pool
    // Use ? directly as db functions now return AppResult
    db::enable_extensions(&config.db_pools.import_pool).await?;
    info!("Database extensions enabled successfully.");

    db::run_migrations(&config.db_pools.import_pool).await?;
    info!("Database migrations ran successfully.");

    // Initialize parsers
    // Use ? directly as initialize_grammar_texts now returns AppResult
    // ELI5: We're creating a special container (Arc) for our grammar rules that can be safely shared
    // between different parts of our program, like sharing a book that multiple people can read at
    // the same time without damaging it. Arc keeps track of how many parts are using it.
    let grammar_texts = Arc::new(initialize_grammar_texts()?);

    // Import initial maildir data using import pool
    let maildir_path = env::var("MAILDIR_PATH").unwrap_or("test-maildir".to_string());

    // Spawn background tasks with import pool
    background::spawn_background_tasks(config.db_pools.import_pool.clone(), maildir_path).await;

    // Initialize email service to verify configuration
    if let Err(e) = notifications::EmailService::new() {
        error!(
            "Failed to initialize email service: {}. Notifications will not be sent.",
            e
        );
    } else {
        info!("Email service initialized successfully");
    }

    info!("Starting HTTP server on 0.0.0.0:8080");
    server::start_server(config, grammar_texts).await
}

fn initialize_grammar_texts() -> AppResult<HashMap<i32, String>> {
    let mut parsers = HashMap::new();
    let grammar_dir = "src/grammar";
    std::fs::create_dir_all(grammar_dir).map_err(AppError::Io)?;

    // --- Lojban (ID 1) ---
    let lojban_grammar_path = format!("{}/lojban.peg", grammar_dir);
    let lojban_default_grammar = r#"
text <- (word / Spacing)+
word <- [a-zA-Z',.]+
Spacing <- [ \t\n\r]+
"#;
    if !std::path::Path::new(&lojban_grammar_path).exists() {
        std::fs::write(&lojban_grammar_path, lojban_default_grammar).map_err(AppError::Io)?;
        info!(
            "Created default Lojban grammar file at {}",
            lojban_grammar_path
        );
    }
    match std::fs::read_to_string(&lojban_grammar_path) {
        Ok(text) => {
            parsers.insert(1, text); // Lojban ID is 1
            info!("Read Lojban grammar from {}", lojban_grammar_path);
        }
        Err(e) => {
            error!(
                "Failed to read Lojban grammar file {}: {}",
                lojban_grammar_path, e
            );
            // Allow server to start without it, but log error.
        }
    }

    // --- Loglan (ID 58) ---
    let loglan_grammar_path = format!("{}/loglan.peg", grammar_dir);
    let loglan_default_grammar = r#"
text <- (word / Spacing)+
word <- [a-zA-Z',.]+
Spacing <- [ \t\n\r]+
"#; // Placeholder grammar
    if !std::path::Path::new(&loglan_grammar_path).exists() {
        std::fs::write(&loglan_grammar_path, loglan_default_grammar).map_err(AppError::Io)?;
        info!(
            "Created default Loglan grammar file at {}",
            loglan_grammar_path
        );
    }
    match std::fs::read_to_string(&loglan_grammar_path) {
        Ok(text) => {
            parsers.insert(58, text); // Loglan ID is 58
            info!("Read Loglan grammar from {}", loglan_grammar_path);
        }
        Err(e) => {
            // Loglan might be optional for now
            warn!(
                "Failed to load Loglan grammar: {}. Loglan analysis will not be available.",
                e
            );
        }
    }

    // Add more languages here...

    // Return even if some grammars failed to load, errors are logged above.
    Ok(parsers)
}
