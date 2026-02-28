pub mod controller;
pub mod dto;
pub mod models;
mod service;

use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

pub use models::MathJaxValidationOptions;
pub use service::{analyze_word, lujvo_segments_from_nodes, validate_mathjax, validate_mathjax_fields};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("language")
            // Public routes
            .service(controller::get_languages)
            // Protected routes
            .service(
                web::scope("")
                    .wrap(HttpAuthentication::bearer(crate::auth::validator))
                    .service(controller::analyze_word)
                    .service(controller::validate_mathjax)
                    .service(controller::parse_lojban),
            ),
    );
}
