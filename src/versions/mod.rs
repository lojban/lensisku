pub mod controller;
pub mod dto;
pub mod models;
pub mod service;

use actix_web::web;
use actix_web_grants::GrantsMiddleware;
pub use dto::*;
pub use models::*;

use crate::auth::extractor::extract_authorities;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("versions")
            .wrap(GrantsMiddleware::with_extractor(extract_authorities))
            .wrap(actix_web_httpauth::middleware::HttpAuthentication::bearer(
                crate::auth::validator,
            ))
            .service(controller::get_version_diff)
            .service(controller::get_definition_history)
            .service(controller::get_version)
            .service(controller::revert_to_version),
    );
}

#[cfg(test)]
mod error_mapping_tests;
