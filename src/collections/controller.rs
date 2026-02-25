use std::collections::HashMap;

use actix_web::{
    delete, get,
    http::header::{ContentDisposition, DispositionType},
    post, put, web, HttpResponse, Responder,
};
use deadpool_postgres::Pool;
use serde_json::json;

use super::{dto::*, service};
use crate::auth::Claims;

#[utoipa::path(
    post,
    path = "/collections",
    tag = "collections",
    request_body = CreateCollectionRequest,
    responses(
        (status = 200, description = "Collection created successfully", body = CollectionResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Create new collection",
    description = "Creates a new collection for the authenticated user. A collection can contain multiple \
                  dictionary definitions and serves as a way to organize related terms. Collections can be \
                  either public or private, and users can add notes to individual items."
)]
#[post("")]
pub async fn create_collection(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<CreateCollectionRequest>,
) -> impl Responder {
    match service::create_collection(&pool, claims.sub, &req).await {
        Ok(collection) => HttpResponse::Ok().json(collection),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to create collection: {}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/collections",
    tag = "collections",
    responses(
        (status = 200, description = "List of user collections", body = CollectionListResponse),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "List user collections",
    description = "Retrieves all collections owned by the authenticated user. Includes basic collection \
                  information such as name, description, and item count, as well as creation and last \
                  update timestamps."
)]
#[get("")]
pub async fn list_collections(pool: web::Data<Pool>, claims: Claims) -> impl Responder {
    match service::list_collections(&pool, claims.sub).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to list collections: {}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/collections/public",
    tag = "collections",
    responses(
        (status = 200, description = "List of public collections", body = CollectionListResponse),
        (status = 500, description = "Internal server error")
    ),
    summary = "List public collections",
    description = "Retrieves all public collections from all users. This endpoint is useful for discovering \
                  shared collections and studying materials created by others in the community. Results are \
                  ordered by last update time."
)]
#[get("/public")]
pub async fn list_public_collections(pool: web::Data<Pool>) -> impl Responder {
    match service::list_public_collections(&pool).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to list public collections: {}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/collections/{id}",
    tag = "collections",
    params(
        ("id" = i32, Path, description = "Collection ID")
    ),
    responses(
        (status = 200, description = "Collection details with items", body = CollectionResponse),
        (status = 404, description = "Collection not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Get collection details",
    description = "Retrieves detailed information about a specific collection, including all its items. \
                  Public collections are accessible to anyone, while private collections are only accessible \
                  to their owners. Items include the original definition along with any user-added notes."
)]
#[get("/{id}")]
pub async fn get_collection(
    pool: web::Data<Pool>,
    claims: Option<Claims>,
    id: web::Path<i32>,
) -> impl Responder {
    match service::get_collection(&pool, id.into_inner(), claims.map(|c| c.sub)).await {
        Ok(collection) => HttpResponse::Ok().json(collection),
        Err(e) => match e.to_string().as_str() {
            "Collection not found" => HttpResponse::NotFound().finish(),
            "Access denied" => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to get collection: {}", e)
            })),
        },
    }
}

#[utoipa::path(
    put,
    path = "/collections/{id}",
    tag = "collections",
    params(
        ("id" = i32, Path, description = "Collection ID")
    ),
    request_body = UpdateCollectionRequest,
    responses(
        (status = 200, description = "Collection updated successfully", body = CollectionResponse),
        (status = 404, description = "Collection not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Update collection",
    description = "Updates the metadata of an existing collection. Only the collection owner can modify \
                  the collection. Allows changing the name, description, and public/private status. \
                  The items within the collection must be modified through separate endpoints."
)]
#[put("/{id}")]
pub async fn update_collection(
    pool: web::Data<Pool>,
    claims: Claims,
    id: web::Path<i32>,
    req: web::Json<UpdateCollectionRequest>,
) -> impl Responder {
    match service::update_collection(&pool, id.into_inner(), claims.sub, &req).await {
        Ok(collection) => HttpResponse::Ok().json(collection),
        Err(e) => match e.to_string().as_str() {
            "Collection not found" => HttpResponse::NotFound().finish(),
            "Access denied" => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to update collection: {}", e)
            })),
        },
    }
}

#[utoipa::path(
    delete,
    path = "/collections/{id}",
    tag = "collections",
    params(
        ("id" = i32, Path, description = "Collection ID")
    ),
    responses(
        (status = 200, description = "Collection deleted successfully"),
        (status = 404, description = "Collection not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Delete collection",
    description = "Permanently deletes a collection and all its items. This action can only be performed \
                  by the collection owner and cannot be undone. All associated items and notes will also \
                  be removed."
)]
#[delete("/{id}")]
pub async fn delete_collection(
    pool: web::Data<Pool>,
    claims: Claims,
    id: web::Path<i32>,
) -> impl Responder {
    match service::delete_collection(&pool, id.into_inner(), claims.sub).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Collection deleted successfully"
        })),
        Err(e) => match e.to_string().as_str() {
            "Collection not found" => HttpResponse::NotFound().finish(),
            "Access denied" => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to delete collection: {}", e)
            })),
        },
    }
}

#[utoipa::path(
    post,
    path = "/collections/{id}/items",
    tag = "collections",
    params(
        ("id" = i32, Path, description = "Collection ID")
    ),
    request_body = AddItemRequest,
    responses(
        (status = 200, description = "Item added successfully", body = CollectionItemResponse),
        (status = 404, description = "Collection not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Add item to collection",
    description = "Adds a new dictionary definition to a collection. Each item consists of a reference to \
                  a dictionary definition and optional user notes. Items can be added to both public and \
                  private collections, but only by the collection owner."
)]
#[post("/{id}/items")]
pub async fn upsert_item(
    pool: web::Data<Pool>,
    claims: Claims,
    id: web::Path<i32>,
    req: web::Json<AddItemRequest>,
) -> impl Responder {
    match service::upsert_item(&pool, id.into_inner(), claims.sub, &req).await {
        Ok(item_response) => HttpResponse::Ok().json(item_response),
        Err(e) => {
            log::error!("Failed to upsert item: {:?}", e);
            match e.to_string().as_str() {
                "Collection not found" => HttpResponse::NotFound().finish(),
                "Access denied" => HttpResponse::Forbidden().finish(),
                "Definition not found" => HttpResponse::BadRequest().json(json!({
                    "error": "Definition not found"
                })),
                _ => HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to add item: {}", e)
                })),
            }
        }
    }
}

#[utoipa::path(
    put,
    path = "/collections/{id}/items/{item_id}/position",
    tag = "collections",
    params(
        ("id" = i32, Path, description = "Collection ID"),
        ("item_id" = i32, Path, description = "Item ID")
    ),
    request_body = UpdateItemPositionRequest,
    responses(
        (status = 200, description = "Item position updated successfully"),
        (status = 404, description = "Collection or item not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Update item position",
    description = "Updates the position of an item within a collection. Other items' positions will be adjusted automatically."
)]
#[put("/{id}/items/{item_id}/position")]
pub async fn update_item_position(
    pool: web::Data<Pool>,
    claims: Claims,
    path: web::Path<(i32, i32)>,
    req: web::Json<UpdateItemPositionRequest>,
) -> impl Responder {
    match service::update_item_position(&pool, path.0, path.1, claims.sub, req.position).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Position updated successfully",
            "collection_id": path.0,
            "item_id": path.1,
            "position": req.position
        })),
        Err(e) => match e.to_string().as_str() {
            "Collection not found" => HttpResponse::NotFound().finish(),
            "Item not found" => HttpResponse::NotFound().finish(),
            "Access denied" => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to update item position: {}", e)
            })),
        },
    }
}

#[utoipa::path(
    delete,
    path = "/collections/{id}/items/{item_id}",
    tag = "collections",
    params(
        ("id" = i32, Path, description = "Collection ID"),
        ("item_id" = i32, Path, description = "Item ID to remove")
    ),
    responses(
        (status = 200, description = "Item removed successfully"),
        (status = 404, description = "Collection or item not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Remove item from collection"
)]
#[delete("/{id}/items/{item_id}")]
pub async fn remove_item(
    pool: web::Data<Pool>,
    claims: Claims,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    let (collection_id, item_id) = path.into_inner();
    // Delete flashcards first, then remove the item
    match service::remove_item(&pool, collection_id, item_id, claims.sub).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Item removed successfully",
            "collection_id": collection_id,
            "item_id": item_id
        })),
        Err(e) => {
            let message = e.to_string();
            match message.as_str() {
                "Collection not found" => HttpResponse::NotFound().finish(),
                "Access denied" => HttpResponse::Forbidden().finish(),
                "Item not found" => HttpResponse::NotFound().finish(),
                _ => HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to remove item: {}", e)
                })),
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/collections/{id}/clone",
    tag = "collections",
    params(
        ("id" = i32, Path, description = "Collection ID to clone")
    ),
    responses(
        (status = 200, description = "Collection cloned successfully", body = CollectionResponse),
        (status = 404, description = "Collection not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Clone collection",
    description = "Creates a new collection as a copy of an existing one. The new collection includes all \
                  items from the source collection but can be modified independently. This is useful for \
                  creating personal copies of public collections or using existing collections as templates."
)]
#[post("/{id}/clone")]
pub async fn clone_collection(
    pool: web::Data<Pool>,
    claims: Claims,
    id: web::Path<i32>,
) -> impl Responder {
    match service::clone_collection(&pool, id.into_inner(), claims.sub).await {
        Ok(collection) => HttpResponse::Ok().json(collection),
        Err(e) => match e.to_string().as_str() {
            "Collection not found" => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to clone collection: {}", e)
            })),
        },
    }
}

#[utoipa::path(
    post,
    path = "/collections/merge",
    tag = "collections",
    request_body = MergeCollectionsRequest,
    responses(
        (status = 200, description = "Collections merged successfully", body = CollectionResponse),
        (status = 404, description = "One or both collections not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Merge collections",
    description = "Combines two collections into one, either by creating a new collection or adding items \
                  from one collection to another. Duplicate items are handled automatically. Both source \
                  collections must be owned by the requesting user. The original collections remain unchanged \
                  unless specified as the target collection."
)]
#[post("/merge")]
pub async fn merge_collections(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<MergeCollectionsRequest>,
) -> impl Responder {
    match service::merge_collections(&pool, claims.sub, &req).await {
        Ok(collection) => HttpResponse::Ok().json(collection),
        Err(e) => match e.to_string().as_str() {
            "Collection not found" => HttpResponse::NotFound().finish(),
            "Access denied" => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to merge collections: {}", e)
            })),
        },
    }
}

#[utoipa::path(
    get,
    path = "/collections/{id}/items",
    tag = "collections",
    params(
        ("id" = i32, Path, description = "Collection ID"),
        ("page" = Option<i64>, Query, description = "Page number (starts from 1)"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("search" = Option<String>, Query, description = "Search term for filtering items")
    ),
    responses(
        (status = 200, description = "List of collection items", body = CollectionItemListResponse),
        (status = 404, description = "Collection not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "List collection items",
    description = "Retrieves a paginated list of items in a collection. Supports search filtering across notes, words, definitions, and definition notes. Public collections are accessible to anyone, while private collections require authentication as the owner."
)]
#[get("/{id}/items")]
pub async fn list_collection_items(
    pool: web::Data<Pool>,
    redis_cache: web::Data<crate::middleware::cache::RedisCache>,
    claims: Option<Claims>,
    id: web::Path<i32>,
    query: web::Query<ListCollectionItemsQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match service::list_collection_items(
        &pool,
        id.into_inner(),
        claims.map(|c| c.sub),
        page,
        per_page,
        query.search.clone(),
        query.item_id.clone(),
        query.exclude_with_flashcards,
        redis_cache.as_ref(),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => match e.to_string().as_str() {
            "Collection not found" => HttpResponse::NotFound().finish(),
            "Access denied" => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to list items: {}", e)
            })),
        },
    }
}

#[utoipa::path(
    put,
    path = "/collections/{id}/items/{item_id}/notes",
    tag = "collections",
    params(
        ("id" = i32, Path, description = "Collection ID"),
        ("item_id" = i32, Path, description = "Item ID")
    ),
    request_body = UpdateItemNotesRequest,
    responses(
        (status = 200, description = "Item notes updated successfully", body = CollectionItemResponse),
        (status = 404, description = "Collection or item not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Update item notes",
    description = "Updates the notes associated with a collection item. Only the collection owner can modify notes."
)]
#[put("/{id}/items/{item_id}/notes")]
pub async fn update_item_notes(
    pool: web::Data<Pool>,
    claims: Claims,
    path: web::Path<(i32, i32)>,
    req: web::Json<UpdateItemNotesRequest>,
) -> impl Responder {
    match service::update_item_notes(&pool, path.0, path.1, claims.sub, &req).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(e) => match e.to_string().as_str() {
            "Collection not found" => HttpResponse::NotFound().finish(),
            "Item not found" => HttpResponse::NotFound().finish(),
            "Access denied" => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to update item notes: {}", e)
            })),
        },
    }
}

#[utoipa::path(
    get,
    path = "/collections/{collection_id}/items/{item_id}/image/{side}",
    tag = "collections",
    params(
        ("collection_id" = i32, Path, description = "Collection ID"),
        ("item_id" = i32, Path, description = "Item ID"),
        ("side" = String, Path, description = "Image side (front/back)")
    ),
    responses(
        (status = 200, description = "Image data", content_type = "image/*"),
        (status = 404, description = "Image not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/{collection_id}/items/{item_id}/image/{side}")]
pub async fn get_item_image(
    pool: web::Data<Pool>,
    path: web::Path<(i32, i32, String)>,
    claims: Option<Claims>,
) -> impl Responder {
    let (_collection_id, item_id, side) = path.into_inner();
    if !["front", "back"].contains(&side.as_str()) {
        return HttpResponse::BadRequest().body("Invalid side parameter");
    }

    match service::get_item_image(&pool, item_id, &side, claims.map(|c| c.sub)).await {
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
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Access denied") {
                HttpResponse::Forbidden().finish()
            } else {
                HttpResponse::InternalServerError().body(format!("Error: {}", e))
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/collections/{collection_id}/items/{item_id}/sound",
    tag = "collections",
    params(
        ("collection_id" = i32, Path, description = "Collection ID"),
        ("item_id" = i32, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Sound data", content_type = "audio/*"),
        (status = 404, description = "Sound not found"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/{collection_id}/items/{item_id}/sound")]
pub async fn get_item_sound(
    pool: web::Data<Pool>,
    path: web::Path<(i32, i32)>,
    claims: Option<Claims>,
) -> impl Responder {
    let (_collection_id, item_id) = path.into_inner();

    match service::get_item_sound(&pool, item_id, claims.map(|c| c.sub)).await {
        Ok(Some((sound_data, mime_type))) => {
            let cd = ContentDisposition {
                disposition: DispositionType::Inline,
                parameters: vec![],
            };
            HttpResponse::Ok()
                .content_type(mime_type)
                .insert_header(cd)
                .body(sound_data)
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Access denied") {
                HttpResponse::Forbidden().finish()
            } else {
                HttpResponse::InternalServerError().body(format!("Error: {}", e))
            }
        }
    }
}

#[utoipa::path(
    put,
    path = "/collections/{collection_id}/items/{item_id}/media",
    tag = "collections",
    params(
        ("collection_id" = i32, Path, description = "Collection ID"),
        ("item_id" = i32, Path, description = "Item ID")
    ),
    request_body = UpdateItemRequest,
    responses(
        (status = 200, description = "Media updated successfully"),
        (status = 400, description = "Invalid request - validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User doesn't have access to collection"),
        (status = 404, description = "Collection or item not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Update item media",
    description = "Updates the front/back images and/or custom sound for a collection item. \
                  Supports image (JPEG, PNG, GIF, WebP) and audio (MP3, OGG, WEBM; WAV not supported) formats \
                  with a 5MB size limit."
)]
#[put("/{collection_id}/items/{item_id}/media")]
pub async fn update_item_media(
    pool: web::Data<Pool>,
    claims: Claims,
    path: web::Path<(i32, i32)>,
    req: web::Json<UpdateItemRequest>,
) -> impl Responder {
    let (collection_id, item_id) = path.into_inner();
    match service::update_item_images(&pool, collection_id, item_id, claims.sub, &req).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Collection not found") {
                HttpResponse::NotFound().finish()
            } else if msg.contains("Access denied") {
                HttpResponse::Forbidden().finish()
            } else if msg.contains("Invalid image") || msg.contains("Invalid audio") || msg.contains("exceeds 5MB") {
                HttpResponse::BadRequest().body(msg)
            } else {
                HttpResponse::InternalServerError().body(format!("Failed to update media: {}", e))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/collections/import/json",
    tag = "collections",
    request_body = ImportJsonRequest,
    responses(
        (status = 200, description = "Collection imported successfully", body = ImportJsonResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Import collection from JSON",
    description = "Creates a new collection from a JSON file containing word definitions. Items without definition_id will be skipped and reported in
warnings."
)]
#[post("/import/json")]
pub async fn import_json(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<ImportJsonRequest>,
) -> impl Responder {
    match service::import_json(&pool, claims.sub, &req).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to import collection: {}", e)
        })),
    }
}

#[utoipa::path(
    post,
    path = "/collections/{id}/import/json",
    tag = "collections",
    params(
        ("id" = i32, Path, description = "Target Collection ID")
    ),
    request_body = ImportCollectionJsonRequest,
    responses(
        (status = 200, description = "Import completed", body = ImportCollectionJsonResponse),
        (status = 400, description = "Invalid request data"),
        (status = 403, description = "Access denied to target collection"),
        (status = 404, description = "Target collection not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Import items into collection from JSON",
    description = "Imports items from a JSON export (using CollectionExportItem format) into an existing collection. Skips items that already exist in the target collection based on definition_id
or free content."
)]
#[post("/{id}/import/json")]
pub async fn import_collection_from_json(
    pool: web::Data<Pool>,
    claims: Claims,
    id: web::Path<i32>,
    req: web::Json<ImportCollectionJsonRequest>,
) -> impl Responder {
    match super::service::import_collection_from_json(
        &pool,
        id.into_inner(),
        claims.sub,
        &req.items,
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => crate::utils::handle_import_error(Box::new(e)),
    }
}

#[utoipa::path(
    get,
    path = "/collections/{id}/export",
    tag = "collections",
    params(
        ("id" = i32, Path, description = "Collection ID")
    ),
    responses(
        (status = 200, description = "Full collection export (items, levels, flashcard directions)", body = CollectionFullExport),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Collection not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Export collection full",
    description = "Exports the collection as JSON including collection metadata, all items (with optional flashcard direction), and all flashcard levels with their params and item assignments. Same access as get collection (public or owner)."
)]
#[get("/{id}/export")]
pub async fn export_collection_full(
    pool: web::Data<Pool>,
    claims: Option<Claims>,
    id: web::Path<i32>,
) -> impl Responder {
    match service::export_collection_full(&pool, id.into_inner(), claims.map(|c| c.sub)).await {
        Ok(export) => HttpResponse::Ok().json(export),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Access denied") || msg.contains("Unauthorized") {
                HttpResponse::Forbidden().finish()
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to export collection: {}", e)
                }))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/collections/import/full",
    tag = "collections",
    request_body = ImportFullRequest,
    responses(
        (status = 200, description = "Collection imported with items and optionally flashcards/levels", body = ImportFullResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Import full collection",
    description = "Creates a new collection from a full export JSON (collection metadata, items, optional levels). When levels are present, flashcards are created for each item and levels with prerequisites and card assignments are recreated."
)]
#[post("/import/full")]
pub async fn import_full(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<ImportFullRequest>,
) -> impl Responder {
    match service::import_full(&pool, claims.sub, &req).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to import: {}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/collections/{collection_id}/search",
    tag = "collections",
    params(
        ("q" = String, Query, description = "Search query"),
        ("user_id" = Option<i32>, Query, description = "Filter by collection owner")
    ),
    responses(
        (status = 200, description = "Search results", body = SearchItemsResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Search collection items",
    description = "Searches across all collection items in word, definition, notes and free content fields"
)]
#[get("/{collection_id}/search")]
pub async fn search_collection_items(
    pool: web::Data<Pool>,
    claims: Claims,
    path: web::Path<i32>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let search_query = match query.get("q") {
        Some(q) => q,
        None => return HttpResponse::BadRequest().body("Missing search query"),
    };

    match service::search_items(&pool, claims.sub, search_query, Some(path.into_inner())).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Search failed: {}", e)
        })),
    }
}
