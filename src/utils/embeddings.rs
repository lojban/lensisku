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
        let model = model_mutex.lock();
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
        let model = model_mutex.lock();
        let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        model
            .embed(refs, None)
            .map_err(|e| AppError::Internal(format!("Batch embedding failed: {e}")))
    })
    .await
    .map_err(|e| AppError::Internal(format!("spawn_blocking panicked: {e}")))?
}
