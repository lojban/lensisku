//! OpenRouter free-model catalog, Redis cache of two probed models, and background refresh.
//! Used by the assistant (fast path) and [`crate::background::service`] (periodic probe/update).

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::AppError;
use crate::middleware::cache::RedisCache;

/// Redis key for the two assistant models (JSON: [`CachedOpenRouterAssistantModels`]).
pub const REDIS_KEY_OPENROUTER_ASSISTANT_MODELS: &str = "openrouter:assistant_models_v1";

/// Max free models to pull from the catalog (newest first). First two may run in parallel when
/// streaming; the rest are used only as sequential fallbacks if earlier choices fail.
pub const OPENROUTER_MODEL_CANDIDATES_MAX: usize = 8;

/// (model_id, display_name) for use in UI. Display name is OpenRouter "name" or id as fallback.
pub type ModelIdName = (String, String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedOpenRouterAssistantModels {
    pub models: Vec<ModelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
}

/// OpenRouter /api/v1/models response: list of models (we use flexible parsing for pricing).
#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModel {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    created: Option<f64>,
    #[serde(default)]
    context_length: Option<f64>,
    #[serde(default)]
    architecture: Option<OpenRouterArchitecture>,
    /// e.g. `{ "prompt": "0", "completion": "0" }` — we require both to be zero for “free” models.
    #[serde(default)]
    pricing: Option<serde_json::Value>,
}

fn price_is_zero(v: &serde_json::Value) -> bool {
    match v {
        serde_json::Value::Number(n) => n.as_f64() == Some(0.0),
        serde_json::Value::String(s) => s == "0" || s == "0.0",
        serde_json::Value::Null => true,
        _ => false,
    }
}

#[derive(Debug, Deserialize)]
struct OpenRouterArchitecture {
    #[serde(default)]
    modality: Option<String>,
    #[serde(default)]
    input_modalities: Option<Vec<String>>,
    #[serde(default)]
    output_modalities: Option<Vec<String>>,
}

fn is_text_to_text(arch: &OpenRouterArchitecture) -> bool {
    let modality = arch.modality.as_deref().unwrap_or("");
    if modality == "text" {
        return true;
    }
    let inp = arch.input_modalities.as_deref().unwrap_or(&[]);
    let out = arch.output_modalities.as_deref().unwrap_or(&[]);
    inp.contains(&"text".to_string()) && out.contains(&"text".to_string())
}

/// OpenRouter aggregate / router endpoints — not concrete provider models (avoid for parallel runs).
fn is_placeholder_or_router_only_model(id: &str) -> bool {
    matches!(id, "openrouter/free" | "openrouter/auto")
}

async fn fetch_openrouter_models_list(
    base_url: &str,
    api_key: &str,
    query: &str,
) -> Result<Vec<OpenRouterModel>, AppError> {
    let url = format!(
        "{}/models{}",
        base_url.trim_end_matches('/'),
        query
    );
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("OpenRouter models request failed: {}", e)))?;

    let status = res.status();
    if !status.is_success() {
        let body = res.text().await.unwrap_or_else(|_| String::from("(no body)"));
        return Err(AppError::ExternalServiceWithRaw {
            message: format!("OpenRouter models returned {}", status),
            raw_response: body,
        });
    }

    let body: OpenRouterModelsResponse = res.json().await.map_err(|e| {
        AppError::ExternalService(format!("OpenRouter models JSON parse error: {}", e))
    })?;
    Ok(body.data)
}

/// Fetch the **newest free** (`pricing.prompt` and `pricing.completion` both zero) text models from
/// OpenRouter's public catalog.
///
/// Returns up to [`OPENROUTER_MODEL_CANDIDATES_MAX`] entries (newest first). The caller uses the first
/// two for optional parallel streaming; remaining entries are sequential fallbacks.
///
/// Uses the full model list (not `/models/user`). Excludes router placeholders (`openrouter/free`,
/// `openrouter/auto`) so parallel runs use real provider slugs when any exist. Prefers long context
/// (100k+), then falls back to 32k+. Sorted by `created` descending.
pub async fn fetch_latest_openrouter_models(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<ModelIdName>, AppError> {
    const MIN_CONTEXT_PREFERRED: f64 = 100_000.0;
    const MIN_CONTEXT_FALLBACK: f64 = 32_000.0;

    let mut data = match fetch_openrouter_models_list(
        base_url,
        api_key,
        "?output_modalities=text&supported_parameters=tools",
    )
    .await
    {
        Ok(d) => d,
        Err(e) => {
            log::debug!(
                "OpenRouter models (tools filter) failed or unavailable: {}; retrying without tools filter",
                e
            );
            fetch_openrouter_models_list(base_url, api_key, "?output_modalities=text").await?
        }
    };

    if data.is_empty() {
        data = fetch_openrouter_models_list(base_url, api_key, "?output_modalities=text").await?;
    }

    let pick = |min_ctx: f64| -> Vec<ModelIdName> {
        let mut eligible: Vec<(f64, String, String)> = data
            .iter()
            .filter_map(|m| {
                if is_placeholder_or_router_only_model(&m.id) {
                    return None;
                }
                let ctx = m.context_length.unwrap_or(0.0);
                if ctx <= min_ctx {
                    return None;
                }
                let arch = m.architecture.as_ref()?;
                if !is_text_to_text(arch) {
                    return None;
                }
                let pricing = m.pricing.as_ref()?;
                let prompt_zero = pricing.get("prompt").is_some_and(price_is_zero);
                let completion_zero = pricing.get("completion").is_some_and(price_is_zero);
                if !prompt_zero || !completion_zero {
                    return None;
                }
                let created = m.created.unwrap_or(0.0);
                let name = m
                    .name
                    .as_ref()
                    .filter(|s| !s.is_empty())
                    .map(String::to_owned)
                    .unwrap_or_else(|| m.id.clone());
                Some((created, m.id.clone(), name))
            })
            .collect();

        eligible.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        eligible
            .into_iter()
            .map(|(_, id, name)| (id, name))
            .take(OPENROUTER_MODEL_CANDIDATES_MAX)
            .collect()
    };

    let mut top = pick(MIN_CONTEXT_PREFERRED);
    if top.is_empty() {
        top = pick(MIN_CONTEXT_FALLBACK);
    }

    if top.is_empty() {
        log::warn!(
            "OpenRouter: no free (zero-priced) text models matched filters; falling back to openrouter/free"
        );
        return Ok(vec![("openrouter/free".to_string(), "OpenRouter Free".to_string())]);
    }
    Ok(top)
}

/// Minimal `POST /chat/completions` to verify the model accepts requests (no tools).
pub async fn probe_openrouter_model_chat_ok(
    base_url: &str,
    api_key: &str,
    model_id: &str,
) -> Result<(), AppError> {
    let client = reqwest::Client::new();
    let body = json!({
        "model": model_id,
        "messages": [{"role": "user", "content": "."}],
    });
    let res = client
        .post(format!(
            "{}/chat/completions",
            base_url.trim_end_matches('/')
        ))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let status = res.status();
    let text = res
        .text()
        .await
        .map_err(|e| AppError::ExternalService(format!("OpenRouter probe read body: {}", e)))?;

    if !status.is_success() {
        return Err(AppError::ExternalServiceWithRaw {
            message: format!("OpenRouter probe returned {}", status),
            raw_response: text,
        });
    }

    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        AppError::ExternalService(format!("OpenRouter probe JSON parse error: {}", e))
    })?;

    if v.get("error").is_some() {
        let raw = text.clone();
        return Err(AppError::ExternalServiceWithRaw {
            message: "OpenRouter probe returned error object in body".to_string(),
            raw_response: raw,
        });
    }

    let choices = v.get("choices").and_then(|c| c.as_array());
    if choices.is_none_or(|c| c.is_empty()) {
        return Err(AppError::ExternalServiceWithRaw {
            message: "OpenRouter probe: missing or empty choices".to_string(),
            raw_response: text,
        });
    }

    Ok(())
}

/// Reads two model ids from Redis if present and valid (non-empty ids, exactly two entries).
pub async fn load_cached_openrouter_assistant_models(
    redis: &RedisCache,
) -> Result<Option<Vec<ModelIdName>>, AppError> {
    let cached = redis
        .get::<CachedOpenRouterAssistantModels>(REDIS_KEY_OPENROUTER_ASSISTANT_MODELS)
        .await
        .map_err(|e| AppError::ExternalService(format!("Redis read assistant models: {}", e)))?;
    let Some(cached) = cached else {
        return Ok(None);
    };
    if cached.models.len() != 2 {
        return Ok(None);
    }
    let a = &cached.models[0];
    let b = &cached.models[1];
    if a.id.trim().is_empty() || b.id.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(vec![
        (a.id.clone(), a.name.clone()),
        (b.id.clone(), b.name.clone()),
    ]))
}

/// Loads Redis pair when available; otherwise fetches from the catalog. Second return is `true`
/// when candidates came **only** from Redis (so callers can retry with a full catalog fetch on failure).
pub async fn load_or_fetch_openrouter_candidates(
    redis: Option<&RedisCache>,
    base_url: &str,
    api_key: &str,
) -> Result<(Vec<ModelIdName>, bool), AppError> {
    if let Some(r) = redis {
        match load_cached_openrouter_assistant_models(r).await {
            Ok(Some(pair)) if pair.len() == 2 => {
                log::debug!(
                    "OpenRouter assistant: using two cached models from Redis: {}, {}",
                    pair[0].0, pair[1].0
                );
                return Ok((pair, true));
            }
            Ok(_) => {}
            Err(e) => {
                log::warn!(
                    "OpenRouter assistant: Redis read for cached models failed ({}); using catalog",
                    e
                );
            }
        }
    }
    let full = fetch_latest_openrouter_models(base_url, api_key).await?;
    Ok((full, false))
}

/// Walks catalog order, probes each model with a minimal chat request, stores the first two that succeed.
pub async fn refresh_openrouter_assistant_models_cache(
    redis: &RedisCache,
    base_url: &str,
    api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let catalog = fetch_latest_openrouter_models(base_url, api_key).await
        .map_err(|e| -> Box<dyn std::error::Error> { format!("{}", e).into() })?;

    let mut picked: Vec<ModelEntry> = Vec::new();
    for (id, name) in catalog {
        match probe_openrouter_model_chat_ok(base_url, api_key, &id).await {
            Ok(()) => {
                log::debug!("OpenRouter cache refresh: probe ok for {}", id);
                picked.push(ModelEntry { id, name });
                if picked.len() == 2 {
                    break;
                }
            }
            Err(e) => {
                log::debug!("OpenRouter cache refresh: skip {} ({})", id, e);
            }
        }
    }

    if picked.len() == 2 {
        redis
            .set(
                REDIS_KEY_OPENROUTER_ASSISTANT_MODELS,
                &CachedOpenRouterAssistantModels { models: picked },
                None,
            )
            .await?;
        log::info!("OpenRouter assistant model cache updated in Redis (two probed models).");
    } else {
        log::warn!(
            "OpenRouter cache refresh: found {} working model(s); keeping previous Redis entry if any",
            picked.len()
        );
    }

    Ok(())
}
