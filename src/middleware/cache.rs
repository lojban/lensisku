use redis::{AsyncCommands, Client, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::jbovlaste::SearchDefinitionsQuery;

pub struct RedisCache {
    pub client: Client,
    default_ttl: Duration,
}

impl RedisCache {
    pub fn new(redis_url: &str, default_ttl: Duration) -> Result<Self, RedisError> {
        Ok(Self {
            client: Client::open(redis_url)?,
            default_ttl,
        })
    }

    pub async fn get_or_set<T, F, Fut>(
        &self,
        key: &str,
        fetch_data: F,
        ttl: Option<Duration>,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: DeserializeOwned + Serialize,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        if let Ok(data) = conn.get::<_, String>(key).await {
            if let Ok(parsed) = serde_json::from_str::<T>(&data) {
                return Ok(parsed);
            }
        }

        let data = fetch_data().await?;
        let serialized = serde_json::to_string(&data)?;

        let ttl = ttl.unwrap_or(self.default_ttl);
        let _: () = conn.set_ex(key, serialized, ttl.as_secs()).await?;

        Ok(data)
    }

    pub async fn invalidate(&self, pattern: &str) -> Result<(), RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let keys: Vec<String> = conn.keys(pattern).await?;

        if !keys.is_empty() {
            let _: i64 = conn.del(&keys).await?;
        }

        Ok(())
    }

    /// Invalidates all caches that may contain definition list/search results
    /// (keyword search, fast search, semantic search). Call when definitions
    /// or votes change so cached results stay correct.
    pub async fn invalidate_definition_search_caches(&self) -> Result<(), RedisError> {
        for pattern in &["search:*", "fast_search:*", "semantic_search:*"] {
            self.invalidate(pattern).await?;
        }
        Ok(())
    }

    /// Invalidates the recent changes page cache. Call when definitions,
    /// comments, threads, or votes change so the Recent Changes page is fresh.
    pub async fn invalidate_recent_changes(&self) -> Result<(), RedisError> {
        self.invalidate("recent_changes:*").await
    }
}

/// Cache key for definition search. `use_fast_search` must match the actual path
/// (fast vs full) so logged-in users don't get cached fast-search results with score 0.
pub fn generate_search_cache_key(query: &SearchDefinitionsQuery, use_fast_search: bool) -> String {
    let prefix = if use_fast_search {
        "fast_search"
    } else {
        "search"
    };
    format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        prefix,
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(20),
        query.search.as_deref().unwrap_or(""),
        query.sort_by.as_deref().unwrap_or("word"),
        query.sort_order.as_deref().unwrap_or("asc"),
        query.include_comments.unwrap_or(false),
        query.languages.as_deref().unwrap_or(""),
        query.selmaho.as_deref().unwrap_or(""),
        query.word_type.unwrap_or(0),
        query.username.as_deref().unwrap_or(""),
        query.source_langid.unwrap_or(1)
    )
}

pub fn generate_semantic_search_cache_key(query: &SearchDefinitionsQuery) -> String {
    // Key includes search term and all filter/pagination options relevant to semantic search
    format!(
        "semantic_search:{}:{}:{}:{}:{}:{}:{}:{}",
        query.search.as_deref().unwrap_or(""),
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(20),
        query.languages.as_deref().unwrap_or(""),
        query.selmaho.as_deref().unwrap_or(""),
        query.username.as_deref().unwrap_or(""),
        query.word_type.unwrap_or(0),
        query.source_langid.unwrap_or(1) // Note: sort_by and sort_order are fixed to 'similarity asc' for semantic search
                                         // Note: include_comments is fixed to false for semantic search
    )
}
