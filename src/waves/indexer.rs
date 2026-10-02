//! Small restart-safe backfill batches; source triggers keep lexical text current.
use deadpool_postgres::Pool;
use pgvector::Vector;

use super::text::{pooled_embedding, semantic_body};
use crate::error::{AppError, AppResult};
use crate::utils::embeddings::{discussion_passages, embeddings_disabled, get_batch_embeddings};

/// Returns the number of documents considered. Inference never holds a transaction
/// or row lock; publication checks the generation to discard concurrently edited text.
pub(crate) async fn index_pending(pool: &Pool) -> AppResult<usize> {
    if embeddings_disabled() {
        return Ok(0);
    }
    let sql = format!(
        "SELECT document_id, generation, kind, title, body FROM wave_search_documents d
         WHERE embedding IS NULL AND {} ORDER BY document_id LIMIT 8",
        include_str!("canonical.sql")
    );
    let rows = pool.get().await?.query(&sql, &[]).await?;
    let count = rows.len();
    for row in rows {
        let id: i64 = row.get("document_id");
        let generation: i64 = row.get("generation");
        let title: String = row.get("title");
        let body: String = row.get("body");
        let kind: String = row.get("kind");
        let passages = discussion_passages(&title, &semantic_body(&body, kind == "mail")).await?;
        let mut vectors = Vec::with_capacity(passages.len());
        for batch in passages.chunks(20) {
            vectors.extend(get_batch_embeddings(batch.to_vec()).await?);
        }
        if vectors.len() != passages.len() || vectors.iter().any(|v| v.len() != 384) {
            return Err(AppError::Internal(
                "Unexpected discussion embedding dimensions/count".into(),
            ));
        }
        let document_vector = Vector::from(pooled_embedding(&vectors));
        let mut client = pool.get().await?;
        let tx = client.transaction().await?;
        let updated = tx
            .execute(
                "UPDATE wave_search_documents SET embedding = $3
             WHERE document_id = $1 AND generation = $2 AND embedding IS NULL",
                &[&id, &generation, &document_vector],
            )
            .await?;
        if updated == 1 {
            for (i, (content, vector)) in passages.iter().zip(vectors).enumerate() {
                tx.execute(
                    "INSERT INTO wave_search_chunks (document_id, chunk_index, content, embedding)
                     VALUES ($1,$2,$3,$4)",
                    &[&id, &(i as i32), content, &Vector::from(vector)],
                )
                .await?;
            }
        }
        tx.commit().await?;
    }
    Ok(count)
}
