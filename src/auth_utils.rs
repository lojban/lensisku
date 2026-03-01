use crate::{AppError, AppResult};
use deadpool_postgres::Transaction;

/// Verifies that the user (or anonymous) may read collection data.
/// Anonymous (user_id None): only public collections.
/// Logged-in: public collections or collection owner.
pub async fn verify_collection_read_access(
    transaction: &Transaction<'_>,
    collection_id: i32,
    user_id: Option<i32>,
) -> AppResult<()> {
    let row = transaction
        .query_one(
            "SELECT user_id, is_public FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let owner_id: i32 = row.get("user_id");
    let is_public: bool = row.get("is_public");
    match user_id {
        None if !is_public => Err(AppError::Auth("Access denied".to_string())),
        Some(uid) if !is_public && owner_id != uid => {
            Err(AppError::Auth("Access denied".to_string()))
        }
        _ => Ok(()),
    }
}

pub async fn verify_collection_ownership(
    transaction: &Transaction<'_>,
    collection_id: i32,
    user_id: i32,
) -> AppResult<()> {
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await?
        .get("user_id");

    if owner_id != user_id {
        return Err(AppError::Auth(
            "Access Denied: Collection does not belong to user".to_string(),
        ));
    }
    Ok(())
}

pub async fn verify_flashcard_ownership(
    transaction: &Transaction<'_>,
    flashcard_id: i32,
    user_id: i32,
) -> AppResult<()> {
    let owner_id: i32 = transaction
        .query_one(
            "SELECT c.user_id
             FROM flashcards f
             JOIN collections c ON f.collection_id = c.collection_id
             WHERE f.id = $1",
            &[&flashcard_id],
        )
        .await?
        .get("user_id");

    if owner_id != user_id {
        return Err(AppError::Auth(
            "Access Denied: Flashcard does not belong to user".to_string(),
        ));
    }
    Ok(())
}
