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
