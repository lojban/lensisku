pub mod controller;
pub mod dto;
mod service;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/lingo").service(controller::get_leaderboard));
}
