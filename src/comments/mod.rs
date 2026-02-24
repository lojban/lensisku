use actix_web::web;
use actix_web_grants::GrantsMiddleware;

use crate::auth::extractor::extract_authorities;

pub mod controller;
pub mod dto;
mod errors;
pub mod models;
mod service;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/comments")
            .service(controller::list_threads)
            .service(controller::get_thread)
            .service(controller::get_user_comments)
            .service(controller::get_opinions)
            .service(controller::get_trending)
            .service(controller::get_comment_stats)
            .service(controller::get_most_bookmarked)
            .service(controller::search_comments)
            .service(controller::list_comments)
            .service(
                web::scope("/hashtags")
                    .service(controller::trending_hashtags)
                    .service(controller::comments_by_hashtag),
            )
            .service(
                web::scope("")
                    .wrap(GrantsMiddleware::with_extractor(extract_authorities))
                    .wrap(actix_web_httpauth::middleware::HttpAuthentication::bearer(
                        crate::auth::validator,
                    ))
                    .service(controller::add_comment)
                    .service(controller::toggle_bookmark)
                    .service(controller::delete_comment)
                    .service(controller::toggle_reaction)
                    .service(controller::get_bookmarks)
                    .service(controller::get_my_reactions)
                    .service(controller::create_opinion)
                    .service(controller::vote_opinion),
            ),
    );
}
