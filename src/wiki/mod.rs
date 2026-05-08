//! Mirror of `mw.lojban.org` articles surfaced as the `wiki` source for `/waves`.
//!
//! - Storage: `wiki_articles` (see `migrations/V100__create_wiki_articles.sql`).
//! - Background sync: [`importer::sync_on_startup`] / [`importer::run_incremental_sync`],
//!   wired in `src/background/service.rs`.
//! - Search/list helpers consumed by [`crate::waves`] live in [`service`].

pub mod controller;
pub mod dto;
pub mod importer;
pub mod markdown;
pub mod models;
pub mod service;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/wiki").service(controller::get_wiki_article));
}
