//! Dialogue ("story") cards: speaker-prefixed script, per-line Kokoro audio, and questions
//! (fill / word bank / choice) answered one by one, each stored with its own score.
pub mod controller;
#[cfg(test)]
mod db_tests;
pub mod parser;
pub mod service;

use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dialogue")
            .service(controller::get_dialogue)
            .service(controller::get_line_audio)
            .service(controller::check_answer)
            .service(
                web::scope("")
                    .wrap(HttpAuthentication::bearer(crate::auth::validator))
                    .service(controller::preview)
                    .service(controller::generate_audio)
                    .service(controller::answer),
            ),
    );
}
