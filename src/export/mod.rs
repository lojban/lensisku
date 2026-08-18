pub mod controller;
pub mod models;
pub mod service;

use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("export")
            .service(controller::download_cached_export)
            .service(controller::list_cached_exports)
            .service(
                web::scope("")
                    .wrap(HttpAuthentication::bearer(crate::auth::validator))
                    .service(controller::export_dictionary)
                    .service(controller::export_search),
            ),
    );
}
