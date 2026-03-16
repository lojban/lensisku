#![allow(clippy::expect_used)]

use crate::mailarchive::{
    dto::MailThreadSummary, Message, SearchQuery, SearchResponse, ThreadQuery, ThreadResponse,
};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use deadpool_postgres::Pool;
use encoding_rs::{GB18030, KOI8_R, WINDOWS_1252};
use mailparse::{parse_mail, MailHeaderMap};
use std::collections::HashSet;
use std::path::Path;
use std::{fs, path::PathBuf};
use tokio_postgres::Client;
use walkdir::WalkDir;

use lazy_static::lazy_static;
use log::{error, info, warn};
use regex::Regex;
use tokio::time::{sleep, Duration};

lazy_static! {
    static ref HEADER_SPLIT_REGEX: Regex =
        Regex::new(r"[;\n]|\\n").expect("Invalid header split regex pattern");
    static ref PARENTHESES_CLEAN_REGEX: Regex =
        Regex::new(r"\([^)]*\)").expect("Invalid parentheses cleanup regex pattern");
    static ref DAY_OF_WEEK_REGEX: Regex = Regex::new(r"^(?:Mon|Tue|Wed|Thu|Fri|Sat|Sun)\s+")
        .expect("Invalid day of week regex pattern");
}

const BATCH_SIZE: usize = 1000;
const BATCH_DELAY: Duration = Duration::from_millis(100);

fn escape_like<S: AsRef<str>>(s: S) -> String {
    s.as_ref()
        .replace("\\", "\\\\")
        .replace("%", "\\%")
        .replace("_", "\\_")
        .replace("'", "''")
}

pub async fn search_messages(
    pool: &Pool,
    query: SearchQuery,
) -> Result<SearchResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;
    let group_by_thread = query.group_by_thread.unwrap_or(false);
    // Split query into words and remove empty ones
    let words: Vec<String> = query
        .query
        .split_whitespace()
        .map(|w| w.to_string())
        .collect();

    let exact_query = format!("%{}%", escape_like(&query.query));
    let word_conditions_sql_parts: Vec<String> = words
        .iter()
        .map(|word| {
            let escaped_word = escape_like(word);
            format!(
                "(m.subject ILIKE '%{}%' OR m.content ILIKE '%{}%')",
                escaped_word, escaped_word
            )
        })
        .collect();

    let main_where_conditions_sql = if word_conditions_sql_parts.is_empty() {
        "TRUE".to_string()
    } else {
        word_conditions_sql_parts.join(" AND ")
    };

    let rank_word_conditions_sql = if word_conditions_sql_parts.is_empty() {
        "FALSE".to_string() // Ensures this part of rank is 0 if no specific words
    } else {
        word_conditions_sql_parts.join(" AND ")
    };

    // Validate sort_by against allowed fields for the outer query
    let outer_sort_column_name = match query.sort_by.as_deref() {
        Some("date") => "date", // Refers to date/sent_at of the representative message
        Some("subject") => "subject", // Refers to cleaned_subject of the representative message
        Some("sent_at") => "sent_at", // Refers to sent_at of the representative message
        _ => "rank",            // Default to rank of the representative message
    };

    let sort_order = match query.sort_order.as_deref() {
        Some(s) if s.eq_ignore_ascii_case("asc") => "ASC",
        _ => "DESC",
    };
    let include_content = query.include_content.unwrap_or(true);
    let content_select = if include_content {
        "parts_json"
    } else {
        "NULL::jsonb as parts_json"
    };

    let query_string;
    let count_query_string;

    if group_by_thread {
        query_string = format!(
            "WITH thread_representatives AS (
                SELECT DISTINCT ON (m.cleaned_subject)
                       m.id, m.message_id, m.date, m.cleaned_subject, m.from_address, m.to_address, m.parts_json, m.sent_at,
                       (SELECT COUNT(*) FROM message_spam_votes msv WHERE msv.message_id = m.id) as spam_vote_count,
                       (CASE
                          WHEN m.subject ILIKE $1 THEN 3
                          WHEN m.content ILIKE $1 THEN 2
                          WHEN {} THEN 1
                          ELSE 0
                        END) as rank
                FROM messages m
                WHERE {}
                ORDER BY m.cleaned_subject,
                         (CASE WHEN m.subject ILIKE $1 THEN 3 WHEN m.content ILIKE $1 THEN 2 WHEN {} THEN 1 ELSE 0 END) DESC,
                         m.sent_at DESC NULLS LAST, m.date DESC NULLS LAST
            )
            SELECT id, message_id, date, cleaned_subject as subject, from_address, to_address, 
                   CASE WHEN {} THEN parts_json ELSE NULL::jsonb END as parts_json,
                   spam_vote_count, rank, sent_at
            FROM thread_representatives
            ORDER BY {} {}, date {}
            LIMIT $2 OFFSET $3",
            rank_word_conditions_sql, // for rank in CTE
            main_where_conditions_sql,  // for WHERE in CTE
            rank_word_conditions_sql, // for ORDER BY in CTE (rank part)
            if include_content { "TRUE" } else { "FALSE" }, // for parts_json selection
            outer_sort_column_name,
            sort_order,
            sort_order
        );
        count_query_string = format!(
            "SELECT COUNT(*) FROM (
                SELECT DISTINCT ON (m.cleaned_subject) 1
                FROM messages m
                WHERE {}
            ) AS distinct_threads",
            main_where_conditions_sql
        );
    } else {
        query_string = format!(
            "SELECT m.id, m.message_id, m.date, m.subject, m.cleaned_subject, m.from_address, m.to_address, {}, m.sent_at,
             (SELECT COUNT(*) FROM message_spam_votes msv WHERE msv.message_id = m.id) as spam_vote_count,
             (CASE
                WHEN m.subject ILIKE $1 THEN 3
                WHEN m.content ILIKE $1 THEN 2
                WHEN {} THEN 1
                ELSE 0
              END) as rank
             FROM messages m
             WHERE {}
             ORDER BY {} {}, m.date {}
             LIMIT $2 OFFSET $3",
            content_select,
            rank_word_conditions_sql,
            main_where_conditions_sql,
            outer_sort_column_name, // Here m. prefix might be needed if not aliasing in CTE
            sort_order,
            sort_order
        );
        count_query_string = format!(
            "SELECT COUNT(*) FROM messages m WHERE {}",
            main_where_conditions_sql
        );
    }

    let messages = transaction
        .query(&query_string, &[&exact_query, &per_page, &offset])
        .await?
        .into_iter()
        .map(Message::from)
        .collect::<Vec<_>>();

    let total: i64 = transaction
        .query_one(
            &count_query_string,
            &[] as &[&(dyn tokio_postgres::types::ToSql + Sync)],
        )
        .await?
        .get(0);

    transaction.commit().await?;

    Ok(SearchResponse {
        messages,
        total,
        page,
        per_page,
    })
}

pub async fn get_message(
    pool: &Pool,
    id: i32,
    user_id: Option<i32>,
) -> Result<Option<Message>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let message = client
        .query_opt(
            "SELECT id, message_id, date, subject, cleaned_subject, from_address, to_address, parts_json, file_path,
             (SELECT COUNT(*) FROM message_spam_votes msv_count WHERE msv_count.message_id = messages.id) as spam_vote_count,
             CASE WHEN $2::INT IS NOT NULL THEN EXISTS (SELECT 1 FROM message_spam_votes msv_user WHERE msv_user.message_id = messages.id AND msv_user.user_id = $2) ELSE NULL END as current_user_voted_spam
             FROM messages
             WHERE id = $1",
            &[&id, &user_id],
        )
        .await?
        .map(Message::from);

    Ok(message)
}

pub async fn show_thread(
    pool: &Pool,
    query: ThreadQuery,
) -> Result<ThreadResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;
    let sort_order = match query.sort_order.as_deref() {
        Some(s) if s.eq_ignore_ascii_case("asc") => "ASC", // Make case-insensitive
        _ => "DESC",
    };
    let include_content = query.include_content.unwrap_or(true);

    let sort_column = match query.sort_by.as_deref() {
        Some("subject") => "m.subject",
        Some("sent_at") => "m.sent_at",
        _ => "m.date", // Default to "m.date" (which implies m.sent_at or m.date from DB)
    };
    // Remove common prefixes and tags from the subject and escape special characters
    let clean_subject = remove_prefixes(&query.subject);

    let query_string = if include_content {
        format!(
            "SELECT m.id, m.message_id, m.date, m.subject, m.from_address, m.to_address, m.parts_json,
             (SELECT COUNT(*) FROM message_spam_votes msv WHERE msv.message_id = m.id) as spam_vote_count
             FROM messages m
             WHERE m.cleaned_subject = $1
             ORDER BY {} {}, date {}
             LIMIT $2 OFFSET $3",
            sort_column, sort_order, sort_order
        )
    } else {
        format!(
            "SELECT m.id, m.message_id, m.date, m.subject, m.from_address, m.to_address, NULL::jsonb as parts_json,
             (SELECT COUNT(*) FROM message_spam_votes msv WHERE msv.message_id = m.id) as spam_vote_count
             FROM messages m
             WHERE m.cleaned_subject = $1
             ORDER BY {} {}, date {}
             LIMIT $2 OFFSET $3",
            sort_column, sort_order, sort_order
        )
    };
    let messages = transaction
        .query(&query_string, &[&clean_subject, &per_page, &offset])
        .await?
        .into_iter()
        .map(Message::from)
        .collect::<Vec<_>>();

    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM messages WHERE cleaned_subject = $1",
            &[&clean_subject],
        )
        .await?
        .get(0);

    // Commit the transaction
    transaction.commit().await?;

    Ok(ThreadResponse {
        messages,
        total,
        page,
        per_page,
        clean_subject,
    })
}

/// List mail threads (grouped by cleaned_subject) for the waves unified list.
pub async fn list_mail_threads(
    pool: &Pool,
    page: i64,
    per_page: i64,
    sort_order: &str,
) -> Result<(Vec<MailThreadSummary>, i64), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    let offset = (page - 1) * per_page;
    let order_dir = if sort_order.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };

    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(DISTINCT cleaned_subject) FROM messages WHERE cleaned_subject IS NOT NULL AND cleaned_subject != ''",
            &[],
        )
        .await?
        .get(0);

    let rows = transaction
        .query(
            &format!(
                r#"
                WITH latest AS (
                    SELECT DISTINCT ON (cleaned_subject)
                        id, cleaned_subject, subject, from_address, sent_at,
                        LEFT(content, 300) as content_preview
                    FROM messages
                    WHERE cleaned_subject IS NOT NULL AND cleaned_subject != ''
                    ORDER BY cleaned_subject, sent_at DESC NULLS LAST
                ),
                counts AS (
                    SELECT cleaned_subject, COUNT(*) as cnt
                    FROM messages
                    WHERE cleaned_subject IS NOT NULL AND cleaned_subject != ''
                    GROUP BY cleaned_subject
                )
                SELECT l.cleaned_subject, l.subject, l.from_address, l.sent_at, c.cnt as message_count, l.content_preview
                FROM latest l
                JOIN counts c ON c.cleaned_subject = l.cleaned_subject
                ORDER BY l.sent_at {} NULLS LAST
                LIMIT $1 OFFSET $2
                "#,
                order_dir
            ),
            &[&per_page, &offset],
        )
        .await?;

    let items: Vec<MailThreadSummary> = rows
        .iter()
        .map(|row| MailThreadSummary {
            cleaned_subject: row.get("cleaned_subject"),
            subject: row.get("subject"),
            from_address: row.get("from_address"),
            last_sent_at: row.get("sent_at"),
            message_count: row.get::<_, i64>("message_count"),
            content_preview: row.get("content_preview"),
        })
        .collect();

    transaction.commit().await?;
    Ok((items, total))
}

fn remove_prefixes(subject: &str) -> String {
    let mut clean_subject = subject.to_string();
    let mut modified = true;

    while modified {
        modified = false;
        let trimmed = clean_subject.trim();

        if trimmed.to_lowercase().starts_with("re:") {
            clean_subject = trimmed[3..].trim().to_string();
            modified = true;
            continue;
        }

        if let Some(start) = trimmed.find('[') {
            if let Some(end) = trimmed[start..].find(']') {
                clean_subject = trimmed[..start].to_string() + trimmed[start + end + 1..].trim();
                modified = true;
                continue;
            }
        }
    }

    clean_subject.trim().to_string()
}

pub async fn import_maildir(
    pool: &deadpool_postgres::Pool,
    maildir_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let maildir = Path::new(maildir_path);
    let mut email_paths = Vec::new();

    // Collect all file paths first
    for entry in WalkDir::new(maildir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            email_paths.push(entry.path().to_path_buf());
        }
    }

    info!("Found {} emails in maildir", email_paths.len());

    // Process in batches
    for chunk in email_paths.chunks(BATCH_SIZE) {
        let mut tasks = Vec::new();

        // Pre-check existence for all files in this batch
        let client = pool.get().await.expect("Failed to get database connection");
        let relative_paths: Vec<String> = chunk
            .iter()
            .filter_map(|path| {
                path.strip_prefix(maildir)
                    .ok()
                    .and_then(|p| p.to_str())
                    .map(|s| s.to_string())
            })
            .collect();

        let query = "SELECT file_path FROM messages WHERE file_path = ANY($1::text[])";
        let existing_paths: HashSet<String> = client
            .query(query, &[&relative_paths])
            .await?
            .iter()
            .map(|row| row.get::<_, String>("file_path"))
            .collect();

        // Process each non-existing email in the chunk
        for file_path in chunk {
            let pool = pool.clone();
            let file_path = file_path.clone();
            let maildir_path = maildir_path.to_string();

            let relative_path = match file_path.strip_prefix(&maildir_path) {
                Ok(path) => path.to_str().unwrap_or_default().to_string(),
                Err(_) => continue,
            };

            // Skip if email already exists
            if existing_paths.contains(&relative_path) {
                continue;
            }

            // Spawn a task for each new email
            let task = tokio::spawn(async move {
                let client = match pool.get().await {
                    Ok(client) => client,
                    Err(e) => {
                        error!("Failed to get database connection: {}", e);
                        return;
                    }
                };
                if let Err(e) = process_email(&client, &file_path, &maildir_path).await {
                    warn!("Error processing email {}: {}", file_path.display(), e);
                }
            });

            tasks.push(task);
        }

        // Wait for all tasks in the batch to complete
        for task in tasks {
            if let Err(e) = task.await {
                warn!("Task failed: {}", e);
            }
        }

        // Add delay between batches
        sleep(BATCH_DELAY).await;
    }

    Ok(())
}

async fn process_email(
    client: &Client,
    file_path: &Path,
    maildir_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read(file_path)?;

    // Try to parse the email with raw content first to get headers
    let content_str = &String::from_utf8_lossy(&content)
        .lines()
        .collect::<Vec<_>>()
        .join("\r\n")
        // .replace('\0', "") // Remove null bytes
        // .replace(|c: char| c.is_control() && c != '\n' && c != '\r', "") // Remove other control chars
        .to_string();

    let index_headers_end = content_str.find("\r\n\r\n").unwrap_or(content_str.len());
    let headers: &str = &content_str[..index_headers_end];

    let headers_str = normalize_email_headers(headers)?;

    // Parse email using processed headers
    let message_body_str = content_str[index_headers_end..].trim();
    let mail_content = format!("{}\r\n\r\n{}", headers_str, &message_body_str);

    let parsed_mail = parse_mail(mail_content.as_bytes())?;

    // Process content based on encoding and charset
    let has_encoding = parsed_mail
        .headers
        .get_first_value("Content-Transfer-Encoding")
        .is_some();
    // Common logic for extracting body content after headers
    let body_lines: Vec<_> = content_str
        .lines()
        .skip_while(|line| !line.is_empty()) // Skip headers
        .skip(1) // Skip empty line after headers
        .collect();
    let joined_body = body_lines.join("\r\n");
    let get_body_content = joined_body.trim();

    let message_body_str = if has_encoding {
        // If encoded, take first part after headers split by blank line
        get_body_content
            .split("\r\n\r\n")
            .next()
            .unwrap_or("")
            .trim()
    } else {
        // Original processing for non-encoded content
        get_body_content
    };

    // Parse email using processed content
    let mail_content = format!("{}\r\n\r\n{}", headers_str, &message_body_str);
    let parsed_mail = parse_mail(mail_content.as_bytes())?;

    // Process all parts of the email
    let parts = collect_parts(&parsed_mail, file_path.to_str().unwrap_or_default());

    let message_id = parsed_mail
        .headers
        .get_first_value("Message-ID")
        .unwrap_or_default();
    let received_date = parsed_mail
        .headers
        .get_first_value("Received")
        .and_then(|received| parse_header_date(&received));

    let x_space_date = parsed_mail
        .headers
        .get_first_value("X-From-Space-Date")
        .and_then(|date| parse_header_date(&date));
    let date = parsed_mail
        .headers
        .get_first_value("Date")
        .unwrap_or_default();
    let fixed_date = fix_timezone_abbreviation(&date)?;
    let parsed_date = parse_email_date(&fixed_date)
        .or_else(|_| {
            received_date.ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to parse date from Received header",
                ))
            })
        })
        .or_else(|_| {
            x_space_date.ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to parse date from X-From-Space-Date header",
                ))
            })
        })
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| {
            warn!(
                "Failed to parse date: '{}' and received date '{:#?}'. Using current time. File: {:#?}",
                fixed_date, received_date, file_path
            );

            Utc::now()
        })
        .to_rfc3339();
    let subject = parsed_mail
        .headers
        .get_first_value("Subject")
        .unwrap_or_default();
    let from_address = parsed_mail
        .headers
        .get_first_value("From")
        .unwrap_or_default();
    let to_address = parsed_mail
        .headers
        .get_first_value("To")
        .unwrap_or_default();

    let relative_path = file_path
        .strip_prefix(maildir_path)?
        .to_str()
        .unwrap_or_default();

    // Extract plain text content from text/plain parts
    let mut plain_text_content_parts = Vec::new();
    for part_json_value in &parts {
        if let Some(mime_type) = part_json_value.get("mime_type").and_then(|m| m.as_str()) {
            if mime_type == "text/plain" {
                if let Some(content_str) = part_json_value.get("content").and_then(|c| c.as_str()) {
                    plain_text_content_parts.push(content_str.trim());
                }
            }
        }
    }
    let plain_text_content = plain_text_content_parts.join(" ").trim().to_string();

    let parts_json_value = serde_json::json!(parts);

    client.execute(
        "INSERT INTO messages (message_id, date, subject, from_address, to_address, file_path, parts_json, content)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (file_path) DO NOTHING",
        &[&message_id, &parsed_date, &subject, &from_address, &to_address, &relative_path, &parts_json_value, &plain_text_content],
    ).await?;

    Ok(())
}

/// Processes all MIME parts of an email and returns structured data
fn collect_parts(parsed_mail: &mailparse::ParsedMail, filepath: &str) -> Vec<serde_json::Value> {
    let mut parts = Vec::new();
    let mut seen = std::collections::HashSet::new();
    collect_parts_recursive(parsed_mail, &mut parts, &mut seen, filepath);
    parts
}

fn collect_parts_recursive(
    parsed_mail: &mailparse::ParsedMail,
    parts: &mut Vec<serde_json::Value>,
    seen: &mut std::collections::HashSet<String>,
    filepath: &str,
) {
    // Skip processing if raw body content matches the "not displayed" message
    let raw_body = parsed_mail.raw_bytes;
    let body_str = String::from_utf8_lossy(raw_body).trim().to_string();

    let index_headers_end = body_str.find("\r\n\r\n").unwrap_or(body_str.len());
    let body_str = body_str[index_headers_end..].trim();

    if body_str == "[Attachment content not displayed.]" {
        return;
    }

    // For each part except root, normalize headers and recreate the part
    let (mime_type, content_type) = parsed_mail
        .headers
        .get_first_value("Content-Type")
        .map(|ct| parse_content_type(&ct))
        .unwrap_or_else(|| ("text/plain".to_string(), "text/plain".to_string()));

    match parsed_mail.get_body() {
        Ok(body_bytes) => {
            process_mail_body(
                parsed_mail,
                parts,
                body_bytes,
                seen,
                mime_type,
                content_type,
            );
        }
        Err(_) => {
            match parsed_mail.get_body_raw() {
                Ok(body_bytes) => {
                    // Clone body_bytes before moving it
                    let body_bytes_clone = body_bytes.clone();
                    if let Ok(body) = String::from_utf8(body_bytes) {
                        process_mail_body(parsed_mail, parts, body, seen, mime_type, content_type);
                    } else {
                        // Fallback processing for failed UTF-8 conversion
                        let raw_body = String::from_utf8_lossy(&body_bytes_clone);
                        let first_part = if parsed_mail
                            .headers
                            .get_first_value("Content-Transfer-Encoding")
                            .is_some()
                        {
                            raw_body.split("\r\n\r\n").next().unwrap_or("").trim()
                        } else {
                            raw_body.trim()
                        };
                        process_mail_body(
                            parsed_mail,
                            parts,
                            first_part.to_string(),
                            seen,
                            mime_type,
                            content_type,
                        );
                    }
                }
                Err(e) => {
                    // Fallback processing for failed body parsing
                    warn!("Failed processing {} {:?}", filepath, e);
                }
            }
        }
    }

    // Process subparts of the recreated part recursively
    for subpart in &parsed_mail.subparts {
        collect_parts_recursive(subpart, parts, seen, filepath);
    }
}

fn process_mail_body(
    parsed_mail: &mailparse::ParsedMail,
    parts: &mut Vec<serde_json::Value>,
    body: String,
    seen: &mut std::collections::HashSet<String>,
    mime_type: String,
    content_type: String,
) {
    // Handle text vs binary content differently
    let content = if mime_type.starts_with("text/") {
        // Convert text content to UTF-8
        convert_to_utf8(&body, &parsed_mail.get_headers())
    } else {
        // For binary content, store as base64 encoded string
        parsed_mail
            .get_body_raw()
            .map(|body| STANDARD.encode(&body))
            .ok()
    };

    if let Some(content_data) = content {
        // Extract filename from headers
        let content_disp = parsed_mail
            .headers
            .get_first_value("Content-Disposition")
            .unwrap_or_default();
        let parsed_disp = mailparse::parse_content_disposition(&content_disp);
        let disp_filename = parsed_disp.params.get("filename").cloned();

        let content_type_header = parsed_mail
            .headers
            .get_first_value("Content-Type")
            .unwrap_or_default();
        let parsed_ct = mailparse::parse_content_type(&content_type_header);
        let ct_filename = parsed_ct.params.get("name").cloned();

        let filename = disp_filename.or(ct_filename).unwrap_or_default();

        let content_id = parsed_mail
            .headers
            .get_first_value("Content-ID")
            .map(|cid| cid.trim_matches(|c| c == '<' || c == '>').to_string());

        let content_data = content_data
            .replace('\0', "") // Remove null bytes
            .replace(|c: char| c.is_control() && c != '\n' && c != '\r', ""); // Remove other control chars

        let parts_json = serde_json::json!({
            "mime_type": mime_type.clone(),
            "content": content_data,
            "content_type": content_type.clone(),
            "is_base64": !mime_type.starts_with("text/"),  // More accurate check
            "filename": filename,
            "content_id": content_id
        });
        let serialized = parts_json.to_string();
        if !seen.contains(&serialized) {
            seen.insert(serialized);
            parts.push(parts_json);
        }
    }
}

/// Converts text content to UTF-8 using proper encoding
fn convert_to_utf8(content: &str, headers: &mailparse::headers::Headers) -> Option<String> {
    let charset = headers
        .get_first_value("Content-Type")
        .and_then(|ct| ct.split("charset=").nth(1).map(|s| s.to_string()))
        .and_then(|s| s.split(';').next().map(|s| s.to_string()))
        .unwrap_or_else(|| "utf-8".to_string())
        .trim()
        .to_lowercase();

    match charset.as_str() {
        "koi8-r" => KOI8_R.decode(content.as_bytes()).0.into_owned().into(),
        "gb2312" | "gbk" => GB18030.decode(content.as_bytes()).0.into_owned().into(),
        "windows-1252" | "iso-8859-1" => WINDOWS_1252
            .decode(content.as_bytes())
            .0
            .into_owned()
            .into(),
        "utf-8" => Some(content.to_string()),
        _ => {
            // Fallback to UTF-8 with replacement characters
            String::from_utf8_lossy(content.as_bytes())
                .into_owned()
                .into()
        }
    }
}

/// Normalizes email headers by replacing tabs and spaces according to email format conventions
fn normalize_email_headers(headers: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Create regexes for each replacement
    let re_leading = Regex::new(r"(?m)(\r\n)[\t ]+")?; // Merge continuation lines starting with tab/space
    let re_remaining_tabs = Regex::new(r"\t")?; // Replace remaining tabs

    // Perform all replacements
    let headers = re_leading
        .replace_all(headers, " ") // Merge continuation lines into previous line
        .to_string(); // Convert Cow<str> to String for further processing

    let headers = re_remaining_tabs
        .replace_all(&headers, " ") // Replace remaining tabs with "\n "
        .to_string();

    // re_eight_or_more_spaces
    //     .replace_all(&headers, "\r\n ") // Replace 8 or more spaces with "\n "
    //     .to_string()
    Ok(headers)
}

/// Parses Content-Type header into (mime_type, content_type)
fn parse_content_type(header: &str) -> (String, String) {
    let parsed = mailparse::parse_content_type(header);
    let mime_type = parsed.mimetype.clone();
    let content_type = parsed
        .params
        .get("type")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "binary".to_string());

    (mime_type, content_type)
}

fn parse_email_date(date_str: &str) -> Result<DateTime<FixedOffset>, Box<dyn std::error::Error>> {
    // Try removing the leading day abbreviation if present
    if let Some(mat) = DAY_OF_WEEK_REGEX.find(date_str) {
        let remaining_str = &date_str[mat.end()..];
        // Attempt parsing the remaining string (without day name)
        // Try with timezone name first (%Z)
        if let Ok(dt) = DateTime::parse_from_str(remaining_str, "%b %e %H:%M:%S %Y %Z") {
            return Ok(dt);
        }
        // Try with numeric offset (%z) as fallback
        if let Ok(dt) = DateTime::parse_from_str(remaining_str, "%b %e %H:%M:%S %Y %z") {
            return Ok(dt);
        }
        // If parsing the modified string fails, we'll fall through to the original logic below
    }

    // Original parsing attempts (keep them as fallbacks)
    DateTime::parse_from_rfc2822(date_str)
        .or_else(|_| DateTime::parse_from_str(date_str, "%a, %d %b %y %H:%M:%S %z"))
        .or_else(|_| DateTime::parse_from_str(date_str, "%d %b %y %H:%M:%S %z"))
        .or_else(|_| DateTime::parse_from_str(date_str, "%a, %d %b %Y %H:%M:%S %z"))
        .or_else(|_| DateTime::parse_from_str(date_str, "%a, %e %b %Y %H:%M:%S %z"))
        .or_else(|_| DateTime::parse_from_str(date_str, "%a %b %e %H:%M:%S %Y %z")) // Handle space-padded day with timezone
        .or_else(|_| DateTime::parse_from_str(date_str, "%b %e %H:%M:%S %Y %z")) // Handle without day of week
        .or_else(|_| {
            NaiveDateTime::parse_from_str(date_str, "%a, %d %b %Y %H:%M:%S")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .or_else(|_| {
            NaiveDateTime::parse_from_str(date_str, "%a, %d %b %y %H:%M:%S")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .or_else(|_| {
            NaiveDateTime::parse_from_str(date_str, "%a, %b %e %H:%M:%S %Y")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .or_else(|_| {
            NaiveDateTime::parse_from_str(date_str, "%a %b %d %H:%M:%S %Y")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .or_else(|_| {
            NaiveDateTime::parse_from_str(date_str, "%a %b %e %H:%M:%S %Y")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .or_else(|_| {
            NaiveDateTime::parse_from_str(date_str, "%a %b %e %H:%M:%S")
                .map_err(|e| e.into())
                .and_then(|ndt| {
                    // Assume EST if no timezone specified
                    let est_offset = FixedOffset::west_opt(5 * 3600)
                        .ok_or_else(|| Box::<dyn std::error::Error>::from("Invalid EST offset"))?; // UTC-5
                    match ndt.and_local_timezone(est_offset) {
                        chrono::LocalResult::Single(dt) => Ok(dt),
                        chrono::LocalResult::Ambiguous(dt1, _) => Ok(dt1), // Choose earliest in case of ambiguity
                        chrono::LocalResult::None => Err(Box::<dyn std::error::Error>::from(
                            "Invalid local time for EST",
                        )),
                    }
                })
        })
        .or_else(|_| {
            NaiveDateTime::parse_from_str(date_str, "%a %b %e %H:%M:%S")
                .map_err(|e| e.into())
                .and_then(|ndt| {
                    // Try PST as final fallback (UTC-8)
                    let pst_offset = FixedOffset::west_opt(8 * 3600)
                        .ok_or_else(|| Box::<dyn std::error::Error>::from("Invalid PST offset"))?; // UTC-8
                    match ndt.and_local_timezone(pst_offset) {
                        chrono::LocalResult::Single(dt) => Ok(dt),
                        chrono::LocalResult::Ambiguous(dt1, _) => Ok(dt1), // Choose earliest in case of ambiguity
                        chrono::LocalResult::None => Err(Box::<dyn std::error::Error>::from(
                            "Invalid local time for PST",
                        )),
                    }
                })
        })
}

fn parse_header_date(header_value: &str) -> Option<DateTime<FixedOffset>> {
    let parts: Vec<&str> = HEADER_SPLIT_REGEX.split(header_value).collect();

    parts
        .iter()
        .rev() // Try newest first (reverse order)
        .find_map(|part| {
            // Clean each part
            let cleaned = PARENTHESES_CLEAN_REGEX
                .replace_all(part, "")
                .trim()
                .to_string();

            if cleaned.is_empty() {
                return None;
            }

            let fixed_date = fix_timezone_abbreviation(&cleaned).ok()?;
            parse_email_date(&fixed_date).ok()
        })
}

fn fix_timezone_abbreviation(date_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Common timezone abbreviations and their offsets
    let tz_map = [
        ("BST", "+0100"),     // British Summer Time
        ("MET", "+0100"),     // Middle European Time
        ("CET", "+0100"),     // Central European Time
        ("WEST", "+0100"),    // Western European Summer Time
        ("EET DST", "+0300"), // Eastern European Summer Time
        ("EEST", "+0300"),    // Eastern European Summer Time
        ("EET", "+0200"),     // Eastern European Time
        ("GMT", "+0000"),     // Greenwich Mean Time
        ("UTC", "+0000"),     // Coordinated Universal Time
        ("PDT", "-0700"),     // Pacific Daylight Time
        ("PST", "-0800"),     // Pacific Standard Time
        ("EDT", "-0400"),     // Eastern Daylight Time
        ("EST", "-0500"),     // Eastern Standard Time
        ("CDT", "-0500"),     // Central Daylight Time
        ("CST", "-0600"),     // Central Standard Time
        ("MDT", "-0600"),     // Mountain Daylight Time
        ("MST", "-0700"),     // Mountain Standard Time
        ("AKDT", "-0800"),    // Alaska Daylight Time
        ("AKST", "-0900"),    // Alaska Standard Time
        ("HST", "-1000"),     // Hawaii Standard Time
        ("HAST", "-1000"),    // Hawaii-Aleutian Standard Time
        ("HADT", "-0900"),    // Hawaii-Aleutian Daylight Time
        ("CHST", "+1000"),    // Chamorro Standard Time
        ("SST", "-1100"),     // Samoa Standard Time
        ("NST", "-0330"),     // Newfoundland Standard Time
        ("NDT", "-0230"),     // Newfoundland Daylight Time
    ];

    // Handle GMT followed by offset (e.g. GMT+2 or GMT+0200) case-insensitively
    let gmt_re = Regex::new(r"(?i)GMT([+-]?)(\d{1,4})$")?;
    if let Some(caps) = gmt_re.captures(date_str) {
        let sign = caps.get(1).map(|m| m.as_str()).unwrap_or("+");
        let digits = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let offset = if digits.len() <= 2 {
            // Handle hour-only format (e.g. GMT+2)
            let hour: i32 = digits.parse().unwrap_or(0);
            format!("{}{:02}00", sign, hour)
        } else {
            // Already in 4-digit format (e.g. GMT+1200)
            format!("{}{}", sign, digits)
        };
        return Ok(format!(
            "{}{}",
            &date_str[..date_str.len() - caps[0].len()],
            offset
        ));
    }

    // Find and replace timezone abbreviations case-insensitively
    for (abbr, offset) in tz_map {
        let lower_abbr = abbr.to_lowercase();
        if date_str.len() >= abbr.len() {
            let date_str_end =
                &date_str[date_str.len().saturating_sub(abbr.len())..].to_lowercase();
            if date_str_end == &lower_abbr {
                return Ok(date_str[..date_str.len() - abbr.len()].to_string() + offset);
            }
        }
    }

    // Normalize spaces and trim
    let space_re = Regex::new(r"\s{2,}")?;
    let result = space_re.replace_all(date_str, " ").to_string();
    Ok(result.trim().to_string())
}

pub async fn check_for_new_emails(
    pool: &deadpool_postgres::Pool,
    maildir_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let maildir: &Path = Path::new(maildir_path);

    // Collect all file paths first
    let file_paths: Vec<(PathBuf, String)> = WalkDir::new(maildir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let full_path = entry.path().to_owned();
            entry
                .path()
                .strip_prefix(maildir)
                .ok()
                .and_then(|p| p.to_str())
                .map(|rel_path| (full_path, rel_path.to_string()))
        })
        .collect();

    if file_paths.is_empty() {
        return Ok(());
    }

    let mut existing_paths = HashSet::new();

    // Process in batches
    for chunk in file_paths.chunks(BATCH_SIZE) {
        let relative_paths: Vec<&str> = chunk.iter().map(|(_, rel)| rel.as_str()).collect();

        let query = "SELECT file_path FROM messages WHERE file_path = ANY($1::text[])";

        // Get existing paths for this batch
        let batch_existing: Vec<String> = client
            .query(query, &[&relative_paths])
            .await?
            .iter()
            .map(|row| row.get::<_, String>("file_path"))
            .collect();

        existing_paths.extend(batch_existing);
    }

    // Process only new files
    for (full_path, relative_path) in file_paths {
        if !existing_paths.contains(&relative_path) {
            if let Err(e) = process_email(&client, &full_path, maildir_path).await {
                error!("Error processing new email {}: {}", full_path.display(), e);
            }
        }
    }

    Ok(())
}

pub async fn vote_spam(
    pool: &Pool,
    message_id: i32,
    user_id: i32,
) -> Result<(i64, bool), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Check if the user has already voted for this message
    let existing_vote = transaction
        .query_opt(
            "SELECT 1 FROM message_spam_votes WHERE message_id = $1 AND user_id = $2",
            &[&message_id, &user_id],
        )
        .await?;

    let user_voted_after_operation: bool;

    if existing_vote.is_some() {
        // User has voted, so unvote (delete the record)
        transaction
            .execute(
                "DELETE FROM message_spam_votes WHERE message_id = $1 AND user_id = $2",
                &[&message_id, &user_id],
            )
            .await?;
        user_voted_after_operation = false;
    } else {
        // User has not voted, so vote (insert the record)
        transaction
            .execute(
                "INSERT INTO message_spam_votes (message_id, user_id) VALUES ($1, $2)",
                &[&message_id, &user_id],
            )
            .await?;
        user_voted_after_operation = true;
    }

    // Get the new total spam vote count for the message.
    let spam_vote_count_row = transaction
        .query_one(
            "SELECT COUNT(*) FROM message_spam_votes WHERE message_id = $1",
            &[&message_id],
        )
        .await?;
    transaction.commit().await?;
    Ok((spam_vote_count_row.get(0), user_voted_after_operation))
}
