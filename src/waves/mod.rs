//! Unified HTTP API for **discussion waves**: comment threads plus mail archive, search and browse.
//!
//! Lower-level primitives live in [`crate::comments::service`] (`search_comments`, `list_threads`)
//! and [`crate::mailarchive`]; this module merges and filters them.

pub mod controller;
pub mod dto;
mod service;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/waves")
            .service(controller::search_waves)
            .service(controller::list_wave_threads),
    );
}
