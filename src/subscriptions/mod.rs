pub mod controller;
pub mod dto;
pub mod models;
mod service;

use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subscriptions")
            .wrap(HttpAuthentication::bearer(crate::auth::validator))
            .service(controller::subscribe)
            .service(controller::unsubscribe)
            .service(controller::get_subscriptions)
            .service(controller::get_notifications)
            .service(controller::mark_notifications_read)
            .service(controller::get_subscription_state),
    );
}
