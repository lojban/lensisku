//! Catches panics in handlers, logs the panic message and returns a generic HTTP 500.
//!
//! Requires `panic = 'unwind'` in Cargo.toml (dev and release). Do not use as a replacement
//! for proper error handling.

use std::{future::ready, panic::AssertUnwindSafe, rc::Rc};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    http::StatusCode,
    HttpResponse,
};
use futures::FutureExt as _;

use crate::error::{internal_error_response, new_error_id};

/// Error for a caught panic; the message is logged, never sent to the client.
#[derive(Debug)]
struct PanicError {
    message: String,
    error_id: String,
}

impl std::fmt::Display for PanicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl actix_web::ResponseError for PanicError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        internal_error_response(StatusCode::INTERNAL_SERVER_ERROR, &self.error_id)
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        return (*s).to_string();
    }
    if let Some(s) = payload.downcast_ref::<String>() {
        return s.clone();
    }
    format!("{:?}", payload)
}

/// Middleware that catches panics and returns the generic 500 body with an `error_id`.
#[derive(Debug, Clone, Default)]
pub struct CatchPanicWithMessage;

impl<S, B> Transform<S, ServiceRequest> for CatchPanicWithMessage
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CatchPanicWithMessageMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CatchPanicWithMessageMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct CatchPanicWithMessageMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CatchPanicWithMessageMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ServiceResponse<B>, Error>> + 'static>,
    >;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        Box::pin(AssertUnwindSafe(fut).catch_unwind().map(|res| match res {
            Ok(Ok(resp)) => Ok(resp),
            Ok(Err(e)) => Err(e),
            Err(panic_payload) => {
                let message = panic_message(panic_payload);
                let error_id = new_error_id();
                log::error!("[{error_id}] Handler panicked: {}", message);
                Err(Error::from(PanicError { message, error_id }))
            }
        }))
    }
}
