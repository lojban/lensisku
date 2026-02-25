pub mod controller;
pub mod dto;
pub mod models;
mod service;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/flashcards")
            .wrap(actix_web_httpauth::middleware::HttpAuthentication::bearer(
                crate::auth::validator,
            ))
            .service(controller::create_flashcard)
            .service(controller::delete_flashcard)
            .service(controller::list_flashcards)
            .service(controller::review_flashcard)
            .service(controller::submit_answer)
            .service(controller::get_due_cards)
            .service(controller::reset_progress)
            .service(controller::update_flashcard_position)
            .service(controller::import_from_collection)
            .service(controller::get_streak)
            .service(controller::update_level)
            .service(controller::add_cards)
            .service(controller::create_level)
            .service(controller::list_levels)
            .service(controller::merge_progress)
            .service(controller::list_level_cards)
            .service(controller::remove_card_from_level)
            .service(controller::delete_level)
            .service(controller::submit_fillin_answer)
            .service(controller::snooze_flashcard)
            .service(controller::submit_quiz_answer)
            .service(controller::generate_quiz_options),
    );
}
