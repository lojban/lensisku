//! Replaces the body of every 5xx response that is not already sanitized with the generic
//! `{error, message, code: "internal_error", error_id}` body, and logs the original body
//! under that id. Handlers that still format internal errors (SQL text, file paths, upstream
//! bodies) into a 500 therefore never leak them to clients.

use actix_web::{
    body::{to_bytes_limited, BoxBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    http::header,
    middleware::Next,
    Error,
};

use crate::error::{internal_error_response, new_error_id, SanitizedServerError};

/// Longest original body read into the log line.
const MAX_LOGGED_BODY_BYTES: usize = 16 * 1024;

pub async fn hide_server_error_details(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let method = req.method().clone();
    let path = req.path().to_string();
    // Errors returned as `Err` (rather than as a response) come from inner middleware such as
    // the panic catcher, whose bodies are already generic; the server renders them later.
    // (The request must not be cloned before routing, so there is no request to build from.)
    let response = next.call(req).await?.map_into_boxed_body();

    let status = response.status();
    if !status.is_server_error()
        || response
            .response()
            .extensions()
            .contains::<SanitizedServerError>()
    {
        return Ok(response);
    }

    let (http_request, original) = response.into_parts();
    let mut headers = original.headers().clone();
    headers.remove(header::CONTENT_TYPE);
    headers.remove(header::CONTENT_LENGTH);
    let detail = match to_bytes_limited(original.into_body(), MAX_LOGGED_BODY_BYTES).await {
        Ok(Ok(bytes)) => String::from_utf8_lossy(&bytes).into_owned(),
        Ok(Err(_)) => "<unreadable body>".to_string(),
        Err(_) => format!("<body over {MAX_LOGGED_BODY_BYTES} bytes>"),
    };

    let error_id = new_error_id();
    log::error!("[{error_id}] {method} {path} -> {status}: {detail}");
    let mut sanitized = internal_error_response(status, &error_id);
    for (name, value) in headers.iter() {
        sanitized.headers_mut().append(name.clone(), value.clone());
    }
    Ok(ServiceResponse::new(http_request, sanitized))
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, middleware::from_fn, test, web, App, HttpResponse};

    use super::hide_server_error_details;
    use crate::error::AppError;

    const LEAK: &str = "duplicate key value violates unique constraint \"users_email_key\"";

    async fn call(path: &str) -> (StatusCode, serde_json::Value) {
        let app = test::init_service(
            App::new()
                .wrap(from_fn(hide_server_error_details))
                .route(
                    "/raw",
                    web::get().to(|| async {
                        HttpResponse::InternalServerError()
                            .json(serde_json::json!({ "error": format!("Error: {LEAK}") }))
                    }),
                )
                .route(
                    "/text",
                    web::get().to(|| async { HttpResponse::InternalServerError().body(LEAK) }),
                )
                .route(
                    "/app_error",
                    web::get().to(|| async {
                        Err::<HttpResponse, AppError>(AppError::Database(LEAK.to_string()))
                    }),
                )
                .route(
                    "/conflict",
                    web::get().to(|| async {
                        Err::<HttpResponse, AppError>(AppError::Conflict("exists".to_string()))
                    }),
                )
                .route(
                    "/ok",
                    web::get()
                        .to(|| async { HttpResponse::Ok().json(serde_json::json!({"a": 1})) }),
                ),
        )
        .await;
        let response =
            test::call_service(&app, test::TestRequest::get().uri(path).to_request()).await;
        let status = response.status();
        let body = test::read_body(response).await;
        let json = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    fn assert_generic(json: &serde_json::Value) {
        let text = json.to_string();
        assert!(
            !text.contains("duplicate key"),
            "internal text leaked: {text}"
        );
        assert!(
            !text.contains("users_email_key"),
            "internal text leaked: {text}"
        );
        assert_eq!(json["code"], "internal_error");
        assert_eq!(json["error"], "Internal server error");
        assert_eq!(json["error_id"].as_str().map(str::len), Some(12));
    }

    #[actix_web::test]
    async fn handler_json_500_is_sanitized() {
        let (status, json) = call("/raw").await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_generic(&json);
    }

    #[actix_web::test]
    async fn handler_text_500_is_sanitized() {
        let (status, json) = call("/text").await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_generic(&json);
    }

    #[actix_web::test]
    async fn app_error_500_is_sanitized() {
        let (status, json) = call("/app_error").await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_generic(&json);
    }

    #[actix_web::test]
    async fn client_errors_keep_message_and_code() {
        let (status, json) = call("/conflict").await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(json["code"], "conflict");
        assert_eq!(json["error"], "Conflict: exists");
    }

    #[actix_web::test]
    async fn success_is_untouched() {
        let (status, json) = call("/ok").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json, serde_json::json!({"a": 1}));
    }
}
