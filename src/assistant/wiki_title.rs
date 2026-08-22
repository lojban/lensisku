use serde::Deserialize;
use deadpool_postgres::Pool;
use tokio_postgres::Row;

use crate::comments::models::CommentContent;
use crate::error::AppError;
use crate::middleware::cache::RedisCache;
use crate::utils::remove_html_tags;

use super::dto::WikiTitleFromCommentResponse;
use super::openrouter::text_completion;

pub const WIKI_TITLE_MAX_LEN: usize = 80;

const PRIOR_COMMENTS_LIMIT: i64 = 15;
const PRIOR_COMMENT_BODY_MAX: usize = 500;
const ENTRY_DEFINITION_MAX: usize = 1000;
const TARGET_COMMENT_BODY_MAX: usize = 3000;

struct PriorCommentSnippet {
    comment_num: i32,
    username: Option<String>,
    subject: String,
    body: String,
}

struct CommentWikiTitleContext {
    comment_id: i32,
    comment_num: i32,
    subject: String,
    body: String,
    prior_comments: Vec<PriorCommentSnippet>,
    entry_valsi_word: Option<String>,
    entry_definition: Option<String>,
}

fn truncate_chars(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    text.chars().take(max_len).collect()
}

fn flatten_comment_content(content: &[CommentContent]) -> (String, String) {
    let subject = content
        .iter()
        .find(|part| part.r#type == "header")
        .map(|part| part.data.trim())
        .unwrap_or("")
        .to_string();
    let body = content
        .iter()
        .filter(|part| part.r#type == "text" && !part.data.trim().is_empty())
        .map(|part| part.data.trim().to_string())
        .collect::<Vec<_>>()
        .join("\n\n");
    (subject, body)
}

fn parse_comment_content(raw: &str) -> Vec<CommentContent> {
    serde_json::from_str(raw).unwrap_or_default()
}

fn heuristic_title(subject: &str, body: &str) -> String {
    let mut word = subject.trim().to_string();
    if word.is_empty() {
        let first_line = body
            .split('\n')
            .find(|line| !line.trim().is_empty())
            .unwrap_or("")
            .trim();
        word = first_line
            .trim_start_matches('#')
            .trim()
            .trim_start_matches('*')
            .trim()
            .trim_start_matches("- ")
            .trim()
            .trim_start_matches("* ")
            .trim()
            .to_string();
    }
    sanitize_wiki_title(&word)
}

pub fn sanitize_wiki_title(raw: &str) -> String {
    let mut title = raw
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim()
        .to_string();
    title = title.trim_start_matches('#').trim().to_string();
    title = title.trim_matches('"').trim_matches('\'').trim().to_string();
    truncate_chars(&title, WIKI_TITLE_MAX_LEN)
}

async fn load_comment_wiki_title_context(
    pool: &Pool,
    comment_id: i32,
) -> Result<CommentWikiTitleContext, AppError> {
    let client = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let target = client
        .query_opt(
            "SELECT c.commentid, c.commentnum, c.threadid, c.subject, c.content::text AS content,
                    t.valsiid, t.definitionid
             FROM convenientcomments c
             JOIN threads t ON t.threadid = c.threadid
             WHERE c.commentid = $1",
            &[&comment_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("comment not found".into()))?;

    let thread_id: i32 = target.get("threadid");
    let comment_num: i32 = target.get("commentnum");
    let content = parse_comment_content(&target.get::<_, String>("content"));
    let (subject, body) = flatten_comment_content(&content);

    let prior_rows = client
        .query(
            "SELECT c.commentnum, c.username, c.subject, c.content::text AS content
             FROM convenientcomments c
             WHERE c.threadid = $1 AND c.commentnum < $2
             ORDER BY c.commentnum ASC
             LIMIT $3",
            &[&thread_id, &comment_num, &PRIOR_COMMENTS_LIMIT],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let prior_comments = prior_rows
        .iter()
        .map(prior_comment_from_row)
        .collect();

    let valsi_id: Option<i32> = target.get("valsiid");
    let definition_id: Option<i32> = target.get("definitionid");
    let (entry_valsi_word, entry_definition) =
        load_entry_context(&client, valsi_id, definition_id).await?;

    Ok(CommentWikiTitleContext {
        comment_id,
        comment_num,
        subject,
        body,
        prior_comments,
        entry_valsi_word,
        entry_definition,
    })
}

fn prior_comment_from_row(row: &Row) -> PriorCommentSnippet {
    let content = parse_comment_content(&row.get::<_, String>("content"));
    let (subject, body) = flatten_comment_content(&content);
    PriorCommentSnippet {
        comment_num: row.get("commentnum"),
        username: row.get("username"),
        subject,
        body: truncate_chars(&body, PRIOR_COMMENT_BODY_MAX),
    }
}

async fn load_entry_context(
    client: &deadpool_postgres::Object,
    valsi_id: Option<i32>,
    definition_id: Option<i32>,
) -> Result<(Option<String>, Option<String>), AppError> {
    if let Some(definition_id) = definition_id.filter(|id| *id > 0) {
        if let Some(row) = client
            .query_opt(
                "SELECT d.definition, v.word
                 FROM definitions d
                 JOIN valsi v ON v.valsiid = d.valsiid
                 WHERE d.definitionid = $1",
                &[&definition_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let word: String = row.get("word");
            let definition: String = row.get("definition");
            let plain = truncate_chars(&remove_html_tags(&definition), ENTRY_DEFINITION_MAX);
            return Ok((Some(word), Some(plain)));
        }
    }

    if let Some(valsi_id) = valsi_id.filter(|id| *id > 0) {
        if let Some(row) = client
            .query_opt("SELECT word FROM valsi WHERE valsiid = $1", &[&valsi_id])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let word: String = row.get("word");
            return Ok((Some(word), None));
        }
    }

    Ok((None, None))
}

fn build_prompt(context: &CommentWikiTitleContext) -> String {
    let mut prompt = String::from(
        "Suggest a short wiki page title for a dictionary site discussion comment.\n\n",
    );

    if context.entry_valsi_word.is_some() || context.entry_definition.is_some() {
        prompt.push_str("Dictionary entry context:\n");
        if let Some(word) = &context.entry_valsi_word {
            prompt.push_str(&format!("- Entry word: {word}\n"));
        }
        if let Some(definition) = &context.entry_definition {
            prompt.push_str(&format!("- Entry definition: {definition}\n"));
        }
        prompt.push('\n');
    }

    if !context.prior_comments.is_empty() {
        prompt.push_str("Earlier comments in this thread:\n");
        for prior in &context.prior_comments {
            let author = prior
                .username
                .as_deref()
                .unwrap_or("unknown")
                .to_string();
            prompt.push_str(&format!("#{} by {}:\n", prior.comment_num, author));
            if !prior.subject.is_empty() {
                prompt.push_str(&format!("Subject: {}\n", prior.subject));
            }
            if !prior.body.is_empty() {
                prompt.push_str(&format!("{}\n", prior.body));
            }
            prompt.push('\n');
        }
    }

    prompt.push_str("Comment to turn into a wiki page:\n");
    prompt.push_str(&format!(
        "#{} (comment id {}):\n",
        context.comment_num, context.comment_id
    ));
    if !context.subject.is_empty() {
        prompt.push_str(&format!("Subject: {}\n", context.subject));
    }
    if !context.body.is_empty() {
        prompt.push_str(&truncate_chars(
            &context.body,
            TARGET_COMMENT_BODY_MAX,
        ));
        prompt.push('\n');
    }

    prompt
}

#[derive(Debug, Deserialize)]
struct TitleSuggestion {
    title: String,
}

async fn suggest_title_with_ai(
    context: &CommentWikiTitleContext,
    redis: Option<&RedisCache>,
) -> Result<Option<String>, AppError> {
    let system_prompt = format!(
        "You suggest short titles for wiki pages on a Lojban dictionary site. \
The user is turning a discussion comment into a wiki article. \
You receive the dictionary entry (if any), earlier thread comments for context, \
and the target comment body. \
Respond ONLY with valid JSON, no markdown, no explanation. \
Schema: {{\"title\": \"...\"}} \
Rules: \
- At most {max_len} characters \
- Plain text only (no markdown, no surrounding quotes) \
- Descriptive and specific to the comment in its thread context \
- Same language as the comment when possible \
- Do not use generic titles like \"Discussion\" or \"Comment\" \
- Summarize rather than copying a long subject line",
        max_len = WIKI_TITLE_MAX_LEN
    );

    let user_prompt = build_prompt(context);
    let raw = match text_completion(
        redis,
        &system_prompt,
        &user_prompt,
        "wiki title from comment",
        20,
    )
    .await
    {
        Some(raw) => raw,
        None => return Ok(None),
    };

    match serde_json::from_str::<TitleSuggestion>(&raw) {
        Ok(parsed) => {
            let title = sanitize_wiki_title(&parsed.title);
            if title.is_empty() {
                Ok(None)
            } else {
                Ok(Some(title))
            }
        }
        Err(e) => {
            log::warn!(
                "Assistant wiki title: failed to parse JSON: {} (raw: {})",
                e,
                raw
            );
            Ok(None)
        }
    }
}

pub async fn suggest_wiki_title_from_comment(
    pool: &Pool,
    redis: Option<&RedisCache>,
    comment_id: i32,
) -> Result<WikiTitleFromCommentResponse, AppError> {
    if comment_id <= 0 {
        return Err(AppError::BadRequest("comment_id must be positive".into()));
    }

    let context = load_comment_wiki_title_context(pool, comment_id).await?;
    let fallback = heuristic_title(&context.subject, &context.body);

    let title = match suggest_title_with_ai(&context, redis).await? {
        Some(title) if !title.is_empty() => title,
        _ => fallback,
    };

    if title.is_empty() {
        return Err(AppError::BadRequest(
            "Could not derive a wiki title from this comment".into(),
        ));
    }

    Ok(WikiTitleFromCommentResponse { title })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_wiki_title_strips_markdown_and_quotes() {
        assert_eq!(
            sanitize_wiki_title("  ##  \"My long wiki title\"  "),
            "My long wiki title"
        );
    }

    #[test]
    fn heuristic_title_uses_first_line_when_no_subject() {
        let title = heuristic_title("", "## Example\n\nBody text");
        assert_eq!(title, "Example");
    }
}
