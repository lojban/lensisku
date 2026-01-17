use actix_web::{
    delete, get,
    http::header::{ContentDisposition, DispositionType},
    post, web, HttpResponse, Responder,
};
use deadpool_postgres::Pool;

use crate::{auth::Claims, comments::models::Comment};

use super::{dto::*, service};

#[utoipa::path(
    get,
    path = "/users",
    tag = "users",
    summary = "List users",
    description = "Retrieves a paginated list of users, optionally filtered by search term.",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("search" = Option<String>, Query, description = "Search by username or realname"),
        ("sort_by" = Option<String>, Query, description = "Sort field (username, realname, created_at)"),
        ("sort_order" = Option<String>, Query, description = "Sort order (asc/desc)"),
        ("role" = Option<String>, Query, description = "Filter by user role (admin, moderator, editor, user, unconfirmed)")
    ),
    responses(
        (status = 200, description = "List of users", body = UserListResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("")]
pub async fn list_users(pool: web::Data<Pool>, query: web::Query<UserListQuery>) -> impl Responder {
    match service::list_users(&pool, query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

#[utoipa::path(
    get,
    path = "/users/{username}/profile",
    tag = "users",
    params(
        ("username" = String, Path, description = "Username to fetch profile for")
    ),
    responses(
        (status = 200, description = "User profile retrieved successfully", body = PublicUserProfile),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get user public profile",
    description = "Retrieves the public profile information for a given username, including their basic info and activity statistics."
)]
#[get("/{username}/profile")]
pub async fn get_public_profile(
    pool: web::Data<Pool>,
    username: web::Path<String>,
) -> impl Responder {
    match service::get_public_profile(&pool, &username).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(e) => match e.downcast_ref::<tokio_postgres::Error>() {
            Some(db_error) if db_error.code().is_some_and(|c| c.code() == "P0002") => {
                HttpResponse::NotFound().body("User not found")
            }
            _ => {
                HttpResponse::InternalServerError().body(format!("Failed to fetch profile: {}", e))
            }
        },
    }
}

#[utoipa::path(
    get,
    path = "/users/{username}/definitions",
    tag = "users",
    params(
        ("username" = String, Path, description = "Username"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of user's definitions", body = ContributionsResponse<Definition>),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/{username}/definitions")]
pub async fn get_user_definitions(
    pool: web::Data<Pool>,
    username: web::Path<String>,
    query: web::Query<ContributionsQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match service::get_user_definitions(&pool, &username, page, per_page).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    get,
    path = "/users/{username}/comments",
    tag = "users",
    params(
        ("username" = String, Path, description = "Username"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of user's comments", body = ContributionsResponse<Comment>),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/{username}/comments")]
pub async fn get_user_comments(
    pool: web::Data<Pool>,
    username: web::Path<String>,
    query: web::Query<ContributionsQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match service::get_user_comments(&pool, &username, page, per_page).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    get,
    path = "/users/votes",
    tag = "users",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of user's votes", body = ContributionsResponse<Vote>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get authenticated user's votes",
    description = "Retrieves a paginated list of definitions that the authenticated user has voted on, including the vote value and definition details"
)]
#[get("/votes")]
pub async fn get_user_votes(
    pool: web::Data<Pool>,
    claims: Claims,
    query: web::Query<ContributionsQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match service::get_user_votes(&pool, claims.sub, page, per_page).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    post,
    path = "/users/profile-image",
    tag = "users",
    request_body = ProfileImageRequest,
    responses(
        (status = 200, description = "Profile image updated successfully", body = ProfileImageResponse),
        (status = 400, description = "Invalid image data"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
)]
#[post("/profile-image")]
pub async fn update_profile_image(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<ProfileImageRequest>,
) -> impl Responder {
    match service::update_profile_image(&pool, claims.sub, &req).await {
        Ok(_) => HttpResponse::Ok().json(ProfileImageResponse {
            success: true,
            message: "Profile image updated successfully".to_string(),
        }),
        Err(e) => HttpResponse::BadRequest().json(ProfileImageResponse {
            success: false,
            message: e.to_string(),
        }),
    }
}

#[utoipa::path(
    get,
    path = "/users/{username}/profile-image",
    tag = "users",
    params(
        ("username" = String, Path, description = "Username")
    ),
    responses(
        (status = 200, description = "Profile image", content_type = "image/*"),
        (status = 404, description = "Image not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/{username}/profile-image")]
pub async fn get_profile_image(
    pool: web::Data<Pool>,
    username: web::Path<String>,
) -> impl Responder {
    match service::get_profile_image(&pool, &username).await {
        Ok(Some((image_data, mime_type))) => {
            let cd = ContentDisposition {
                disposition: DispositionType::Inline,
                parameters: vec![],
            };

            HttpResponse::Ok()
                .content_type(mime_type)
                .insert_header(cd)
                .body(image_data)
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    delete,
    path = "/users/profile-image",
    tag = "users",
    responses(
        (status = 200, description = "Profile image removed successfully", body = ProfileImageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[delete("/profile-image")]
pub async fn remove_profile_image(pool: web::Data<Pool>, claims: Claims) -> impl Responder {
    match service::remove_profile_image(&pool, claims.sub).await {
        Ok(_) => HttpResponse::Ok().json(ProfileImageResponse {
            success: true,
            message: "Profile image removed successfully".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ProfileImageResponse {
            success: false,
            message: e.to_string(),
        }),
    }
}
