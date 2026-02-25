pub mod controller;
pub mod dto;
pub mod models;
mod service;

use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("collections")
            .service(controller::list_public_collections)
            .service(controller::export_collection_full)
            .service(controller::get_collection)
            .service(controller::list_collection_items)
            .service(controller::get_item_image)
            .service(controller::get_item_sound)
            .service(controller::search_collection_items)
            .service(
                web::scope("")
                    .wrap(HttpAuthentication::bearer(crate::auth::validator))
                    .service(controller::create_collection)
                    .service(controller::list_collections)
                    .service(controller::update_collection)
                    .service(controller::delete_collection)
                    .service(controller::upsert_item)
                    .service(controller::update_item_position)
                    .service(controller::update_item_notes)
                    .service(controller::remove_item)
                    .service(controller::clone_collection)
                    .service(controller::merge_collections)
                    .service(controller::update_item_media)
                    .service(controller::import_json)
                    .service(controller::import_collection_from_json)
                    .service(controller::import_full),
            ),
    );
}
