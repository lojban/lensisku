//! In-process embedding inference using `fastembed-rs` (ONNX Runtime).
//!
//! Model: `AllMiniLML6V2` — identical to `Xenova/all-MiniLM-L6-v2` used by the
//! semantic-search MCP. Produces 384-dimensional vectors with mean pooling and
//! L2 normalisation applied internally, so the output is directly comparable to
//! embeddings stored in the database.
//!
//! The model is initialised lazily on first use and cached for the lifetime of
//! the process.  The first call downloads ~80 MB from Hugging Face (or reads
//! from the local cache at `~/.cache/huggingface/hub` / `FASTEMBED_CACHE_PATH`).
//!
//! Set `DISABLE_EMBEDDINGS=1` (or `true`/`yes`) to skip loading the model and
//! all embedding computation (e.g. for local development).

use std::env;

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;

use crate::error::{AppError, AppResult};

static MODEL: OnceCell<Mutex<TextEmbedding>> = OnceCell::new();

/// Returns true when embedding model loading and inference are disabled via env.
pub fn embeddings_disabled() -> bool {
    env::var("DISABLE_EMBEDDINGS")
        .ok()
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn get_model() -> AppResult<&'static Mutex<TextEmbedding>> {
    if embeddings_disabled() {
        return Err(AppError::Internal(
            "Embeddings are disabled (DISABLE_EMBEDDINGS is set)".into(),
        ));
    }
    MODEL.get_or_try_init(|| {
        log::info!("Initialising embedding model (AllMiniLML6V2) — first run may download ~80 MB…");
        let model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::AllMiniLML6V2))
            .map_err(|e| AppError::Internal(format!("Failed to load embedding model: {e}")))?;
        log::info!("Embedding model loaded.");
        Ok(Mutex::new(model))
    })
}

/// Generate a single embedding vector (384-dim, L2-normalised).
///
/// Runs the blocking ONNX inference on a Tokio blocking thread so the async
/// runtime is not stalled.
pub async fn get_embedding(text: &str) -> AppResult<Vec<f32>> {
    let text = text.to_owned();
    tokio::task::spawn_blocking(move || {
        let model_mutex = get_model()?;
        let mut model = model_mutex.lock();
        let mut results = model
            .embed(vec![text.as_str()], None)
            .map_err(|e| AppError::Internal(format!("Embedding failed: {e}")))?;
        results
            .pop()
            .ok_or_else(|| AppError::Internal("Empty embedding result".into()))
    })
    .await
    .map_err(|e| AppError::Internal(format!("spawn_blocking panicked: {e}")))?
}

/// Generate embeddings for a batch of texts.
///
/// More efficient than calling [`get_embedding`] in a loop because the model
/// processes the whole batch in a single ONNX forward pass.
pub async fn get_batch_embeddings(texts: Vec<String>) -> AppResult<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(vec![]);
    }
    tokio::task::spawn_blocking(move || {
        let model_mutex = get_model()?;
        let mut model = model_mutex.lock();
        let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        model
            .embed(refs, None)
            .map_err(|e| AppError::Internal(format!("Batch embedding failed: {e}")))
    })
    .await
    .map_err(|e| AppError::Internal(format!("spawn_blocking panicked: {e}")))?
}

/// Paragraph-aware passages with a hard WordPiece budget for the current model.
/// Offsets refer to UTF-8 bytes, so splitting also preserves non-Latin text.
/// Long paragraphs overlap by 32 tokens to retain context across boundaries.
pub async fn discussion_passages(title: &str, body: &str) -> AppResult<Vec<String>> {
    let title = title.to_owned();
    let body = body.replace("\r\n", "\n");
    tokio::task::spawn_blocking(move || {
        let mut tokenizer = get_model()?.lock().tokenizer.clone();
        tokenizer
            .with_truncation(None)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tokenizer.with_padding(None);
        let title_encoding = tokenizer
            .encode(title.as_str(), false)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let title_end = title_encoding
            .get_offsets()
            .get(31)
            .map(|(_, end)| *end)
            .unwrap_or(title.len());
        let heading = &title[..title_end];
        // At most 32 heading + 220 passage + 2 special tokens = 254 tokens.
        let mut passages = Vec::new();
        for paragraph in body.split("\n\n").map(str::trim).filter(|s| !s.is_empty()) {
            let encoding = tokenizer
                .encode(paragraph, false)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let offsets = encoding.get_offsets();
            let mut start = 0;
            while start < offsets.len() {
                let end = (start + 220).min(offsets.len());
                let passage = &paragraph[offsets[start].0..offsets[end - 1].1];
                passages.push(if heading.is_empty() {
                    passage.to_owned()
                } else {
                    format!("{heading}\n{passage}")
                });
                if end == offsets.len() {
                    break;
                }
                start = end - 32;
            }
        }
        if passages.is_empty() {
            // Also marks empty/quoted-only messages as processed; title remains useful.
            passages.push(heading.to_owned());
        }
        Ok(passages)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Passage preparation failed: {e}")))?
}

#[cfg(test)]
mod discussion_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "loads the embedding model/tokenizer"]
    async fn passages_retain_tail_and_obey_wordpiece_budget() -> AppResult<()> {
        let body = format!(
            "{}\n\nFinal paragraph about quantifier scope.",
            "na'e cmavo .i zo broda — 日本語 中文 العربية ".repeat(200)
        );
        let passages = discussion_passages(&"A very long title ".repeat(40), &body).await?;
        assert!(passages.len() > 2);
        assert!(passages
            .last()
            .is_some_and(|s| s.contains("Final paragraph")));
        let mut tokenizer = get_model()?.lock().tokenizer.clone();
        tokenizer
            .with_truncation(None)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tokenizer.with_padding(None);
        for passage in passages {
            let encoded = tokenizer
                .encode(passage, true)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            assert!(
                encoded.len() <= 256,
                "Passage exceeded model token budget: {}",
                encoded.len()
            );
        }
        Ok(())
    }
}
