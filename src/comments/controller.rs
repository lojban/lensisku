use actix_web::{delete, get, post, web, HttpResponse, Responder};
use actix_web_grants::protect;
use deadpool_postgres::Pool;
use serde_json::json;

use crate::middleware::cache::RedisCache;

use super::{
    dto::{
        CommentStats, CreateOpinionRequest, NewCommentRequest, OpinionVoteRequest,
        PaginatedCommentsResponse, ReactionSummary, SearchCommentsQuery, ThreadQuery,
        TrendingHashtag,
    },
    models::{Comment, CommentOpinion},
    service,
};
use crate::{
    auth::Claims,
    comments::{
        dto::{
            CommentActionRequest, FreeThreadQuery, NewCommentParams, PaginationQuery,
            ReactionPaginationQuery, ReactionRequest, SearchCommentsParams, ThreadParams,
            TrendingQuery,
        },
        models::TrendingTimespan,
    },
};

#[utoipa::path(
    get,
    path = "/comments/thread",
    tag = "comments",
    params(
        ("valsi_id" = Option<i32>, Query, description = "Valsi ID"),
        ("natlang_word_id" = Option<i32>, Query, description = "Natural Language Word ID"),
        ("definition_id" = Option<i32>, Query, description = "Definition ID"),
        ("comment_id" = Option<i32>, Query, description = "Comment ID"),
        ("target_user_id" = Option<i32>, Query, description = "Target User ID for profile comments")
    ),
    responses(
        (status = 200, description = "Comments thread", body = Vec<Comment>),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get comments for a thread",
    description = "Retrieves comments for a specific thread. \
    The thread can be identified by `thread_id`, `comment_id` (to find its thread), or a context such as `valsi_id` (for a valsi's main thread), `natlang_word_id`, `definition_id`, or `target_user_id` (for a user's profile thread). \
    Combinations of context IDs (e.g., `valsi_id` and `definition_id`) can pinpoint more specific threads. \
    If only `valsi_id` is provided, it attempts to find the unique thread associated solely with that valsi."
)]
#[get("/thread")]
pub async fn get_thread(
    pool: web::Data<Pool>,
    query: web::Query<ThreadQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    if query.thread_id.is_none()
        && query.comment_id.is_none()
        && query.valsi_id.is_none()
        && query.natlang_word_id.is_none()
        && query.definition_id.is_none()
        && query.target_user_id.is_none()
    {
        return HttpResponse::BadRequest()
            .body("Must specify at least one of: thread_id, comment_id, valsi_id, natlang_word_id, definition_id, or target_user_id.");
    }

    let params = ThreadParams {
        thread_id: query.thread_id,
        valsi_id: query.valsi_id,
        natlang_word_id: query.natlang_word_id,
        definition_id: query.definition_id,
        definition_link_id: query.definition_link_id,
        target_user_id: query.target_user_id,
        comment_id: query.comment_id,
        scroll_to: query.scroll_to,
        current_user_id: claims.map(|c| c.sub),
        page: query.page,
        per_page: query.per_page,
    };

    match service::get_thread_comments(&pool, params).await {
        Ok(comments) => HttpResponse::Ok().json(comments),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get comments",
            "details": format!("{:#?}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/comments/search",
    tag = "comments",
    params(
        ("search" = Option<String>, Query, description = "Search term"),
        ("page" = Option<i64>, Query, description = "Page number starting from 1"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("sort_by" = Option<String>, Query, description = "Sort field: time, reactions, replies"),
        ("sort_order" = Option<String>, Query, description = "Sort order: asc, desc"),
        ("username" = Option<String>, Query, description = "Filter by username"),
        ("valsi_id" = Option<i32>, Query, description = "Filter by valsi ID"),
        ("definition_id" = Option<i32>, Query, description = "Filter by definition ID"),
        ("target_user_id" = Option<i32>, Query, description = "Filter by target user ID for profile comments")
    ),
    responses(
        (status = 200, description = "Paginated comments matching search", body = PaginatedCommentsResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Search comments",
    description = "Search comments with filtering and sorting options"
)]
#[get("/search")]
pub async fn search_comments(
    pool: web::Data<Pool>,
    query: web::Query<SearchCommentsQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    let params = SearchCommentsParams {
        page: query.page.unwrap_or(1),
        per_page: query.per_page.unwrap_or(20),
        search_term: query.search.as_deref().unwrap_or("").to_string(),
        sort_by: query.sort_by.as_deref().unwrap_or("time").to_string(),
        sort_order: query.sort_order.as_deref().unwrap_or("desc").to_string(),
        username: query.username.clone(),
        valsi_id: query.valsi_id,
        definition_id: query.definition_id,
        definition_link_id: query.definition_link_id,
        target_user_id: query.target_user_id,
    };

    match service::search_comments(&pool, params, claims.map(|c| c.sub)).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to search comments",
            "details": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/comments/list",
    tag = "comments",
    params(
        ("page" = Option<i64>, Query, description = "Page number starting from 1"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("sort_order" = Option<String>, Query, description = "Sort order: asc, desc (default: desc)")
    ),
    responses(
        (status = 200, description = "Paginated list of all comments", body = PaginatedCommentsResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "List all comments",
    description = "Retrieves a flat, paginated list of all comments, sorted by creation time."
)]
#[get("/list")]
pub async fn list_comments(
    pool: web::Data<Pool>,
    query: web::Query<super::dto::ListCommentsQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let sort_order = query.sort_order.as_deref().unwrap_or("desc");

    match service::list_comments(&pool, page, per_page, sort_order, claims.map(|c| c.sub)).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to list comments",
            "details": e.to_string()
        })),
    }
}

#[utoipa::path(
    post,
    path = "/comments",
    tag = "comments", 
    request_body = NewCommentRequest,
    responses(
        (status = 200, description = "Comment created", body = Comment),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Add new comment or create a new thread",
    description = "Creates a new comment or free-standing thread. For new threads:\n\
                  - Omit all IDs to create a free-standing thread\n\
                  - First comment becomes the thread starter\n\
                  Authentication is required and the comment will be associated with the authenticated user."
)]
#[post("")]
#[protect(any("create_comment"))]
pub async fn add_comment(
    pool: web::Data<Pool>,
    redis_cache: web::Data<RedisCache>,
    claims: Claims,
    request: web::Json<NewCommentRequest>,
) -> impl Responder {
    let content_json = match serde_json::to_string(&request.content) {
        Ok(json) => json,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid content format",
                "details": e.to_string()
            }))
        }
    };

    let params = NewCommentParams {
        pool: pool.get_ref().clone(),
        user_id: claims.sub,
        valsi_id: request.valsi_id,
        natlang_word_id: request.natlang_word_id,
        definition_id: request.definition_id,
        definition_link_id: request.definition_link_id,
        target_user_id: request.target_user_id,
        parent_id: request.parent_id,
        subject: request.subject.clone(),
        content: content_json,
    };

    match service::add_comment(params).await {
        Ok(comment) => {
            if let Err(e) = redis_cache.invalidate_recent_changes().await {
                log::error!("Failed to invalidate recent changes cache after add comment: {}", e);
            }
            HttpResponse::Ok().json(comment)
        },
        Err(e) => {
            let error_message = e.to_string();
            if error_message.contains("exceeds the maximum size") {
                HttpResponse::BadRequest().json(json!({
                    "error": "Comment too large",
                    "details": error_message
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to add comment",
                    "details": format!("{:#?}", e)
                }))
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/comments/bookmarks",
    tag = "comments",
    params(
        ("page" = Option<i64>, Query, description = "Page number starting from 1"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "Bookmarked comments retrieved successfully", body = Vec<Comment>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get bookmarked comments",
    description = "Retrieves paginated comments bookmarked by the current user"
)]
#[get("/bookmarks")]
pub async fn get_bookmarks(
    pool: web::Data<Pool>,
    claims: Claims,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match service::get_bookmarked_comments(&pool, claims.sub, page, per_page).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get bookmarked comments",
            "details": format!("{:#?}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/comments/reactions/my",
    tag = "comments",
    params(
        ("page" = Option<i64>, Query, description = "Page number starting from 1"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "Comments where user has reacted retrieved successfully", body = PaginatedCommentsResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get comments with user reactions",
    description = "Retrieves paginated comments where the authenticated user has added at least one reaction"
)]
#[get("/reactions/my")]
pub async fn get_my_reactions(
    pool: web::Data<Pool>,
    claims: Claims,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match service::get_my_reactions(&pool, claims.sub, page, per_page).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get reaction comments",
            "details": format!("{:#?}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/comments/user/{user_id}",
    tag = "comments",
    params(
        ("user_id" = i32, Path, description = "User ID"),
        ("page" = Option<i64>, Query, description = "Page number starting from 1"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "User comments retrieved successfully", body = Vec<Comment>),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get user comments",
    description = "Retrieves paginated comments made by a specific user"
)]
#[get("/user/{user_id}")]
pub async fn get_user_comments(
    pool: web::Data<Pool>,
    user_id: web::Path<i32>,
    query: web::Query<PaginationQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match service::get_user_comments(
        &pool,
        user_id.into_inner(),
        page,
        per_page,
        claims.map(|c| c.sub),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get user comments",
            "details": format!("{:#?}", e)
        })),
    }
}

#[utoipa::path(
    post,
    path = "/comments/bookmark",
    tag = "comments",
    request_body = CommentActionRequest,
    responses(
        (status = 200, description = "Bookmark action completed successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Bookmark or unbookmark a comment",
    description = "Toggles the bookmark status of a comment for the current user"
)]
#[post("/bookmark")]
pub async fn toggle_bookmark(
    pool: web::Data<Pool>,
    claims: Claims,
    request: web::Json<CommentActionRequest>,
) -> impl Responder {
    match service::toggle_bookmark(&pool, request.comment_id, claims.sub, request.action).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": if request.action { "Comment bookmarked" } else { "Comment unbookmarked" }
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to toggle bookmark",
            "details": format!("{:#?}", e)
        })),
    }
}

#[utoipa::path(
    post,
    path = "/comments/opinions",
    tag = "comments",
    request_body = CreateOpinionRequest,
    responses(
        (status = 200, description = "Opinion created successfully", body = CommentOpinion),
        (status = 400, description = "Invalid opinion content"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Create comment opinion",
    description = "Creates a new opinion for a comment. The opinion must be 12 characters or less."
)]
#[post("/opinions")]
pub async fn create_opinion(
    pool: web::Data<Pool>,
    claims: Claims,
    request: web::Json<CreateOpinionRequest>,
) -> impl Responder {
    match service::create_opinion(&pool, claims.sub, &request).await {
        Ok(opinion) => HttpResponse::Ok().json(opinion),
        Err(e) => {
            if e.to_string().contains("Invalid opinion") {
                HttpResponse::BadRequest().json(json!({
                    "error": e.to_string()
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to create opinion",
                    "details": e.to_string()
                }))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/comments/opinions/vote",
    tag = "comments",
    request_body = OpinionVoteRequest,
    responses(
        (status = 200, description = "Vote action completed successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Vote on comment opinion",
    description = "Adds or removes a vote for a comment opinion"
)]
#[post("/opinions/vote")]
pub async fn vote_opinion(
    pool: web::Data<Pool>,
    claims: Claims,
    request: web::Json<OpinionVoteRequest>,
) -> impl Responder {
    match service::set_opinion_vote(&pool, claims.sub, &request).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": if request.vote { "Vote added" } else { "Vote removed" }
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to process vote",
            "details": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/comments/{comment_id}/opinions",
    tag = "comments",
    params(
        ("comment_id" = i32, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Opinions retrieved successfully", body = Vec<CommentOpinion>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get comment opinions",
    description = "Retrieves the top 5 opinions for a comment, ordered by vote count"
)]
#[get("/{comment_id}/opinions")]
pub async fn get_opinions(
    pool: web::Data<Pool>,
    comment_id: web::Path<i32>,
    claims: Option<Claims>,
) -> impl Responder {
    match service::get_comment_opinions(&pool, comment_id.into_inner(), claims.map(|c| c.sub)).await
    {
        Ok(opinions) => HttpResponse::Ok().json(opinions),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get opinions",
            "details": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/comments/trending",
    tag = "comments",
    params(
        ("timespan" = Option<String>, Query, description = "Time window: day, week, month, year, all", example = "week"),
        ("limit" = Option<i32>, Query, description = "Maximum number of comments", example = 10)
    ),
    responses(
        (status = 200, description = "Trending comments retrieved successfully", body = Vec<Comment>),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get trending comments",
    description = "Retrieves the most actively engaged comments within specified timespan, ordered by reactions and bookmarks"
)]
#[get("/trending")]
pub async fn get_trending(
    pool: web::Data<Pool>,
    query: web::Query<TrendingQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    let timespan = match query.timespan.as_deref() {
        Some("day") => TrendingTimespan::LastDay,
        Some("week") => TrendingTimespan::LastWeek,
        Some("month") => TrendingTimespan::LastMonth,
        Some("year") => TrendingTimespan::LastYear,
        Some("all") => TrendingTimespan::AllTime,
        _ => TrendingTimespan::LastWeek,
    };
    let limit = query.limit.unwrap_or(10);

    match service::get_trending_comments(&pool, timespan, claims.map(|c| c.sub), limit).await {
        Ok(comments) => HttpResponse::Ok().json(comments),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get trending comments",
            "details": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/comments/stats/{comment_id}",
    tag = "comments",
    params(
        ("comment_id" = i32, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Comment statistics retrieved successfully", body = CommentStats),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get comment statistics",
    description = "Retrieves detailed statistics for a comment including bookmarks, replies, reactions, and opinions"
)]
#[get("/stats/{comment_id}")]
pub async fn get_comment_stats(
    pool: web::Data<Pool>,
    comment_id: web::Path<i32>,
) -> impl Responder {
    match service::get_comment_stats(&pool, comment_id.into_inner()).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => match e.to_string() {
            s if s.contains("not found") => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get comment statistics",
                "details": e.to_string()
            })),
        },
    }
}

#[utoipa::path(
    get,
    path = "/comments/most-bookmarked",
    tag = "comments",
    params(
        ("page" = Option<i64>, Query, description = "Page number starting from 1"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "Most bookmarked comments retrieved successfully", body = Vec<Comment>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get most bookmarked comments",
    description = "Retrieves paginated comments ordered by number of bookmarks"
)]
#[get("/most-bookmarked")]
pub async fn get_most_bookmarked(
    pool: web::Data<Pool>,
    query: web::Query<PaginationQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match service::get_most_bookmarked_comments(&pool, page, per_page, claims.map(|c| c.sub)).await
    {
        Ok(comments) => HttpResponse::Ok().json(comments),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get most bookmarked comments",
            "details": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/hashtags/trending",
    tag = "comments",
    params(
        ("timespan" = Option<String>, Query, description = "Time window: day, week, month, year, all", example = "week"),
        ("limit" = Option<i32>, Query, description = "Maximum number of hashtags to return", example = 10)
    ),
    responses(
        (status = 200, description = "List of trending hashtags", body = Vec<TrendingHashtag>),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get trending hashtags",
    description = "Returns the most used hashtags in comments within the specified time window"
)]
#[get("/trending")]
pub async fn trending_hashtags(
    pool: web::Data<Pool>,
    query: web::Query<TrendingQuery>,
) -> impl Responder {
    let timespan = match query.timespan.as_deref() {
        Some("day") => TrendingTimespan::LastDay,
        Some("week") => TrendingTimespan::LastWeek,
        Some("month") => TrendingTimespan::LastMonth,
        Some("year") => TrendingTimespan::LastYear,
        Some("all") => TrendingTimespan::AllTime,
        _ => TrendingTimespan::LastWeek,
    };
    let limit = query.limit.unwrap_or(10);

    match service::get_trending_hashtags(&pool, timespan, limit).await {
        Ok(hashtags) => HttpResponse::Ok().json(hashtags),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get trending hashtags",
            "details": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/hashtags/{tag}",
    tag = "comments",
    params(
        ("tag" = String, Path, description = "Hashtag to search for"),
    ),
    responses(
        (status = 200, description = "List of comments with the specified hashtag", body = Vec<Comment>),
        (status = 404, description = "No comments found with this hashtag"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get comments by hashtag",
    description = "Retrieves all comments that contain the specified hashtag"
)]
#[get("/{tag}")]
pub async fn comments_by_hashtag(
    pool: web::Data<Pool>,
    path: web::Path<String>,
    query: web::Query<PaginationQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    match service::get_comments_by_hashtag(
        &pool,
        &path,
        claims.map(|c| c.sub),
        query.page,
        query.per_page,
    )
    .await
    {
        Ok(comments) => HttpResponse::Ok().json(comments),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get comments by hashtag",
            "details": e.to_string()
        })),
    }
}

#[utoipa::path(
    delete,
    path = "/comments/{comment_id}",
    tag = "comments",
    params(
        ("comment_id" = i32, Path, description = "Comment ID to delete")
    ),
    responses(
        (status = 200, description = "Comment deleted successfully"),
        (status = 400, description = "Comment has replies or invalid request"),
        (status = 403, description = "Unauthorized to delete comment"),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Delete a comment",
    description = "Deletes a comment if user is the author and it has no replies. All associated reactions, opinions and bookmarks will also be deleted."
)]
#[delete("/{comment_id}")]
pub async fn delete_comment(
    pool: web::Data<Pool>,
    comment_id: web::Path<i32>,
    claims: Claims,
) -> impl Responder {
    match service::delete_comment(&pool, comment_id.into_inner(), claims.sub).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Comment deleted successfully"
        })),
        Err(e) => {
            let err_msg = e.to_string();
            let mut status = if err_msg.contains("Unauthorized") {
                HttpResponse::Forbidden()
            } else if err_msg.contains("replies") {
                HttpResponse::BadRequest()
            } else if err_msg.contains("not found") {
                HttpResponse::NotFound()
            } else {
                HttpResponse::InternalServerError()
            };

            status.json(json!({
                "error": err_msg,
                "details": err_msg
            }))
        }
    }
}

#[utoipa::path(
    post,
    path = "/comments/reactions",
    tag = "comments",
    request_body = ReactionRequest,
    responses(
        (status = 200, description = "Reaction toggled successfully"),
        (status = 400, description = "Invalid reaction"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Toggle reaction on comment",
    description = "Adds or removes a reaction for the current user on a comment"
)]
#[post("/reactions")]
pub async fn toggle_reaction(
    pool: web::Data<Pool>,
    claims: Claims,
    request: web::Json<ReactionRequest>,
) -> impl Responder {
    match service::toggle_reaction(&pool, request.comment_id, claims.sub, &request.reaction).await {
        Ok(added) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": if added { "Reaction added" } else { "Reaction removed" }
        })),
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("too long") {
                HttpResponse::BadRequest().json(json!({
                    "error": "Invalid reaction",
                    "details": err_msg
                }))
            } else if err_msg.contains("limit reached") {
                HttpResponse::BadRequest().json(json!({
                    "error": "Reaction limit reached",
                    "details": err_msg
                }))
            } else if err_msg.contains("reactions per comment reached") {
                HttpResponse::BadRequest().json(json!({
                    "error": "Maximum reactions per comment reached",
                    "details": err_msg
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to toggle reaction",
                    "details": err_msg
                }))
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/comments/{comment_id}/reactions",
    tag = "comments",
    params(
        ("comment_id" = i32, Path, description = "Comment ID"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("page_size" = Option<i32>, Query, description = "Number of reactions per page")
    ),
    responses(
        (status = 200, description = "Reactions retrieved successfully", body = ReactionSummary),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/{comment_id}/reactions")]
pub async fn get_reactions(
    pool: web::Data<Pool>,
    comment_id: web::Path<i32>,
    query: web::Query<ReactionPaginationQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    match service::get_reactions(
        &pool,
        comment_id.into_inner(),
        claims.map(|c| c.sub),
        query.page,
        query.page_size,
    )
    .await
    {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get reactions",
            "details": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/comments/threads",
    tag = "comments",
    params(
        ("page" = Option<i64>, Query, description = "Page number starting from 1"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("sort_by" = Option<String>, Query, description = "Sort field: time, subject", example = "time"),
        ("sort_order" = Option<String>, Query, description = "Sort order: asc, desc", example = "desc")
    ),
    responses(
        (status = 200, description = "Paginated threads with last comment", body = PaginatedCommentsResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "List threads",
    description = "Lists threads sorted by last comment activity"
)]
#[get("/threads")]
pub async fn list_threads(
    pool: web::Data<Pool>,
    query: web::Query<FreeThreadQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let sort_by = query.sort_by.as_deref().unwrap_or("time").to_string();
    let sort_order = query.sort_order.as_deref().unwrap_or("desc").to_string();

    match service::list_threads(&pool, page, per_page, &sort_by, &sort_order).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get free threads",
            "details": e.to_string()
        })),
    }
}
