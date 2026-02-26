pub mod controller;
pub mod dto;
mod service;

use actix_web::web;
pub use service::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("assistant").service(controller::chat));
}
