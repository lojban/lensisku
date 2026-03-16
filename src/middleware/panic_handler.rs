//! Catches panics in handlers and returns HTTP 500 with the panic message in the response body.
//!
//! Requires `panic = 'unwind'` in Cargo.toml (dev and release). Do not use as a replacement
//! for proper error handling.

use std::{
    future::ready,
    panic::AssertUnwindSafe,
    rc::Rc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    http::StatusCode,
    HttpResponse,
};
use futures::FutureExt as _;
use serde_json::json;

/// Error type that renders as JSON with the panic message.
#[derive(Debug)]
struct PanicError {
    message: String,
}

impl std::fmt::Display for PanicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl actix_web::ResponseError for PanicError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .json(json!({
                "error": "Internal server error",
                "details": self.message
            }))
            .map_into_boxed_body()
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

/// Middleware that catches panics and returns 500 with `{"error": "...", "details": "<panic message>"}`.
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
        Box<
            dyn std::future::Future<Output = Result<ServiceResponse<B>, Error>>
                + 'static,
        >,
    >;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        Box::pin(
            AssertUnwindSafe(fut)
                .catch_unwind()
                .map(|res| match res {
                    Ok(Ok(resp)) => Ok(resp),
                    Ok(Err(e)) => Err(e),
                    Err(panic_payload) => {
                        let message = panic_message(panic_payload);
                        log::error!("Handler panicked: {}", message);
                        Err(Error::from(PanicError { message }))
                    }
                }),
        )
    }
}
