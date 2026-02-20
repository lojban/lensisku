//! In-process embedding inference using ONNX Runtime.
//!
//! Model: `onnx-community/embeddinggemma-300m-ONNX` (`onnx/model_q4.onnx`) — a
//! 300 M-parameter Google EmbeddingGemma model.  Produces 768-dimensional
//! vectors; output is mean-pooled over the last hidden state and L2-normalised.
//!
//! We bypass `fastembed`'s `UserDefinedEmbeddingModel` because that path uses
//! `commit_from_memory`, which cannot resolve the model's external data file
//! (`model_q4.onnx_data`).  Instead we download both files via `hf_hub` (which
//! places them in the same cache directory), then load the session directly from
//! the file-system path so that ORT can locate the companion data shard.
//!
//! The model is initialised lazily on first use and cached for the lifetime of
//! the process.  `HF_HOME` / `HF_ENDPOINT` env-vars are respected.

use ndarray::{Array, Array2, Array3, Axis};
use once_cell::sync::OnceCell;
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Value,
};
use parking_lot::Mutex;
use tokenizers::{PaddingParams, PaddingStrategy, Tokenizer, TruncationParams};

use crate::error::{AppError, AppResult};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const HF_REPO: &str = "onnx-community/embeddinggemma-300m-ONNX";
const ONNX_FILE: &str = "onnx/model_q4.onnx";
/// Companion external-data shard referenced inside the ONNX protobuf.
const ONNX_DATA_FILE: &str = "onnx/model_q4.onnx_data";
/// EmbeddingGemma max context window.
const MAX_LENGTH: usize = 2048;
/// Output embedding dimension.
pub const EMBEDDING_DIM: usize = 768;

// ---------------------------------------------------------------------------
// Model wrapper
// ---------------------------------------------------------------------------

struct EmbeddingModel {
    session: Session,
    tokenizer: Tokenizer,
}

static MODEL: OnceCell<Mutex<EmbeddingModel>> = OnceCell::new();

// ---------------------------------------------------------------------------
// Initialisation
// ---------------------------------------------------------------------------

fn hf_cache_dir() -> std::path::PathBuf {
    std::env::var("HF_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".cache")
                .join("huggingface")
        })
}

fn pull_model() -> AppResult<EmbeddingModel> {
    log::info!("Initialising embedding model ({HF_REPO}/{ONNX_FILE}) — first run may download…");

    let endpoint =
        std::env::var("HF_ENDPOINT").unwrap_or_else(|_| "https://huggingface.co".to_string());

    let api = hf_hub::api::sync::ApiBuilder::new()
        .with_cache_dir(hf_cache_dir())
        .with_endpoint(endpoint)
        .with_progress(true)
        .build()
        .map_err(|e| AppError::Internal(format!("HF API build failed: {e}")))?;

    let repo = api.model(HF_REPO.to_string());

    // Download the ONNX file.  The returned path is the local cache location.
    let onnx_path = repo
        .get(ONNX_FILE)
        .map_err(|e| AppError::Internal(format!("Failed to fetch {ONNX_FILE}: {e}")))?;

    // Download the external data shard so ORT can find it beside the ONNX file.
    repo.get(ONNX_DATA_FILE)
        .map_err(|e| AppError::Internal(format!("Failed to fetch {ONNX_DATA_FILE}: {e}")))?;

    // Build ORT session from the file path (not from memory) so ORT resolves
    // the external data file relative to the ONNX file's directory.
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let session = Session::builder()
        .map_err(|e| AppError::Internal(format!("ORT builder failed: {e}")))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| AppError::Internal(format!("ORT opt level failed: {e}")))?
        .with_intra_threads(threads)
        .map_err(|e| AppError::Internal(format!("ORT threads failed: {e}")))?
        .commit_from_file(&onnx_path)
        .map_err(|e| AppError::Internal(format!("ORT session load failed: {e}")))?;

    // Build tokenizer from downloaded JSON files.
    let read = |name: &str| -> AppResult<Vec<u8>> {
        let p = repo
            .get(name)
            .map_err(|e| AppError::Internal(format!("Failed to fetch {name}: {e}")))?;
        std::fs::read(&p)
            .map_err(|e| AppError::Internal(format!("Failed to read {name}: {e}")))
    };

    let tokenizer_bytes = read("tokenizer.json")?;
    let config_bytes = read("config.json")?;
    let tokenizer_config_bytes = read("tokenizer_config.json")?;
    let special_tokens_map_bytes = read("special_tokens_map.json")?;

    let config: serde_json::Value = serde_json::from_slice(&config_bytes)
        .map_err(|e| AppError::Internal(format!("Bad config.json: {e}")))?;
    let tokenizer_config: serde_json::Value = serde_json::from_slice(&tokenizer_config_bytes)
        .map_err(|e| AppError::Internal(format!("Bad tokenizer_config.json: {e}")))?;
    let special_tokens_map: serde_json::Value = serde_json::from_slice(&special_tokens_map_bytes)
        .map_err(|e| AppError::Internal(format!("Bad special_tokens_map.json: {e}")))?;

    let model_max_length = tokenizer_config["model_max_length"]
        .as_f64()
        .unwrap_or(MAX_LENGTH as f64) as usize;
    let max_length = MAX_LENGTH.min(model_max_length);

    let pad_id = config["pad_token_id"].as_u64().unwrap_or(0) as u32;
    let pad_token = tokenizer_config["pad_token"]
        .as_str()
        .unwrap_or("<pad>")
        .to_string();

    let mut tokenizer = Tokenizer::from_bytes(&tokenizer_bytes)
        .map_err(|e| AppError::Internal(format!("Bad tokenizer.json: {e}")))?;

    tokenizer
        .with_padding(Some(PaddingParams {
            strategy: PaddingStrategy::BatchLongest,
            pad_token,
            pad_id,
            ..Default::default()
        }))
        .with_truncation(Some(TruncationParams {
            max_length,
            ..Default::default()
        }))
        .map_err(|e| AppError::Internal(format!("Tokenizer config failed: {e}")))?;

    // Register special tokens from special_tokens_map.json.
    if let serde_json::Value::Object(map) = special_tokens_map {
        for (_, val) in &map {
            let content = if val.is_string() {
                val.as_str().map(str::to_owned)
            } else {
                val["content"].as_str().map(str::to_owned)
            };
            if let Some(content) = content {
                tokenizer.add_special_tokens(&[tokenizers::AddedToken {
                    content,
                    special: true,
                    ..Default::default()
                }]);
            }
        }
    }

    log::info!("Embedding model loaded ({HF_REPO}).");
    Ok(EmbeddingModel { session, tokenizer })
}

fn get_model() -> AppResult<&'static Mutex<EmbeddingModel>> {
    MODEL.get_or_try_init(|| pull_model().map(Mutex::new))
}

// ---------------------------------------------------------------------------
// Inference helpers
// ---------------------------------------------------------------------------

/// Run a batch through the model and return mean-pooled, L2-normalised vectors.
fn embed_batch(model: &EmbeddingModel, texts: &[&str]) -> AppResult<Vec<Vec<f32>>> {
    let encodings = model
        .tokenizer
        .encode_batch(texts.to_vec(), true)
        .map_err(|e| AppError::Internal(format!("Tokenise failed: {e}")))?;

    let seq_len = encodings[0].len();
    let batch = encodings.len();

    let mut ids_flat = Vec::with_capacity(batch * seq_len);
    let mut mask_flat = Vec::with_capacity(batch * seq_len);

    for enc in &encodings {
        ids_flat.extend(enc.get_ids().iter().map(|&x| x as i64));
        mask_flat.extend(enc.get_attention_mask().iter().map(|&x| x as i64));
    }

    let ids_arr = Array::from_shape_vec((batch, seq_len), ids_flat)
        .map_err(|e| AppError::Internal(format!("Shape error: {e}")))?;
    
    // Convert mask to f32 for pooling before moving mask_flat into the i64 array for ORT.
    let mask_f32_flat: Vec<f32> = mask_flat.iter().map(|&x| x as f32).collect();
    let mask_arr: Array2<i64> = Array::from_shape_vec((batch, seq_len), mask_flat)
        .map_err(|e| AppError::Internal(format!("Shape error: {e}")))?;

    let need_token_type_ids = model
        .session
        .inputs
        .iter()
        .any(|i| i.name == "token_type_ids");

    // Build ORT Values before the macro so errors can be mapped to AppError.
    let ids_val = Value::from_array(ids_arr)
        .map_err(|e| AppError::Internal(format!("ORT input error: {e}")))?;
    let mask_val = Value::from_array(mask_arr.view())
        .map_err(|e| AppError::Internal(format!("ORT input error: {e}")))?;

    let mut inputs = ort::inputs!["input_ids" => ids_val, "attention_mask" => mask_val]
        .map_err(|e| AppError::Internal(format!("ORT inputs error: {e}")))?;

    if need_token_type_ids {
        let type_ids: Array2<i64> = Array::zeros((batch, seq_len));
        let type_ids_val = Value::from_array(type_ids)
            .map_err(|e| AppError::Internal(format!("ORT input error: {e}")))?;
        inputs.push(("token_type_ids".into(), type_ids_val.into()));
    }

    let outputs = model
        .session
        .run(inputs)
        .map_err(|e| AppError::Internal(format!("ORT run failed: {e}")))?;

    // EmbeddingGemma exports `last_hidden_state` [batch, seq, 768].
    // try_extract_tensor() returns ArrayViewD<f32>.
    let hidden = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|e| AppError::Internal(format!("ORT extract failed: {e}")))?;

    let shape = hidden.shape();
    let (b, s, d) = (shape[0], shape[1], shape[2]);

    let raw: Array3<f32> = Array::from_shape_vec(
        (b, s, d),
        hidden.as_slice().expect("contiguous").to_vec(),
    )
    .map_err(|e| AppError::Internal(format!("Shape error: {e}")))?;

    let mask_f32: Array2<f32> = Array::from_shape_vec((b, s), mask_f32_flat)
        .map_err(|e| AppError::Internal(format!("Shape error: {e}")))?;


    // Mean pooling: sum(hidden * mask) / sum(mask), then L2-normalise.
    let mut embeddings = Vec::with_capacity(b);
    for i in 0..b {
        let h = raw.index_axis(Axis(0), i); // [s, d]
        let m = mask_f32.index_axis(Axis(0), i); // [s]
        let mask_sum = m.sum().max(1e-9);

        // weighted sum over sequence dimension
        let pooled: Vec<f32> = (0..d)
            .map(|j| {
                let col = h.index_axis(Axis(1), j); // [s]
                col.iter().zip(m.iter()).map(|(v, w)| v * w).sum::<f32>() / mask_sum
            })
            .collect();

        // L2 normalise
        let norm = pooled.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-12);
        let normed: Vec<f32> = pooled.iter().map(|x| x / norm).collect();
        embeddings.push(normed);
    }

    Ok(embeddings)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a single embedding vector (768-dim, mean-pooled, L2-normalised).
///
/// Runs on a Tokio blocking thread so the async runtime is not stalled.
pub async fn get_embedding(text: &str) -> AppResult<Vec<f32>> {
    let text = text.to_owned();
    tokio::task::spawn_blocking(move || {
        let guard = get_model()?.lock();
        let mut batch = embed_batch(&guard, &[text.as_str()])?;
        batch
            .pop()
            .ok_or_else(|| AppError::Internal("Empty embedding result".into()))
    })
    .await
    .map_err(|e| AppError::Internal(format!("spawn_blocking panicked: {e}")))?
}

/// Generate embeddings for a batch of texts (single ONNX forward pass).
pub async fn get_batch_embeddings(texts: Vec<String>) -> AppResult<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(vec![]);
    }
    tokio::task::spawn_blocking(move || {
        let guard = get_model()?.lock();
        let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
        embed_batch(&guard, &refs)
    })
    .await
    .map_err(|e| AppError::Internal(format!("spawn_blocking panicked: {e}")))?
}
