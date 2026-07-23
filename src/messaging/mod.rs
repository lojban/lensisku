pub mod controller;
pub mod dto;
pub mod encryption;
pub mod models;
pub mod service;
pub mod webrtc;
pub mod webrtc_handler; // Functional WebRTC implementation
pub mod websocket_handler; // Functional WebSocket implementation

use actix_web::web;
pub use service::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("messaging")
            .service(controller::get_threads)
            .service(controller::create_thread)
            .service(controller::get_thread)
            .service(controller::update_thread)
            .service(controller::delete_thread)
            .service(controller::get_thread_messages)
            .service(controller::send_message)
            .service(controller::get_message)
            // WebSocket routes
            .route(
                "/ws/{thread_id}",
                web::get().to(controller::websocket_handler),
            )
            .route("/ws", web::get().to(controller::websocket_index_handler))
            // WebRTC signaling routes
            .route("/webrtc/signal", web::post().to(controller::webrtc_signal))
            .route(
                "/webrtc/signals/{user_id}",
                web::get().to(controller::get_webrtc_signals),
            )
            .route(
                "/webrtc/signal/{signal_id}/processed",
                web::put().to(controller::mark_signal_processed),
            )
            .route(
                "/webrtc/config",
                web::get().to(controller::get_webrtc_config),
            )
            .route("/webrtc/calls", web::get().to(controller::get_active_calls))
            .route(
                "/webrtc/cleanup",
                web::delete().to(controller::cleanup_expired_signals),
            )
            // Health check
            .route("/health", web::get().to(controller::health_check)),
    );
}
