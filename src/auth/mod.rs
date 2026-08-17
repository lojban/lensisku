pub mod controller;
pub mod dto;
mod email_confirmation;
pub mod error;
pub mod extractor;
pub mod models;
pub mod permissions;
pub mod service;

use actix_web::web;
use actix_web_grants::GrantsMiddleware;
use actix_web_httpauth::middleware::HttpAuthentication;
pub use dto::*;
use extractor::extract_authorities;
pub use models::{decode_token, Claims, User};
pub use service::*;
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("auth")
            // Public routes (no auth required)
            .service(controller::signup)
            .service(controller::login)
            .service(controller::logout)
            .service(controller::refresh_token)
            .service(controller::restore_password)
            .service(controller::request_password_reset)
            .service(controller::confirm_email)
            .service(controller::resend_confirmation)
            .service(controller::google_oauth_signup)
            // Protected routes (require auth)
            .service(
                web::scope("")
                    .wrap(GrantsMiddleware::with_extractor(extract_authorities))
                    .wrap(HttpAuthentication::bearer(crate::auth::validator))
                    .service(controller::get_profile)
                    .service(controller::update_profile)
                    .service(controller::set_following)
                    .service(controller::initiate_password_change)
                    .service(controller::complete_password_change)
                    .service(controller::assign_role)
                    .service(controller::block_user)
                    .service(controller::get_roles)
                    .service(controller::create_role)
                    .service(controller::update_role)
                    .service(controller::delete_role)
                    .service(controller::get_permissions),
            ),
    );
}
