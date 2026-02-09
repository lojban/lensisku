use actix_web::{get, post, web, HttpResponse, Responder};
// use actix_web_grants::protect;
use deadpool_postgres::Pool;

use crate::auth::permissions::PermissionCache;

use super::{
    dto::{GetDiffQuery, GetVersionsQuery, VersionHistoryResponse},
    service, Version, VersionDiff,
};

#[utoipa::path(
    get,
    path = "/versions/{definition_id}/history",
    tag = "versions",
    params(
        ("definition_id" = i32, Path, description = "Definition ID"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "Version history retrieved successfully", body = VersionHistoryResponse),
        (status = 404, description = "Definition not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get definition version history",
    description = "Retrieves the version history for a specific definition, showing all changes made over time. \
                  The history includes metadata about each version such as the author, timestamp, and commit message. \
                  Results are paginated and ordered by creation date descending (newest first)."
)]
#[get("/{definition_id}/history")]
pub async fn get_definition_history(
    pool: web::Data<Pool>,
    definition_id: web::Path<i32>,
    query: web::Query<GetVersionsQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match service::get_definition_history(&pool, definition_id.into_inner(), page, per_page).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => match e.downcast_ref::<tokio_postgres::Error>() {
            Some(db_error) if db_error.code().is_some_and(|c| c.code() == "P0002") => {
                HttpResponse::NotFound().finish()
            }
            _ => HttpResponse::InternalServerError().body(format!("Failed to get history: {}", e)),
        },
    }
}

#[utoipa::path(
    get,
    path = "/versions/{version_id}/version",
    tag = "versions",
    params(
        ("version_id" = i32, Path, description = "Version ID to retrieve")
    ),
    responses(
        (status = 200, description = "Version retrieved successfully", body = Version),
        (status = 404, description = "Version not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get specific version details",
    description = "Retrieves detailed information about a specific version of a definition. This includes \
                  the full content of the definition at that version, including any keywords, notes, and metadata. \
                  The version ID can be obtained from the version history endpoint."
)]
#[get("/{version_id}/version")]
pub async fn get_version(pool: web::Data<Pool>, version_id: web::Path<i32>) -> impl Responder {
    match service::get_version(&pool, version_id.into_inner()).await {
        Ok(version) => HttpResponse::Ok().json(version),
        Err(e) => match e.downcast_ref::<tokio_postgres::Error>() {
            Some(db_error) if db_error.code().is_some_and(|c| c.code() == "P0002") => {
                HttpResponse::NotFound().finish()
            }
            _ => HttpResponse::InternalServerError().body(format!("Failed to get version: {}", e)),
        },
    }
}

#[utoipa::path(
    post,
    path = "/versions/{version_id}/revert",
    tag = "versions",
    params(
        ("version_id" = i32, Path, description = "Version ID to revert to")
    ),
    responses(
        (status = 200, description = "Definition reverted successfully", body = Version),
        (status = 404, description = "Version not found"),
        (status = 403, description = "User is not the author and lacks revert permissions"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Revert to previous version",
    description = "Reverts a definition to a specific historical version. This creates a new version with the \
                  content from the specified historical version. The revert operation itself is tracked as a new \
                  version in the history. Only users with edit permissions can perform this operation."
)]
#[post("/{version_id}/revert")]
pub async fn revert_to_version(
    pool: web::Data<Pool>,
    version_id: web::Path<i32>,
    user: crate::auth::Claims,
    perm_cache: web::Data<PermissionCache>,
) -> impl Responder {
    match service::revert_to_version(
        &pool,
        version_id.into_inner(),
        user.sub,
        &user.role.to_string(),
        perm_cache.get_ref(),
    )
    .await
    {
        Ok(new_version) => HttpResponse::Ok().json(new_version),
        Err(e) => match e.downcast_ref::<tokio_postgres::Error>() {
            Some(db_error) if db_error.code().is_some_and(|c| c.code() == "P0002") => {
                HttpResponse::NotFound().finish()
            }
            _ => HttpResponse::InternalServerError().body(format!("Failed to revert: {}", e)),
        },
    }
}

#[utoipa::path(
    get,
    path = "/versions/diff",
    tag = "versions",
    params(
        ("from_version" = i32, Query, description = "Original version ID"),
        ("to_version" = i32, Query, description = "Target version ID to compare against")
    ),
    responses(
        (status = 200, description = "Diff between versions retrieved successfully", body = VersionDiff),
        (status = 404, description = "One or both versions not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Compare two versions",
    description = "Generates a detailed comparison between two versions of a definition. The diff shows \
                  changes in the definition text, notes, keywords, and other fields. Changes are classified \
                  as additions, removals, or modifications. This helps track how a definition has evolved \
                  between any two points in its history."
)]
#[get("/diff")]
pub async fn get_version_diff(
    pool: web::Data<Pool>,
    query: web::Query<GetDiffQuery>,
) -> impl Responder {
    match service::get_diff_with_transaction(&pool, query.from_version, query.to_version).await {
        Ok(diff) => HttpResponse::Ok().json(diff),
        Err(e) => match e.downcast_ref::<tokio_postgres::Error>() {
            Some(db_error) if db_error.code().is_some_and(|c| c.code() == "P0002") => {
                HttpResponse::NotFound().body("One or both versions not found")
            }
            _ => HttpResponse::InternalServerError().body(format!("Failed to get diff: {}", e)),
        },
    }
}
