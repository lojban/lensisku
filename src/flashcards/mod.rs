#[cfg(test)]
mod challenge_mix_progress_tests;
pub mod controller;
#[cfg(test)]
mod course_order_tests;
#[cfg(test)]
mod course_settings_tests;
pub mod dto;
pub(crate) mod error;
#[cfg(test)]
mod level_completion_tests;
#[cfg(test)]
mod merge_progress_tests;
pub mod models;
pub(crate) mod rating_policy;
mod service;

pub(crate) use error::FlashcardError;
pub(crate) use service::generate_and_set_quiz_options;
pub use service::list_flashcards_public;
#[cfg(test)]
pub(crate) use service::{add_cards_to_level, list_flashcards};

use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/flashcards")
            // Guest-readable endpoints (Option<Claims> inside). Must stay outside the
            // bearer middleware or anonymous course study cannot load levels/cards.
            .service(controller::list_levels)
            .service(controller::list_level_cards)
            .service(controller::get_course_settings)
            .service(controller::list_item_levels)
            .service(
                web::scope("")
                    .wrap(HttpAuthentication::bearer(crate::auth::validator))
                    .service(controller::create_flashcard)
                    .service(controller::delete_flashcard)
                    .service(controller::list_flashcards)
                    .service(controller::review_flashcard)
                    .service(controller::submit_answer)
                    .service(controller::get_due_cards)
                    .service(controller::get_next_quiz)
                    .service(controller::reset_progress)
                    .service(controller::update_flashcard_position)
                    .service(controller::import_from_collection)
                    .service(controller::get_streak)
                    .service(controller::update_level)
                    .service(controller::add_cards)
                    .service(controller::create_level)
                    .service(controller::merge_progress)
                    .service(controller::claim_guest_progress)
                    .service(controller::remove_card_from_level)
                    .service(controller::delete_level)
                    .service(controller::submit_fillin_answer)
                    .service(controller::snooze_flashcard)
                    .service(controller::submit_quiz_answer)
                    .service(controller::generate_quiz_options)
                    .service(controller::apply_challenge_mix_level)
                    .service(controller::apply_challenge_mix_collection)
                    .service(controller::update_course_settings)
                    .service(controller::save_course_order),
            ),
    );
}
