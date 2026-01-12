use crate::utils::remove_html_tags;
use deadpool_postgres::Pool;
use log::{info, warn};
use std::time::SystemTime;

use super::dto::{MuplisEntry, MuplisSearchQuery, MuplisSearchResponse};

pub fn sanitize_html(html: &str) -> String {
    remove_html_tags(html)
}

pub async fn search_muplis(
    pool: &Pool,
    query: MuplisSearchQuery,
) -> Result<MuplisSearchResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;
    let search_query = format!("%{}%", query.query);
    let full_word_regex = format!(r"\y{}\y([^']|$)", query.query);

    // Determine sort ordering based on both search query and sort parameter
    let sort_column = match query.sort.as_deref() {
        Some("lojban") => "lojban",
        Some("english") => "english",
        // Add other valid sortable columns here
        _ => "rank",
    };

    let sort_order = if query.query.is_empty() {
        // Use RANDOM() for empty search query
        "RANDOM()".to_string()
    } else {
        // Use specified sort order for non-empty search
        let direction = match query.sort.as_deref() {
            Some(s) if s.eq_ignore_ascii_case("asc") => "ASC",
            Some(s) if s.eq_ignore_ascii_case("desc") => "DESC",
            _ => "ASC",
        };
        format!("{} {}", sort_column, direction)
    };

    let query_string = format!(
        "SELECT id, lojban, english,
         (CASE 
            WHEN lojban ~ $1 OR english ~ $1 THEN 2
            WHEN lojban ILIKE $2 OR english ILIKE $2 THEN 1
            ELSE 0
         END) as rank
         FROM muplis
         WHERE lojban ILIKE $2 OR english ILIKE $2
         ORDER BY {} LIMIT $3 OFFSET $4",
        sort_order
    );

    let entries = transaction
        .query(
            &query_string,
            &[&full_word_regex, &search_query, &per_page, &offset],
        )
        .await?
        .into_iter()
        .map(MuplisEntry::from)
        .collect::<Vec<_>>();

    let total: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM muplis WHERE lojban ILIKE $1 OR english ILIKE $1",
            &[&search_query],
        )
        .await?
        .get(0);

    transaction.commit().await?;

    Ok(MuplisSearchResponse {
        entries,
        total,
        page,
        per_page,
    })
}

pub async fn update_data(pool: &Pool, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let response = reqwest::get(url).await?;
    let content = response.text().await?;

    let transaction = client.transaction().await?;

    // Flush the muplis table before updating
    transaction.execute("DELETE FROM muplis", &[]).await?;

    for (line_num, line) in content.lines().enumerate() {
        let parts: Vec<&str> = line.split('\t').collect();
        if let (Some(lojban), Some(english)) = (parts.get(0), parts.get(1)) {
            let sanitized_lojban = sanitize_html(lojban);
            let sanitized_english = sanitize_html(english);
            transaction
                .execute(
                    "INSERT INTO muplis (lojban, english) VALUES ($1, $2)",
                    &[&sanitized_lojban, &sanitized_english],
                )
                .await?;
        } else {
            warn!(
                "Skipping invalid line {} in muplis data: {}",
                line_num + 1,
                line
            );
        }
    }

    transaction.commit().await?;
    Ok(())
}

pub async fn update_if_needed(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs() as i64;

    let row = client
        .query_opt(
            "SELECT last_update FROM muplis_update ORDER BY id DESC LIMIT 1",
            &[],
        )
        .await?;

    let last_update: i64 = row.and_then(|row| row.get(0)).unwrap_or_default();

    if now - last_update > 3 * 24 * 60 * 60 {
        let muplis_url = "https://github.com/La-Lojban/sutysisku-lojban-corpus-downloader/raw/refs/heads/gh-pages/data/dumps/muplis-jb2en.tsv";
        update_data(pool, muplis_url).await?;

        client
            .execute(
                "INSERT INTO muplis_update (last_update) VALUES ($1)",
                &[&now],
            )
            .await?;
        info!("Muplis data updated successfully");
    }

    Ok(())
}
