pub mod chat_store;
pub mod context_compress;
pub mod controller;
pub mod dto;
pub mod models;
pub mod openrouter;
pub mod persist;
mod service;
pub mod sse_ui_sync;
pub mod stored_messages;
mod wiki_title;

use actix_web::web;

pub use service::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    controller::configure(cfg);
}
