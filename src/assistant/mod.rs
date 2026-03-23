pub mod chat_store;
pub mod context_compress;
pub mod controller;
pub mod dto;
pub mod persist;
pub mod sse_ui_sync;
pub mod stored_messages;
mod service;

use actix_web::web;

pub use service::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    controller::configure(cfg);
}
