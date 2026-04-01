//! OpenAPI spec and Swagger UI for HTTP routes under the Rust server.
//!
//! **SPA coverage:** The Vue app calls a subset of these endpoints via `frontend/src/api.js` (axios base URL `/api`).
//! Many routes exist for other consumers (email/background jobs, admin, future UI, or non-browser clients).
//! Absence from `api.js` does not mean a route is unused server-side.

use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, SwaggerUi};

use crate::api_docs::ApiModifier;
use utoipauto::utoipauto;

#[utoipauto()]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "Users endpoints"),
        (name = "comments", description = "Discussions endpoints"), 
        (name = "language", description = "Linguistics-related endpoints"), 
        (name = "versions", description = "Definition versioning endpoints"), 
        (name = "export", description = "Dictionary exports endpoints"),
        (name = "mail", description = "Mail archive thread and message view"),
        (name = "waves", description = "Unified discussion waves and mail search"),
        (name = "jbovlaste", description = "Lojban dictionary management endpoints"),
        (name = "collections", description = "Organized bookmarks endpoints"),
        (name = "flashcards", description = "Flashcard learning system endpoints"),
        (name = "payments", description = "Payments and balance handling endpoints"),
        (name = "Sessions", description = "User session management endpoints"),
    ),
    modifiers(&ApiModifier),
    components(schemas(
        crate::comments::dto::ListCommentsQuery,
        crate::jbovlaste::dto::ListDefinitionsQuery, // Add the new DTO here
        crate::sessions::dto::PaginationParams,
        crate::sessions::dto::UserSessionDto,
        crate::flashcards::dto::SubmitQuizAnswerDto,
        crate::flashcards::dto::QuizAnswerResultDto,
        crate::collections::dto::KittenTtsGenerateRequest,
        crate::mailarchive::dto::SpamVoteResponse,
        crate::sessions::dto::PaginatedUserSessionsResponse,
        crate::waves::dto::WavesSearchResponse,
        crate::waves::dto::WavesThreadsResponse,
        crate::waves::dto::WaveSearchHit,
        crate::waves::dto::WaveThreadSummary,
    ))
)]
struct ApiDoc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let openapi = ApiDoc::openapi();
    let config = Config::new(["openapi.json"]).persist_authorization(true);
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}")
            .url("/swagger-ui/openapi.json", openapi.clone())
            .url("/api-docs/openapi.json", openapi)
            .config(config),
    );
}
