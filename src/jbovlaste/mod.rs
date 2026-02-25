pub mod broadcast;
pub mod controller;
pub mod dto;
pub mod models;
pub mod service;

use broadcast::Broadcaster;

use actix_web::web;
use actix_web_grants::GrantsMiddleware;
use actix_web_httpauth::middleware::HttpAuthentication;
pub use dto::*;
pub use models::*;

use crate::auth::extractor::extract_authorities;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let broadcaster = Broadcaster::create();
    cfg.app_data(web::Data::from(broadcaster)).service(
        web::scope("jbovlaste")
            .service(controller::get_sitemap)
            .service(controller::get_valsi_sound)
            .service(controller::get_definitions_by_entry)
            .service(controller::get_entry_details)
            .service(controller::search_definitions)
            .service(controller::semantic_search)
            .service(controller::get_definition)
            .service(controller::list_definitions)
            .service(controller::list_non_lojban_definitions)
            .service(controller::get_definition_image)
            .service(controller::get_recent_changes)
            .service(controller::list_valsi_types)
            .service(
                web::scope("")
                    .wrap(GrantsMiddleware::with_extractor(extract_authorities))
                    .wrap(HttpAuthentication::bearer(crate::auth::validator))
                    .service(controller::add_definition)
                    .service(controller::bulk_import_definitions)
                    .service(controller::cancel_bulk_import)
                    .service(controller::delete_bulk_definitions)
                    .service(controller::update_definition)
                    .service(controller::delete_definition)
                    .service(controller::get_vote)
                    .service(controller::update_vote)
                    .service(controller::list_bulk_import_clients_handler)
                    .service(controller::upload_definition_image)
                    .service(controller::list_client_definitions_handler)
                    .service(controller::get_bulk_votes)
                    .service(controller::link_definitions_handler)
                    .service(controller::unlink_definitions_handler),
            )
            .service(controller::get_translations_handler)
            .service(controller::get_definition_link_handler)
            .service(controller::export_pairs_handler),
    );
}
