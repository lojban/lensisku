use reqwest::Client;
use std::{env, time::Duration};

const PURGE_ENDPOINT: &str = "https://api.cloudflare.com/client/v4/zones";

#[derive(Clone)]
pub struct CloudflareCachePurger {
    client: Option<Client>,
    api_token: Option<String>,
    zone_id: Option<String>,
}

impl CloudflareCachePurger {
    pub fn from_env() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .ok();

        Self {
            client,
            api_token: env::var("CLOUDFLARE_API_TOKEN")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            zone_id: env::var("CLOUDFLARE_ZONE_ID")
                .ok()
                .filter(|value| !value.trim().is_empty()),
        }
    }

    /// Purge cached URL prefixes. Missing credentials and Cloudflare errors are
    /// deliberately ignored so cache maintenance never fails a user write.
    pub async fn purge_tags(&self, tags: impl IntoIterator<Item = String>) {
        let (Some(client), Some(api_token), Some(zone_id)) =
            (&self.client, &self.api_token, &self.zone_id)
        else {
            return;
        };

        let tags: Vec<String> = tags
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        if tags.is_empty() {
            return;
        }

        let url = format!("{PURGE_ENDPOINT}/{zone_id}/purge_cache");
        let _ = client
            .post(url)
            .bearer_auth(api_token)
            .json(&serde_json::json!({ "tags": tags }))
            .send()
            .await;
    }

    pub async fn purge_definition(
        &self,
        definition_id: i32,
        valsi_id: Option<i32>,
        word: Option<&str>,
    ) {
        let mut tags = vec![
            "definitions".to_string(),
            "dictionary-search".to_string(),
            "recent-changes".to_string(),
            format!("definition:{definition_id}"),
        ];
        if let Some(valsi_id) = valsi_id {
            tags.push(format!("valsi:{valsi_id}"));
        }
        if let Some(word) = word.filter(|word| !word.is_empty()) {
            tags.push(valsi_tag(word));
        }
        self.purge_tags(tags).await;
    }

    pub async fn purge_comment(
        &self,
        valsi_id: Option<i32>,
        definition_id: Option<i32>,
        word: Option<&str>,
    ) {
        let mut tags = vec![
            "comments".to_string(),
            "comment-lists".to_string(),
            "comment-aggregates".to_string(),
            "recent-changes".to_string(),
            "definitions".to_string(),
            "dictionary-search".to_string(),
        ];
        if let Some(definition_id) = definition_id {
            tags.push(format!("definition:{definition_id}"));
        }
        if let Some(valsi_id) = valsi_id {
            tags.push(format!("valsi:{valsi_id}"));
        }
        if let Some(word) = word.filter(|word| !word.is_empty()) {
            tags.push(valsi_tag(word));
        }
        self.purge_tags(tags).await;
    }
}

pub fn tags_for_get(path: &str, query: Option<&str>) -> Vec<String> {
    let mut tags = Vec::new();
    if path.starts_with("/jbovlaste/") {
        tags.push("definitions".into());
    }
    if path.starts_with("/comments/") {
        tags.push("comments".into());
    }
    if let Some(rest) = path.strip_prefix("/jbovlaste/valsi/") {
        if let Some(identifier) = rest.split('/').next() {
            tags.push(valsi_tag(identifier));
        }
    }

    if let Some(id) = path.strip_prefix("/jbovlaste/definition/") {
        if let Some(id) = id.split('/').next().and_then(parse_positive_id) {
            tags.push(format!("definition:{id}"));
        }
    }
    if let Some(id) = path
        .strip_prefix("/jbovlaste/valsi/wiki/by-definition/")
        .and_then(parse_positive_id)
    {
        tags.push(format!("definition:{id}"));
    }
    if let Some(rest) = path.strip_prefix("/jbovlaste/definitions/") {
        if let Some(id) = rest.split('/').next().and_then(parse_positive_id) {
            tags.push(format!("definition:{id}"));
        }
    }
    if let Some(id) = path.strip_prefix("/versions/").and_then(|s| s.strip_suffix("/history"))
        .and_then(parse_positive_id)
    {
        tags.push(format!("definition:{id}"));
    }
    if let Some(id) = path
        .strip_prefix("/jbovlaste/vote/")
        .and_then(parse_positive_id)
    {
        tags.push(format!("definition:{id}"));
    }
    if let Some(id) = path
        .strip_prefix("/jbovlaste/definition-links/")
        .and_then(parse_positive_id)
    {
        tags.push(format!("definition:{id}"));
    }

    match path {
        "/jbovlaste/definitions" | "/jbovlaste/definitions/list"
        | "/jbovlaste/non-lojban-definitions" => tags.push("definitions".into()),
        "/jbovlaste/semantic-search" | "/jbovlaste/semantic-graph" => {
            tags.push("dictionary-search".into())
        }
        "/jbovlaste/changes" => tags.push("recent-changes".into()),
        "/comments/thread" => {
            tags.push("comments".into());
            for key in ["definition_id", "valsi_id", "thread_id"] {
                if let Some(id) = query_value(query, key)
                    .as_deref()
                    .and_then(parse_positive_id)
                {
                    let tag = match key {
                        "definition_id" => format!("definition:{id}"),
                        "valsi_id" => format!("valsi:{id}"),
                        _ => format!("comment-thread:{id}"),
                    };
                    tags.push(tag);
                }
            }
        }
        "/comments/list" => tags.push("comment-lists".into()),
        "/comments/top" | "/comments/trending" | "/comments/most-bookmarked" => {
            tags.push("comment-aggregates".into())
        }
        _ => {}
    }
    tags.sort();
    tags.dedup();
    tags
}

fn query_value<'a>(query: Option<&'a str>, key: &str) -> Option<String> {
    query?
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(candidate, _)| *candidate == key)
        .map(|(_, value)| value.to_string())
}

fn parse_positive_id(value: &str) -> Option<i32> {
    value.parse::<i32>().ok().filter(|id| *id > 0)
}

fn valsi_tag(valsi: &str) -> String {
    if let Some(id) = parse_positive_id(valsi) {
        format!("valsi:{id}")
    } else {
        format!("valsi:{}", valsi.replace(' ', "_").to_lowercase())
    }
}

