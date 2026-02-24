use actix_web::http::header::{ContentDisposition, ContentType, DispositionType};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use actix_web_grants::protect;
use chrono::Utc;
use deadpool_postgres::Pool;
use serde_json::json;

use super::dto::ClientIdGroup;
use super::{BulkImportRequest, SearchDefinitionsQuery, UserVoteResponse};
use crate::auth::Claims;
// Removed unused Permission import
use crate::jbovlaste::broadcast::Broadcaster;
use crate::jbovlaste::dto::{ListDefinitionsQuery, NonLojbanDefinitionsQuery};
use crate::jbovlaste::service::validate_image;
use crate::jbovlaste::{
    service, AddDefinitionRequest, AddValsiResponse, BulkImportParams, BulkVoteRequest,
    BulkVoteResponse, DefinitionDetail, DefinitionListResponse, DefinitionTranslation, ExportPairsQuery,
    GetImageDefinitionQuery, ImageUploadRequest, LinkDefinitionsRequest, RecentChangesQuery,
    RecentChangesResponse, SearchDefinitionsParams, UpdateDefinitionRequest,
    UpdateDefinitionResponse, ValsiDefinitionsQuery, ValsiDetail, ValsiTypeListResponse,
    VoteRequest, VoteResponse,
};
use crate::language::{validate_mathjax_fields, MathJaxValidationOptions};
use crate::middleware::cache::{generate_search_cache_key, RedisCache};
use camxes_rs::peg::grammar::Peg;
use std::collections::HashMap;
use std::sync::Arc;

#[utoipa::path(
    get,
    tag = "jbovlaste",
    path = "/jbovlaste/semantic-search",
    params(
        ("query" = SearchDefinitionsQuery, Query, description = "Semantic search parameters")
    ),
    responses(
        (status = 200, description = "List of definitions sorted by semantic similarity", body = DefinitionListResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Semantic search definitions",
    description = "Search for definitions using semantic similarity. Returns paginated results sorted by cosine distance."
)]
#[get("/semantic-search")]
pub async fn semantic_search(
    pool: web::Data<Pool>,
    redis_cache: web::Data<RedisCache>,
    query: web::Query<SearchDefinitionsQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    // Parse languages
    let languages = query.languages.as_ref().and_then(|langs| {
        let parsed: Result<Vec<i32>, _> = langs
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::parse::<i32>)
            .collect();
        parsed.ok()
    });

    let cache_key = crate::middleware::cache::generate_semantic_search_cache_key(&query);

    let processed_text = query.search.as_deref().unwrap_or("").trim().to_string();

    // Generate embedding in-process via fastembed (AllMiniLML6V2, mean pooling, L2-normalised)
    let embedding = match crate::embeddings::get_embedding(&processed_text).await {
        Ok(emb) => emb,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to generate embedding: {}", e)
            }));
        }
    };

    match redis_cache
        .get_or_set(
            &cache_key,
            || async {
                let params = SearchDefinitionsParams {
                    page,
                    per_page,
                    search_term: processed_text.clone(),
                    include_comments: false,
                    sort_by: "similarity".to_string(),
                    sort_order: "asc".to_string(),
                    languages: languages.clone(),
                    selmaho: query.selmaho.clone(),
                    username: query.username.clone(),
                    word_type: query.word_type,
                    source_langid: query.source_langid,
                };

                service::semantic_search(&pool, params, embedding).await
            },
            None, // Use default TTL
        )
        .await
    {
        Ok(response) => HttpResponse::Ok().json(DefinitionListResponse {
            definitions: response.definitions,
            total: response.total,
            page,
            per_page,
            decomposition: vec![],
        }),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[utoipa::path(
    get,
    tag = "jbovlaste",
    path = "/jbovlaste/definitions/list",
    params(
        ("query" = ListDefinitionsQuery, Query, description = "Listing and filtering parameters")
    ),
    responses(
        (status = 200, description = "List of definitions", body = DefinitionListResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "List all definitions",
    description = "Retrieves a paginated list of all definitions with filtering and sorting options."
)]
#[get("/definitions/list")]
pub async fn list_definitions(
    pool: web::Data<Pool>,
    query: web::Query<ListDefinitionsQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    match service::list_definitions(&pool, &query, claims.map(|c| c.sub)).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[utoipa::path(
    get,
    tag = "jbovlaste",
    path = "/jbovlaste/non-lojban-definitions",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("search" = Option<String>, Query, description = "Search term for word or definition"),
        ("sort_by" = Option<String>, Query, description = "Sort field (word, time, score)"),
        ("sort_order" = Option<String>, Query, description = "Sort order (asc/desc)"),
        ("languages" = Option<String>, Query, description = "Comma-separated list of definition language IDs"),
        ("username" = Option<String>, Query, description = "Filter by definition author username"),
        ("source_langid" = Option<i32>, Query, description = "Filter by valsi source language ID")
    ),
    responses(
        (status = 200, description = "List of non-Lojban definitions", body = DefinitionListResponse),
        (status = 500, description = "Internal server error")
    ),
    summary = "List non-Lojban definitions",
    description = "Retrieves definitions whose associated valsi are not Lojban (source_langid != 1). Supports pagination and filtering by source language ID."
)]
#[get("/non-lojban-definitions")]
pub async fn list_non_lojban_definitions(
    pool: web::Data<Pool>,
    query: web::Query<NonLojbanDefinitionsQuery>,
) -> impl Responder {
    match service::list_non_lojban_definitions(&pool, query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[utoipa::path(
    get,
    tag = "jbovlaste",
    path = "/jbovlaste/definitions",
    params(
        ("query" = SearchDefinitionsQuery, Query, description = "Search and pagination parameters")
    ),
    responses(
        (status = 200, description = "List of definitions", body = DefinitionListResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Search definitions",
    description = "Search for definitions across the dictionary with filtering and sorting options. \
                  Returns paginated results including definition details, scores, and optional comment counts."
)]
#[get("/definitions")]
pub async fn search_definitions(
    pool: web::Data<Pool>,
    redis_cache: web::Data<RedisCache>,
    query: web::Query<SearchDefinitionsQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let search_term = query.search.as_deref().unwrap_or("").trim();
    let include_comments = query.include_comments.unwrap_or(false);

    // Use fast search if explicitly requested via 'fast' parameter, or for non-logged-in users
    let use_fast_search = query.fast.unwrap_or(false) || claims.is_none();

    let cache_key = generate_search_cache_key(&query);

    match redis_cache
        .get_or_set(
            &cache_key,
            || async {
                let (sort_by, sort_order) =
                    match (query.sort_by.as_deref(), query.sort_order.as_deref()) {
                        (Some(sort), Some(order)) => (sort.to_string(), order.to_string()),
                        (Some(sort), None) => (sort.to_string(), "asc".to_string()),
                        _ => ("word".to_string(), "asc".to_string()),
                    };

                let languages = query.languages.as_ref().and_then(|langs| {
                    let parsed: Result<Vec<i32>, _> = langs
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(str::parse::<i32>)
                        .collect();
                    parsed.ok()
                });

                let params = SearchDefinitionsParams {
                    page,
                    per_page,
                    search_term: search_term.to_string(),
                    include_comments,
                    sort_by,
                    sort_order,
                    languages,
                    selmaho: query.selmaho.clone(),
                    username: query.username.clone(),
                    word_type: query.word_type,
                    source_langid: query.source_langid,
                };

                if use_fast_search {
                    service::fast_search_definitions(&pool, params).await
                } else {
                    service::search_definitions(&pool, params, &redis_cache).await
                }
            },
            None,
        )
        .await
    {
        Ok(response) => HttpResponse::Ok().json(DefinitionListResponse {
            definitions: response.definitions,
            decomposition: response.decomposition,
            total: response.total,
            page,
            per_page,
        }),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[utoipa::path(
    get,
    tag = "jbovlaste",
    path = "/jbovlaste/valsi/{id_or_word}",
    summary = "Get valsi details",
    description = "Retrieves detailed information about a specific valsi entry, including its definitions, \
                  etymologies, and metadata. Returns a 404 if the valsi is not found.",
    params(
        ("id_or_word" = String, Path, description = "Valsi ID or word"),
    ),
    responses(
        (status = 200, description = "Valsi details", body = ValsiDetail),
        (status = 404, description = "Valsi not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/valsi/{id_or_word}")]
pub async fn get_entry_details(
    pool: web::Data<Pool>,
    id_or_word: web::Path<String>,
) -> impl Responder {
    match service::get_entry_details(&pool, &id_or_word.into_inner()).await {
        Ok(valsi_detail) => HttpResponse::Ok().json(json!({
            "valsi": valsi_detail
        })),
        Err(e) => {
            if e.to_string().contains("Valsi not found") {
                HttpResponse::NotFound().json(json!({
                    "error": "Valsi not found"
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "error": format!("Database error: {}", e)
                }))
            }
        }
    }
}

#[utoipa::path(
    get,
    tag = "jbovlaste",
    path = "/jbovlaste/valsi/{id_or_word}/definitions",
    params(
        ("id_or_word" = String, Path, description = "Valsi ID or word"),
        ("langid" = Option<i32>, Query, description = "Preferred language ID"),
        ("username" = Option<String>, Query, description = "Preferred username")
    ),
    responses(
        (status = 200, description = "List of definitions", body = Vec<DefinitionDetail>),
        (status = 404, description = "Valsi not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get definitions for valsi",
    description = "Retrieve all definitions for a specific valsi (Lojban word), ordered by preferred \
                  language and username if specified. Each definition includes full details, scores, \
                  and keyword mappings."
)]
#[get("/valsi/{id_or_word}/definitions")]
pub async fn get_definitions_by_entry(
    pool: web::Data<Pool>,
    redis_cache: web::Data<RedisCache>,
    id_or_word: web::Path<String>,
    query: web::Query<ValsiDefinitionsQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    match service::get_definitions_by_entry(
        &pool,
        &id_or_word.into_inner(),
        claims.map(|c| c.sub),
        query.langid,
        query.username.clone(),
        &redis_cache,
    )
    .await
    {
        Ok(definitions) => {
            if definitions.is_empty() {
                HttpResponse::NotFound().body("Valsi not found")
            } else {
                HttpResponse::Ok().json(definitions)
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

#[utoipa::path(
    post,
    tag = "jbovlaste",
    path = "/jbovlaste/valsi",
    summary = "Add new definition",
    description = "Creates a new definition. The word type is automatically \
                  determined based on Lojban morphology rules. Includes validation of the word structure.",
    request_body = AddDefinitionRequest,
    responses(
        (status = 200, description = "Valsi added successfully", body = AddValsiResponse),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Insufficient permissions"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/valsi")]
#[protect(any("create_definition"))]
pub async fn add_definition(
    pool: web::Data<Pool>,
    claims: Claims,
    parsers: web::Data<Arc<HashMap<i32, Peg>>>,
    redis_cache: web::Data<RedisCache>,
    request: web::Json<AddDefinitionRequest>,
) -> impl Responder {
    let options = MathJaxValidationOptions { use_tectonic: true };
    let definition = crate::utils::remove_html_tags(&request.definition);
    let notes = request
        .notes
        .as_ref()
        .map(|n| crate::utils::remove_html_tags(n));
    let etymology = request
        .etymology
        .as_ref()
        .map(|e| crate::utils::remove_html_tags(e));
    let fields: Vec<(&str, &str)> = vec![
        ("definition", definition.as_str()),
        ("notes", notes.as_deref().unwrap_or("")),
        ("etymology", etymology.as_deref().unwrap_or("")),
    ]
    .into_iter()
    .filter(|(_, t)| !t.trim().is_empty())
    .collect::<Vec<_>>();
    if let Err((field_name, e)) = validate_mathjax_fields(&fields, &options).await {
        return HttpResponse::BadRequest().json(AddValsiResponse {
            success: false,
            word_type: String::new(),
            definition_id: 0,
            error: Some(format!("Invalid LaTeX/MathJax in {}: {}", field_name, e)),
        });
    }
    if let Some(image) = &request.image {
        if let Err(e) = validate_image(image) {
            return HttpResponse::BadRequest().json(AddValsiResponse {
                success: false,
                word_type: String::new(),
                definition_id: 0,
                error: Some(e),
            });
        }
    }
    // Pass the parser map to the service
    match service::add_definition(
        &pool,
        &claims,
        parsers.get_ref().clone(),
        &request,
        &redis_cache,
        true,
    )
    .await
    {
        Ok((word_type, definition_id)) => HttpResponse::Ok().json(AddValsiResponse {
            success: true,
            word_type,
            definition_id,
            error: None,
        }),
        Err(e) => HttpResponse::InternalServerError().json(AddValsiResponse {
            success: false,
            word_type: String::new(),
            definition_id: 0,
            error: Some(e.to_string()),
        }),
    }
}

#[utoipa::path(
    get,
    tag = "jbovlaste",
    path = "/jbovlaste/definition",
    summary = "Get definition details",
    description = "Retrieves detailed information about a specific definition, including its gloss words, \
                  place structure, and any associated notes or examples.",
    request_body = AddDefinitionRequest,
    responses(
        (status = 200, description = "Valsi added successfully", body = AddValsiResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/definition/{id}")]
pub async fn get_definition(
    pool: web::Data<Pool>,
    redis_cache: web::Data<RedisCache>,
    id: web::Path<i32>,
    claims: Option<Claims>,
) -> impl Responder {
    let definition_id = id.into_inner();

    match service::get_definition(&pool, definition_id, claims.map(|c| c.sub), &redis_cache).await {
        Ok(Some(definition)) => HttpResponse::Ok().json(definition),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

#[utoipa::path(
    get,
    path = "/jbovlaste/definition_image/{definition_id}/image",
    tag = "jbovlaste",
    params(
        ("definition_id" = i32, Path, description = "Definition ID"),
        ("image_id" = Option<i32>, Query, description = "Optional image ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Image data", content_type = "image/*"),
        (status = 404, description = "Image not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/definition_image/{definition_id}/image")]
pub async fn get_definition_image(
    pool: web::Data<Pool>,
    definition_id: web::Path<i32>,
    query: web::Query<GetImageDefinitionQuery>,
) -> impl Responder {
    match service::get_definition_image(&pool, definition_id.into_inner(), query.into_inner()).await
    {
        Ok(Some((image_data, mime_type))) => {
            let cd = ContentDisposition {
                disposition: DispositionType::Inline,
                parameters: vec![], // Add parameters if needed
            };

            HttpResponse::Ok()
                .content_type(mime_type)
                .insert_header(cd)
                .body(image_data)
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[utoipa::path(
    put,
    tag = "jbovlaste",
    path = "/jbovlaste/valsi/{id}",
    summary = "Update definition",
    description = "Updates an existing definition with new content. Includes validation of any MathJax/LaTeX \
                  content and maintains version history of the changes.",
    params(
        ("id" = i32, Path, description = "Definition ID")
    ),
    request_body = UpdateDefinitionRequest,
    responses(
        (status = 200, description = "Definition updated successfully", body = UpdateDefinitionResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[put("/valsi/{id}")]
#[protect(any("edit_definition"))]
pub async fn update_definition(
    pool: web::Data<Pool>,
    claims: Claims,
    redis_cache: web::Data<RedisCache>,
    req: web::Json<UpdateDefinitionRequest>,
    id: web::Path<i32>,
) -> impl Responder {
    let definition_id = id.into_inner();

    let options = MathJaxValidationOptions { use_tectonic: true };

    let definition = crate::utils::remove_html_tags(&req.definition);
    let notes = req
        .notes
        .as_ref()
        .map(|n| crate::utils::remove_html_tags(n));
    let etymology = req
        .etymology
        .as_ref()
        .map(|e| crate::utils::remove_html_tags(e));

    let fields: Vec<(&str, &str)> = vec![
        ("definition", definition.as_str()),
        ("notes", notes.as_deref().unwrap_or("")),
        ("etymology", etymology.as_deref().unwrap_or("")),
    ]
    .into_iter()
    .filter(|(_, t)| !t.trim().is_empty())
    .collect::<Vec<_>>();

    if let Err((field_name, e)) = validate_mathjax_fields(&fields, &options).await {
        return HttpResponse::BadRequest().json(UpdateDefinitionResponse {
            success: false,
            error: Some(format!("Invalid LaTeX/MathJax in {}: {}", field_name, e)),
        });
    }

    if let Some(image) = &req.image {
        if let Err(e) = validate_image(image) {
            return HttpResponse::BadRequest().json(UpdateDefinitionResponse {
                success: false,
                error: Some(e),
            });
        }
    }

    match service::update_definition(&pool, definition_id, claims.sub, &req, &redis_cache).await {
        Ok(_) => HttpResponse::Ok().json(UpdateDefinitionResponse {
            success: true,
            error: None,
        }),
        Err(e) => HttpResponse::InternalServerError().json(UpdateDefinitionResponse {
            success: false,
            error: Some(format!("Failed to update definition: {}", e)),
        }),
    }
}

#[utoipa::path(
    get,
    path = "/jbovlaste/vote/{definition_id}",
    tag = "jbovlaste",
    params(
        ("definition_id" = i32, Path, description = "Definition ID")
    ),
    responses(
        (status = 200, description = "User's current vote", body = UserVoteResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Definition not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get user's vote",
    description = "Retrieve the current user's vote (upvote/downvote) for a specific definition. \
                  Returns null if the user hasn't voted."

)]
#[get("/vote/{definition_id}")]
pub async fn get_vote(
    pool: web::Data<Pool>,
    definition_id: web::Path<i32>,
    claims: Claims,
) -> impl Responder {
    match service::get_user_vote(&pool, claims.sub, definition_id.into_inner()).await {
        Ok(vote) => HttpResponse::Ok().json(json!({
            "vote": vote,
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get vote: {}", e)
        })),
    }
}

#[utoipa::path(
    post,
    tag = "jbovlaste",
    path = "/jbovlaste/vote",
    summary = "Vote on definition",
    description = "Records a user's vote (upvote or downvote) for a specific definition. Each user can only \
                  have one active vote per definition, and voting affects the definition's overall score.",
    request_body = VoteRequest,
    responses(
        (status = 200, description = "Vote recorded successfully", body = VoteResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/vote")]
#[protect("vote_definition")]
pub async fn update_vote(
    pool: web::Data<Pool>,
    claims: Claims,
    redis_cache: web::Data<RedisCache>,
    req: web::Json<VoteRequest>,
) -> impl Responder {
    match service::update_vote(
        &pool,
        &redis_cache,
        claims.sub,
        req.definition_id,
        req.downvote.unwrap_or(false),
    )
    .await
    {
        Ok((success, message, word, score)) => HttpResponse::Ok().json(VoteResponse {
            success,
            message,
            word,
            score,
        }),
        Err(e) => {
            // Determine error type and return appropriate response
            match e.to_string() {
                e if e.contains("Invalid definition ID") => {
                    HttpResponse::BadRequest().json(VoteResponse {
                        success: false,
                        message: format!("Invalid definition ID: {}", e),
                        word: None,
                        score: None,
                    })
                }
                _ => HttpResponse::InternalServerError().json(VoteResponse {
                    success: false,
                    message: format!("Server error: {}", e),
                    word: None,
                    score: None,
                }),
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/jbovlaste/changes",
    tag = "jbovlaste",
    summary = "Get recent changes to the dictionary",
    description = "Retrieves a list of recent changes to the dictionary, including modifications to definitions, \
                  comments, and valsi entries. Changes are ordered by time descending. \
                  Excludes automated changes made by the system account.",
    params(
        ("days" = Option<i32>, Query, description = "Number of days to look back")
    ),
    responses(
        (status = 200, description = "Recent changes retrieved successfully", body = RecentChangesResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/changes")]
pub async fn get_recent_changes(
    pool: web::Data<Pool>,
    redis_cache: web::Data<RedisCache>,
    query: web::Query<RecentChangesQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    let days = query.days.unwrap_or(7);
    let user_id = claims.map(|c| c.sub);

    match service::get_recent_changes(&pool, days, &redis_cache, user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Error retrieving changes: {}", e))
        }
    }
}

#[utoipa::path(
    post,
    path = "/jbovlaste/bulk-import/cancel/{job_id}",
    tag = "jbovlaste",
    params(
        ("job_id" = String, Path, description = "Import Job ID")
    ),
    responses(
        (status = 200, description = "Import cancelled successfully"),
        (status = 404, description = "Job not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = ["ADMIN"])
    ),
    summary = "Cancel bulk import",
    description = "Cancels an ongoing bulk import operation using the client ID"
)]
#[post("/bulk-import/cancel/{client_id}")]
#[protect("bulk_import")]
pub async fn cancel_bulk_import(
    broadcaster: web::Data<Broadcaster>,
    client_id: web::Path<String>,
) -> impl Responder {
    match broadcaster.cancel_import(&client_id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Import cancellation requested for client ID"
        })),
        Err(e) => HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": e
        })),
    }
}

#[utoipa::path(
    post,
    path = "/jbovlaste/votes",
    tag = "jbovlaste",
    request_body = BulkVoteRequest,
    responses(
        (status = 200, description = "User votes retrieved", body = BulkVoteResponse),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get bulk user votes",
    description = "Retrieve the current user's votes for multiple definitions in one request"
)]
#[post("/votes")]
pub async fn get_bulk_votes(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<BulkVoteRequest>,
) -> impl Responder {
    match service::get_bulk_user_votes(&pool, claims.sub, &req.definition_ids).await {
        Ok(votes) => HttpResponse::Ok().json(BulkVoteResponse { votes }),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get votes: {}", e)
        })),
    }
}

#[utoipa::path(
    post,
    path = "/jbovlaste/bulk-import/delete/{client_id}",
    tag = "jbovlaste",
    params(
        ("client_id" = String, Path, description = "Client ID from bulk import metadata")
    ),
    responses(
        (status = 200, description = "Bulk delete results", body = serde_json::Value, example = json!({
            "deleted": [1, 2, 3],
            "skipped": [4, 5]
        })),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = ["ADMIN"])
    ),
    summary = "Delete bulk imported definitions",
    description = "Delete all definitions from a bulk import by client ID. Skips definitions with comments."
)]
#[post("/bulk-import/delete/{client_id}")]
#[protect("bulk_import")]
pub async fn delete_bulk_definitions(
    pool: web::Data<Pool>,
    client_id: web::Path<String>,
) -> impl Responder {
    match service::delete_bulk_definitions(&pool, &client_id.into_inner()).await {
        Ok((deleted, skipped)) => HttpResponse::Ok().json(json!({
            "deleted": deleted,
            "skipped": skipped
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to delete definitions: {}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/jbovlaste/bulk-import/active",
    tag = "jbovlaste",
    responses(
        (status = 200, description = "List of active import jobs", body = Vec<String>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = ["ADMIN"])
    ),
    summary = "List active imports",
    description = "Returns Client IDs of all active bulk import operations"
)]
#[get("/bulk-import/active")]
#[protect("bulk_import")]
pub async fn list_active_imports(broadcaster: web::Data<Broadcaster>) -> impl Responder {
    let client_ids = broadcaster.list_active_imports().await;
    HttpResponse::Ok().json(client_ids)
}

#[utoipa::path(
    post,
    path = "/jbovlaste/bulk-import",
    tag = "jbovlaste",
    request_body = BulkImportRequest,
    responses(
        (status = 200, description = "SSE stream of import progress", content_type = "text/event-stream"),
        (status = 400, description = "Invalid CSV format"),
        (status = 403, description = "Admin access required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = ["ADMIN"])
    ),
    summary = "Bulk import gismu definitions with progress updates",
    description = "Admin endpoint for bulk importing gismu definitions from CSV with real-time progress updates via SSE. CSV format: gismu,definition,notes,glosswords"
)]
#[post("/bulk-import")]
#[protect("bulk_import")]
pub async fn bulk_import_definitions(
    pool: web::Data<Pool>,
    claims: Claims,
    parsers: web::Data<Arc<HashMap<i32, Peg>>>,
    redis_cache: web::Data<RedisCache>,
    broadcaster: web::Data<Broadcaster>,
    request: web::Json<BulkImportRequest>,
) -> impl Responder {
    // Get client_id, SSE stream, and cancellation receiver from the broadcaster
    let (client_id, sse, cancel_rx) = broadcaster.new_client().await;
    let client_id_clone = client_id.clone(); // Clone client_id for the spawned task

    // Send client ID as first event (used for cancellation and deletion)
    if let Ok(json_str) = serde_json::to_string(&json!({
        "type": "client_id",
        "client_id": client_id
    })) {
        // Broadcast using the obtained client_id
        if let Err(e) = broadcaster.broadcast(&client_id, &json_str).await {
            log::error!(
                "Failed to broadcast client_id event to {}: {}",
                client_id,
                e
            );
            // Consider returning an error response if initial broadcast fails
        }
    } else {
        log::error!("Failed to serialize job_id event JSON");
    }

    // Spawn the import task
    actix_web::rt::spawn(async move {
        let params = BulkImportParams {
            csv_data: &request.csv,
            lang_id: request.lang_id,
            client_id: client_id_clone.clone(), // Use the cloned client_id
            import_time: Utc::now(),
        };

        let result = service::bulk_import_definitions(
            &pool,
            &claims,
            parsers.get_ref().clone(), // Pass parser map
            params,
            &broadcaster, // Pass broadcaster reference
            &redis_cache,
            cancel_rx,
        )
        .await;

        // Send final status based on the result from the service
        match result {
            Ok((success_count, error_count)) => {
                let total_processed = success_count + error_count; // Total attempted/processed
                let final_payload = json!({
                    "type": "complete",
                    "success": error_count == 0, // Success if no errors
                    "client_id": &client_id_clone,
                    "success_count": success_count,
                    "error_count": error_count,
                    "total_processed": total_processed,
                    "message": format!("Import finished. Success: {}, Errors: {}", success_count, error_count)
                });
                if let Ok(json_str) = serde_json::to_string(&final_payload) {
                    log::info!(
                        "Sending 'complete' event to client {}: {}",
                        client_id_clone,
                        json_str
                    );
                    if let Err(e) = broadcaster.broadcast(&client_id_clone, &json_str).await {
                        log::error!(
                            "Failed to broadcast complete event to {}: {}",
                            client_id_clone,
                            e
                        );
                    }
                } else {
                    log::error!("Failed to serialize complete event JSON");
                }
            }
            Err(e) => {
                // Handle errors from the service function itself (e.g., cancellation)
                log::error!("Bulk import service returned an error: {}", e);
                let error_payload = json!({
                    "type": "error", // Use 'error' type for fatal service errors
                    "success": false,
                    "error": format!("Import process failed: {}", e)
                });
                if let Ok(json_str) = serde_json::to_string(&error_payload) {
                    log::info!(
                        "Sending fatal 'error' event to client {}: {}",
                        client_id_clone,
                        json_str
                    );
                    if let Err(broadcast_err) =
                        broadcaster.broadcast(&client_id_clone, &json_str).await
                    {
                        log::error!(
                            "Failed to broadcast fatal error event to {}: {}",
                            client_id_clone,
                            broadcast_err
                        );
                    }
                } else {
                    log::error!(
                        "Failed to serialize fatal error event JSON for error: {}",
                        e
                    );
                }
            }
        }

        // Ensure client removal happens after attempting to send the final message
        log::info!(
            "Removing client {} from broadcaster after processing.",
            client_id_clone
        );
        broadcaster.remove_client(&client_id_clone).await;
    });

    sse
}

#[utoipa::path(
    get,
    path = "/jbovlaste/types",
    tag = "jbovlaste",
    responses(
        (status = 200, description = "List of valsi types", body = ValsiTypeListResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "List valsi types",
    description = "Get all valid valsi (word) types in the dictionary. Types include gismu, \
                  cmavo, lujvo, etc. Used for categorizing and filtering words."
)]
#[get("/types")]
pub async fn list_valsi_types(pool: web::Data<Pool>) -> impl Responder {
    match service::list_valsi_types(&pool).await {
        Ok(types) => HttpResponse::Ok().json(ValsiTypeListResponse { types }),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

#[utoipa::path(
    post,
    path = "/jbovlaste/definition_image/{id}/image",
    tag = "jbovlaste",
    params(
        ("id" = i32, Path, description = "Definition ID")
    ),
    request_body(content = ImageUploadRequest, description = "Image data and metadata", content_type = "application/json"),
    responses(
        (status = 200, description = "Image uploaded successfully"),
        (status = 400, description = "Invalid image data or format"),
        (status = 403, description = "Not authorized to add images"),
        (status = 404, description = "Definition not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = ["USER"])
    ),
    summary = "Upload definition image",
    description = "Adds a new image to a definition. Images are automatically compressed and converted to WebP format."
)]
#[post("/definition_image/{id}/image")]
#[protect("edit_definition")]
pub async fn upload_definition_image(
    pool: web::Data<Pool>,
    id: web::Path<i32>,
    claims: Claims,
    req: web::Json<ImageUploadRequest>,
) -> impl Responder {
    if let Err(e) = validate_image(&req.image) {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": e
        }));
    }

    match service::add_definition_image(
        &pool,
        id.into_inner(),
        claims.sub,
        &req.image,
        req.description.as_deref(),
    )
    .await
    {
        Ok(image_id) => HttpResponse::Ok().json(json!({
            "success": true,
            "image_id": image_id
        })),
        Err(e) => {
            if e.to_string().contains("not authorized") {
                HttpResponse::Forbidden().json(json!({
                    "success": false,
                    "error": e.to_string()
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": format!("Failed to upload image: {}", e)
                }))
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/jbovlaste/sitemap.xml",
    tag = "jbovlaste",
    responses(
        (status = 200, description = "XML sitemap", content_type = "application/xml"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get XML sitemap",
    description = "Generates XML sitemap with all dictionary entries for search engine indexing. Cached for 24 hours and automatically regenerated daily."
)]
#[get("/sitemap.xml")]
pub async fn get_sitemap(
    pool: web::Data<Pool>,
    redis_cache: web::Data<RedisCache>,
) -> impl Responder {
    match service::get_sitemap(&pool, &redis_cache).await {
        Ok(xml) => HttpResponse::Ok()
            .content_type(ContentType::xml())
            .body(xml),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Error generating sitemap: {}", e))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/jbovlaste/definition/{id}",
    tag = "jbovlaste",
    params(
        ("id" = i32, Path, description = "Definition ID")
    ),
    responses(
        (status = 200, description = "Definition deleted successfully"),
        (status = 400, description = "Definition has comments and cannot be deleted"),
        (status = 404, description = "Definition not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = ["ADMIN"])
    ),
    summary = "Delete definition",
    description = "Deletes a definition if it has no comments. Only administrators can delete definitions."
)]
#[delete("/definition/{id}")]
pub async fn delete_definition(
    pool: web::Data<Pool>,
    id: web::Path<i32>,
    claims: Claims,
) -> impl Responder {
    match service::delete_definition(&pool, id.into_inner(), claims.sub).await {
        Ok(result) => {
            if !result.definition_deleted {
                if result.has_remaining_definitions {
                    HttpResponse::BadRequest().json(json!({
                        "success": false,
                        "message": "Definition has comments and cannot be deleted"
                    }))
                } else {
                    HttpResponse::NotFound().json(json!({
                        "success": false,
                        "message": "Definition not found"
                    }))
                }
            } else {
                HttpResponse::Ok().json(json!({
                    "success": true,
                    "message": "Definition deleted successfully",
                    "valsi_deleted": result.valsi_deleted,
                    "has_remaining_definitions": result.has_remaining_definitions
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "message": format!("Failed to delete definition: {}", e)
        })),
    }
}

#[utoipa::path(
    get,
    tag = "jbovlaste",
    path = "/jbovlaste/bulk-import/clients",
    responses(
        (status = 200, description = "List of client IDs and their definition counts", body = Vec<ClientIdGroup>),
        (status = 403, description = "Insufficient permissions"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = ["bulk_import"])
    ),
    summary = "List bulk import client groups",
    description = "Retrieves a list of unique client_ids found in definition metadata where bulk_import is true, along with the count of definitions for each client_id."
)]
#[get("/bulk-import/clients")]
#[protect(any("bulk_import"))]
pub async fn list_bulk_import_clients_handler(
    pool: web::Data<Pool>,
    _claims: Claims, // Claims needed for protect macro, but not used directly here
) -> impl Responder {
    match service::list_bulk_import_client_groups(&pool).await {
        Ok(groups) => HttpResponse::Ok().json(groups),
        Err(e) => {
            log::error!("Failed to list bulk import client groups: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to retrieve client groups"}))
        }
    }
}

#[utoipa::path(
    get,
    tag = "jbovlaste",
    path = "/jbovlaste/bulk-import/clients/{client_id}/definitions",
    params(
        ("client_id" = String, Path, description = "Client ID from bulk import metadata"),
        ("page" = Option<i64>, Query, description = "Page number for pagination", example = 1),
        ("per_page" = Option<i64>, Query, description = "Number of definitions per page", example = 20)
    ),
    responses(
        (status = 200, description = "Paginated list of definitions for the client ID", body = DefinitionListResponse),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Client ID not found or no definitions associated"), // Assuming service might return empty list for not found
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = ["bulk_import"])
    ),
    summary = "List definitions for a specific bulk import client ID",
    description = "Retrieves a paginated list of definitions associated with a specific client_id from the definition metadata."
)]
#[get("/bulk-import/clients/{client_id}/definitions")]
#[protect(any("bulk_import"))]
pub async fn list_client_definitions_handler(
    pool: web::Data<Pool>,
    path: web::Path<String>,
    query: web::Query<ListDefinitionsQuery>,
    claims: Claims, // Needed for user_id and permission check
) -> impl Responder {
    let client_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let user_id = claims.sub; // Get user ID from claims

    match service::list_definitions_by_client_id(&pool, &client_id, page, per_page, Some(user_id))
        .await
    {
        Ok(response) => {
            // Consider if an empty list should be 404 or 200 OK with empty data
            // Current service implementation likely returns Ok with empty list, so 200 is appropriate.
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!(
                "Failed to list definitions for client_id {}: {}",
                client_id,
                e
            );
            HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to retrieve definitions"}))
        }
    }
}

#[utoipa::path(
    post,
    path = "/jbovlaste/definitions/link",
    tag = "jbovlaste",
    request_body = LinkDefinitionsRequest,
    responses(
        (status = 200, description = "Definitions linked successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Link two definitions",
    description = "Creates a bidirectional link between two definitions, indicating they are translations of each other."
)]
#[post("/definitions/link")]
#[protect("edit_definition")]
pub async fn link_definitions_handler(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<LinkDefinitionsRequest>,
) -> impl Responder {
    match service::link_definitions(
        &pool,
        req.definition_id,
        req.translation_id,
        claims.sub,
    )
    .await
    {
        Ok(link_id) => HttpResponse::Ok().json(json!({ "success": true, "link_id": link_id })),
        Err(e) => {
            let msg = e.to_string();
            // Return 400 for known validation errors
            if msg.contains("Linking is only allowed for phrases")
                || msg.contains("Cannot link a definition to itself")
                || msg.contains("One or both definitions do not exist")
            {
                HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "error": msg
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": msg
                }))
            }
        }
    }
}

#[utoipa::path(
    delete,
    path = "/jbovlaste/definitions/link/{definition_id}/{translation_id}",
    tag = "jbovlaste",
    params(
        ("definition_id" = i32, Path, description = "Definition ID"),
        ("translation_id" = i32, Path, description = "Translation ID")
    ),
    responses(
        (status = 200, description = "Link removed successfully"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Unlink definitions",
    description = "Removes the bidirectional link between two definitions."
)]
#[delete("/definitions/link/{definition_id}/{translation_id}")]
#[protect("edit_definition")]
pub async fn unlink_definitions_handler(
    pool: web::Data<Pool>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    let (definition_id, translation_id) = path.into_inner();
    match service::unlink_definitions(&pool, definition_id, translation_id).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/jbovlaste/definitions/{id}/translations",
    tag = "jbovlaste",
    params(
        ("id" = i32, Path, description = "Definition ID")
    ),
    responses(
        (status = 200, description = "List of translations", body = Vec<DefinitionTranslation>),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get definition translations",
    description = "Retrieves all definitions linked as translations to the specified definition."
)]
#[get("/definitions/{id}/translations")]
pub async fn get_translations_handler(
    pool: web::Data<Pool>,
    id: web::Path<i32>,
) -> impl Responder {
    match service::get_definition_translations(&pool, id.into_inner()).await {
        Ok(translations) => HttpResponse::Ok().json(translations),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[utoipa::path(
    get,
    path = "/jbovlaste/definitions/export-pairs",
    tag = "jbovlaste",
    params(
        ("from_lang" = i32, Query, description = "Source language ID"),
        ("to_lang" = i32, Query, description = "Target language ID")
    ),
    responses(
        (status = 200, description = "TSV export", content_type = "text/tab-separated-values"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Export linked pairs",
    description = "Exports all linked definition pairs between two languages as a TSV file."
)]
#[get("/definitions/export-pairs")]
pub async fn export_pairs_handler(
    pool: web::Data<Pool>,
    query: web::Query<ExportPairsQuery>,
) -> impl Responder {
    match service::export_linked_pairs(&pool, query.from_lang, query.to_lang).await {
        Ok(tsv_content) => {
            let cd = ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![
                    actix_web::http::header::DispositionParam::Filename(
                        format!("pairs_{}_{}.tsv", query.from_lang, query.to_lang)
                    )
                ],
            };

            HttpResponse::Ok()
                .content_type("text/tab-separated-values")
                .insert_header(cd)
                .body(tsv_content)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[utoipa::path(
    get,
    path = "/jbovlaste/definition-links/{id}",
    tag = "jbovlaste",
    params(
        ("id" = i32, Path, description = "Link ID")
    ),
    responses(
        (status = 200, description = "Definition link details"),
        (status = 404, description = "Link not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get definition link details",
    description = "Retrieves details about a specific definition link, including its definitions and valsi words."
)]
#[get("/definition-links/{id}")]
pub async fn get_definition_link_handler(
    pool: web::Data<Pool>,
    id: web::Path<i32>,
) -> impl Responder {
    match service::get_definition_link(&pool, id.into_inner()).await {
        Ok(Some(link)) => HttpResponse::Ok().json(link),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
