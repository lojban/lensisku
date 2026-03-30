pub mod controller;
pub mod dto;
mod service;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(controller::get_profile_image)
            .service(controller::get_public_profile)
            .service(controller::get_user_definitions)
            .service(controller::get_user_comments)
            .service(controller::list_users)
            .service(
                web::scope("")
                    .wrap(actix_web_httpauth::middleware::HttpAuthentication::bearer(
                        crate::auth::validator,
                    ))
                    .service(controller::get_user_votes)
                    .service(controller::update_profile_image)
                    .service(controller::remove_profile_image),
            ),
    );
}
