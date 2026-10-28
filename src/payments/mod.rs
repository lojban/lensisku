use crate::error::AppError;
pub mod controller;
pub mod dto;
pub mod paypal;
pub mod service;

use actix_web::web;
use async_trait::async_trait;

pub(crate) mod error;
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn verify_webhook_signature(
        &self,
        payload: &str,
        transmission_sig: &str,
        transmission_id: &str,
        transmission_time: &str,
        cert_url: &str,
    ) -> Result<bool, AppError>;
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payments")
            .service(controller::create_payment)
            .service(controller::capture_order)
            .service(controller::get_balance)
            .service(controller::paypal_webhook)
            .service(controller::create_paypal_subscription)
            .service(controller::get_paypal_subscription_details)
            .service(controller::cancel_paypal_subscription),
    );
}
