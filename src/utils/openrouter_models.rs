//! OpenRouter free-model catalog, Redis cache of two probed models, and background refresh.
//! Used by the assistant (fast path) and [`crate::background::service`] (periodic probe/update).

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::error::AppError;
use crate::middleware::cache::RedisCache;

/// Redis key for the two assistant models (JSON: [`CachedOpenRouterAssistantModels`]).
pub const REDIS_KEY_OPENROUTER_ASSISTANT_MODELS: &str = "openrouter:assistant_models_v1";

/// Last pair that passed a probe. Never evicted by chat failures (only overwritten by the next
/// successful refresh) so chat does not fall back to untested newest catalog models.
pub const REDIS_KEY_OPENROUTER_LAST_GOOD_MODELS: &str = "openrouter:assistant_models_last_good_v1";
const LAST_GOOD_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// Free models observed to answer the assistant's tool-calling request (2026-10-31). Probed
/// alongside the newest catalog entries and used before the raw catalog when Redis is empty.
/// Override with `OPENROUTER_ASSISTANT_FALLBACK_MODELS` (comma-separated ids).
const DEFAULT_FALLBACK_MODELS: &[&str] = &[
    "nvidia/nemotron-3.5-super-vl:free",
    "dots-studio/dots-3-note-preview:free",
    "nvidia/nemotron-3.5-lightning:free",
    "poolside/laguna-s-2.1:free",
];

/// Per-model ceiling for the cheap background probe (all candidates run concurrently).
pub const OPENROUTER_PROBE_TIMEOUT: Duration = Duration::from_secs(45);

pub fn fallback_model_ids() -> Vec<String> {
    parse_fallback_models(std::env::var("OPENROUTER_ASSISTANT_FALLBACK_MODELS").ok().as_deref())
}

fn parse_fallback_models(raw: Option<&str>) -> Vec<String> {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => s
            .split(',')
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect(),
        None => DEFAULT_FALLBACK_MODELS.iter().map(|s| s.to_string()).collect(),
    }
}

/// Stored pair first, then known-good fallbacks, then catalog; de-duplicated, order kept.
pub fn merge_candidates(lists: &[Vec<ModelIdName>]) -> Vec<ModelIdName> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for l in lists {
        for (id, name) in l {
            if !id.trim().is_empty() && seen.insert(id.clone()) {
                out.push((id.clone(), name.clone()));
            }
        }
    }
    out
}

pub fn fallback_candidates() -> Vec<ModelIdName> {
    fallback_model_ids().into_iter().map(|id| (id.clone(), id)).collect()
}

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

// /// Checks if a model ID belongs to a preferred provider (nvidia, google, anthropic, openai, minimax).
// fn is_preferred_provider(model_id: &str) -> bool {
//     const PREFERRED_PROVIDERS: &[&str] = &[
//         "nvidia/",
//         "google/",
//         "anthropic/",
//         "openai/",
//         "minimax/",
//         "deepseek/",
//         "moonshotai/",
//         "qwen/",
//     ];
//     PREFERRED_PROVIDERS
//         .iter()
//         .any(|prefix| model_id.starts_with(prefix))
// }

async fn fetch_openrouter_models_list(
    base_url: &str,
    api_key: &str,
    query: &str,
) -> Result<Vec<OpenRouterModel>, AppError> {
    let url = format!("{}/models{}", base_url.trim_end_matches('/'), query);
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| {
            AppError::ExternalService(format!("OpenRouter models request failed: {}", e))
        })?;

    let status = res.status();
    if !status.is_success() {
        let body = res
            .text()
            .await
            .unwrap_or_else(|_| String::from("(no body)"));
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
///
/// Does not prioritize any provider; sorts purely by `created` descending.
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

        eligible.sort_by(|a, b| {
            // Provider priority disabled: do not limit models to certain companies.
            // let a_preferred = is_preferred_provider(&a.1);
            // let b_preferred = is_preferred_provider(&b.1);
            // match (a_preferred, b_preferred) {
            //     (true, false) => std::cmp::Ordering::Less,
            //     (false, true) => std::cmp::Ordering::Greater,
            //     _ => b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal),
            // }
            b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
        });
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
        return Ok(vec![(
            "openrouter/free".to_string(),
            "OpenRouter Free".to_string(),
        )]);
    }
    Ok(top)
}

/// Reads two model ids from Redis if present and valid (non-empty ids, exactly two entries).
async fn load_cached_pair(
    redis: &RedisCache,
    key: &str,
) -> Result<Option<Vec<ModelIdName>>, AppError> {
    let cached = redis
        .get::<CachedOpenRouterAssistantModels>(key)
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
        for key in [
            REDIS_KEY_OPENROUTER_ASSISTANT_MODELS,
            REDIS_KEY_OPENROUTER_LAST_GOOD_MODELS,
        ] {
            match load_cached_pair(r, key).await {
                Ok(Some(pair)) if pair.len() == 2 => {
                    log::debug!(
                        "OpenRouter assistant: using cached models from Redis `{key}`: {}, {}",
                        pair[0].0,
                        pair[1].0
                    );
                    return Ok((pair, true));
                }
                Ok(_) => {}
                Err(e) => {
                    log::warn!("OpenRouter assistant: Redis read `{key}` failed ({e})");
                }
            }
        }
    }
    // No probed pair: known-good fallbacks first, untested newest catalog models last.
    let catalog = fetch_latest_openrouter_models(base_url, api_key)
        .await
        .unwrap_or_default();
    Ok((merge_candidates(&[fallback_candidates(), catalog]), false))
}

/// Removes a model id from the cached assistant models pair in Redis.
///
/// If the cached entry contains the given id, the whole key is invalidated so the next request
/// either uses a fresh catalog fetch or waits for the next background refresh to repopulate it
/// with two probed models. Missing key / id-not-present is a no-op.
pub async fn evict_openrouter_assistant_model_from_cache(
    redis: &RedisCache,
    model_id: &str,
) -> Result<(), AppError> {
    let cached = match redis
        .get::<CachedOpenRouterAssistantModels>(REDIS_KEY_OPENROUTER_ASSISTANT_MODELS)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => return Ok(()),
        Err(e) => {
            return Err(AppError::ExternalService(format!(
                "Redis read assistant models for eviction: {}",
                e
            )));
        }
    };
    if !cached.models.iter().any(|m| m.id == model_id) {
        return Ok(());
    }
    log::warn!(
        "OpenRouter assistant: evicting failing model `{}` from Redis cache",
        model_id
    );
    redis
        .invalidate(REDIS_KEY_OPENROUTER_ASSISTANT_MODELS)
        .await
        .map_err(|e| AppError::ExternalService(format!("Redis del assistant models: {}", e)))?;
    Ok(())
}

/// Probes catalog + known-good fallbacks **concurrently** (each bounded by
/// [`OPENROUTER_PROBE_TIMEOUT`]) and stores the two fastest successes in Redis (both the hot key
/// and the persistent last-good key). Returns how many models passed, so the caller can retry
/// sooner when fewer than two did; the previous Redis entries are kept in that case.
pub async fn refresh_openrouter_assistant_models_cache<F, Fut>(
    redis: &RedisCache,
    base_url: &str,
    api_key: &str,
    probe: F,
) -> Result<usize, AppError>
where
    F: Fn(String, String) -> Fut,
    Fut: std::future::Future<Output = Result<(), AppError>>,
{
    let catalog = fetch_latest_openrouter_models(base_url, api_key)
        .await
        .unwrap_or_default();
    let mut prev = Vec::new();
    if let Ok(Some(p)) = load_cached_pair(redis, REDIS_KEY_OPENROUTER_LAST_GOOD_MODELS).await {
        prev = p;
    }
    let candidates = merge_candidates(&[prev, fallback_candidates(), catalog]);
    let picked = probe_concurrently(candidates, &probe).await;

    if picked.len() >= 2 {
        let models: Vec<ModelEntry> = picked
            .iter()
            .take(2)
            .map(|(id, name)| ModelEntry {
                id: id.clone(),
                name: name.clone(),
            })
            .collect();
        let entry = CachedOpenRouterAssistantModels { models };
        redis
            .set(REDIS_KEY_OPENROUTER_ASSISTANT_MODELS, &entry, None)
            .await?;
        redis
            .set(REDIS_KEY_OPENROUTER_LAST_GOOD_MODELS, &entry, Some(LAST_GOOD_TTL))
            .await?;
        log::info!(
            "OpenRouter assistant model cache updated: {}, {} ({} passed)",
            picked[0].0,
            picked[1].0,
            picked.len()
        );
    } else {
        log::warn!(
            "OpenRouter cache refresh: found {} working model(s); keeping previous Redis entry if any",
            picked.len()
        );
    }
    Ok(picked.len())
}

/// Runs `probe` for every candidate at once; returns successes ordered fastest first.
pub async fn probe_concurrently<F, Fut>(candidates: Vec<ModelIdName>, probe: &F) -> Vec<ModelIdName>
where
    F: Fn(String, String) -> Fut,
    Fut: std::future::Future<Output = Result<(), AppError>>,
{
    let futs = candidates.into_iter().map(|(id, name)| async move {
        let started = Instant::now();
        let res = tokio::time::timeout(OPENROUTER_PROBE_TIMEOUT, probe(id.clone(), name.clone()))
            .await;
        match res {
            Ok(Ok(())) => {
                log::debug!("OpenRouter probe ok: {id} ({} ms)", started.elapsed().as_millis());
                Some((started.elapsed(), id, name))
            }
            Ok(Err(e)) => {
                log::debug!("OpenRouter probe skip {id}: {e}");
                None
            }
            Err(_) => {
                log::debug!("OpenRouter probe skip {id}: timed out");
                None
            }
        }
    });
    let mut ok: Vec<(Duration, String, String)> =
        futures::future::join_all(futs).await.into_iter().flatten().collect();
    ok.sort_by_key(|(d, _, _)| *d);
    ok.into_iter().map(|(_, id, name)| (id, name)).collect()
}

#[cfg(test)]
mod model_selection_tests {
    use super::*;

    fn m(id: &str) -> ModelIdName {
        (id.to_string(), id.to_string())
    }

    #[test]
    fn merge_dedups_and_keeps_priority() {
        let out = merge_candidates(&[vec![m("a"), m("b")], vec![m("b"), m("c")], vec![m(""), m("a"), m("d")]]);
        let ids: Vec<_> = out.into_iter().map(|x| x.0).collect();
        assert_eq!(ids, ["a", "b", "c", "d"]);
    }

    #[test]
    fn fallback_env_parsing() {
        assert_eq!(parse_fallback_models(Some(" x , ,y ")), ["x", "y"]);
        assert_eq!(parse_fallback_models(Some("  ")).len(), DEFAULT_FALLBACK_MODELS.len());
        assert_eq!(parse_fallback_models(None).len(), DEFAULT_FALLBACK_MODELS.len());
    }

    #[tokio::test]
    async fn probes_run_concurrently_and_rank_fastest_first() {
        let probe = |id: String, _n: String| async move {
            let ms = match id.as_str() {
                "slow" => 300,
                "fast" => 10,
                "mid" => 100,
                _ => 0,
            };
            tokio::time::sleep(Duration::from_millis(ms)).await;
            if id == "bad" {
                Err(AppError::ExternalService("429".into()))
            } else {
                Ok(())
            }
        };
        let started = Instant::now();
        let got = probe_concurrently(vec![m("slow"), m("bad"), m("fast"), m("mid")], &probe).await;
        // Sequential would take >= 410 ms.
        assert!(started.elapsed() < Duration::from_millis(390), "{:?}", started.elapsed());
        let ids: Vec<_> = got.into_iter().map(|x| x.0).collect();
        assert_eq!(ids, ["fast", "mid", "slow"]);
    }
}
