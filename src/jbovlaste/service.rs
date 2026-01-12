use crate::utils::remove_html_tags;
use camxes_rs::peg::grammar::Peg;
use chrono::TimeZone;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;

use deadpool_postgres::{Pool, Transaction};
use log::debug;

use super::broadcast::Broadcaster;
use super::dto::ClientIdGroup;
use super::{
    AddDefinitionRequest, BulkImportParams, DefinitionListResponse, DefinitionResponse,
    GetImageDefinitionQuery, ImageData, KeywordMapping, ListDefinitionsQuery,
    NonLojbanDefinitionsQuery, RecentChange, RecentChangesResponse, SearchDefinitionsParams,
    UpdateDefinitionRequest, ValsiDetail, ValsiType,
};
use crate::jbovlaste::models::DefinitionDetail;
use vlazba::jvokaha::jvokaha;

use crate::auth::Claims;
use crate::language::{analyze_word, validate_mathjax, MathJaxValidationOptions};
use crate::middleware::cache::RedisCache;
use crate::subscriptions::models::SubscriptionTrigger;
use crate::versions::service::{get_diff, get_version_with_transaction};
use crate::versions::{Change, ChangeType, VersionContent, VersionDiff};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use chrono::{DateTime, Duration, Utc};

pub fn sanitize_html(html: &str) -> String {
    remove_html_tags(html)
}

pub async fn semantic_search(
    pool: &Pool,
    params: SearchDefinitionsParams,
    query_embedding: Vec<f32>,
) -> Result<DefinitionResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let offset = (params.page - 1) * params.per_page;

    // Convert Option<Vec<i32>> to Option<&[i32]> for Postgres
    let languages_slice: Option<&[i32]> = match params.languages.as_deref() {
        Some(&[1]) => Some(&[]),
        other => other,
    };

    // Convert Vec<f32> to pgvector::Vector
    let vector = pgvector::Vector::from(query_embedding);
    // Start with parameters needed for both queries (main and count)
    let mut query_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        vec![&vector, &languages_slice];

    // Build dynamic conditions and add parameters
    let mut conditions = vec![];

    // Add selmaho condition if present
    if let Some(selmaho) = &params.selmaho {
        conditions.push(format!("AND d.selmaho = ${}", query_params.len() + 1));
        query_params.push(selmaho);
    }

    // Add username condition if present
    if let Some(username) = &params.username {
        conditions.push(format!("AND u.username = ${}", query_params.len() + 1));
        query_params.push(username);
    }

    let word_type_value;
    if let Some(word_type) = params.word_type {
        word_type_value = word_type;
        conditions.push(format!("AND v.typeid = ${}", query_params.len() + 1));
        query_params.push(&word_type_value);
    }

    // Add source_langid condition if present, otherwise default to 1 (Lojban)
    let source_langid_value = params.source_langid.unwrap_or(1);
    conditions.push(format!("AND v.source_langid = ${}", query_params.len() + 1));
    query_params.push(&source_langid_value);

    let additional_conditions = conditions.join(" ");

    // --- Execute Count Query ---
    // Optimized to use JOINs instead of subqueries
    let count_query = format!(
        r#"
        WITH vote_scores AS (
            SELECT definitionid, COALESCE(SUM(value), 0) as score
            FROM definitionvotes
            GROUP BY definitionid
        ),
        vector_search AS (
            SELECT 
                d.definitionid,
                d.embedding <=> $1::vector as similarity,
                COALESCE(dv.score, 0) as score
            FROM definitions d
            JOIN valsi v ON d.valsiid = v.valsiid
            JOIN valsitypes vt ON v.typeid = vt.typeid
            JOIN users u ON d.userid = u.userid
            JOIN languages l ON d.langid = l.langid
            LEFT JOIN vote_scores dv ON dv.definitionid = d.definitionid
            WHERE d.langid != 1 
              AND (d.langid = ANY($2) OR $2 IS NULL) 
              AND d.definition != ''
              AND d.embedding IS NOT NULL
            {additional_conditions}
            ORDER BY d.embedding <=> $1::vector
            LIMIT 1000
        )
        SELECT COUNT(*)
        FROM vector_search
        WHERE score > 0 AND similarity < 0.4"#
    );

    // Execute count query with all necessary parameters accumulated so far
    let total: i64 = transaction
        .query_one(&count_query, &query_params)
        .await?
        .get(0);

    // --- Prepare and Execute Main Query ---
    // Add limit and offset parameters ONLY for the main query
    let limit_param_index = query_params.len() + 1;
    query_params.push(&params.per_page);
    let offset_param_index = query_params.len() + 1;
    query_params.push(&offset);

    let query_string = format!(
        r#"
        WITH vote_scores AS (
            SELECT definitionid, COALESCE(SUM(value), 0) as score
            FROM definitionvotes
            GROUP BY definitionid
        ),
        definition_images_flag AS (
            SELECT DISTINCT definition_id
            FROM definition_images
        ),
        vector_search AS (
            SELECT 
                d.definitionid, d.valsiid, d.langid, d.definition, d.notes, d.etymology, d.created_at,
                d.selmaho, d.jargon, d.definitionnum, d.time, d.owner_only,
                v.word as valsiword,
                u.username,
                l.realname as langrealname,
                vt.descriptor as type_name,
                COALESCE(dv.score, 0) as score,
                COALESCE(cc.comment_count, 0) as comment_count,
                (di.definition_id IS NOT NULL) as has_image,
                d.embedding <=> $1::vector as similarity
            FROM definitions d
            JOIN valsi v ON d.valsiid = v.valsiid
            JOIN valsitypes vt ON v.typeid = vt.typeid
            JOIN users u ON d.userid = u.userid
            JOIN languages l ON d.langid = l.langid
            LEFT JOIN vote_scores dv ON dv.definitionid = d.definitionid
            LEFT JOIN LATERAL (
                SELECT COUNT(c.commentid) as comment_count
                FROM threads t
                LEFT JOIN comments c ON c.threadid = t.threadid
                WHERE (t.valsiid = v.valsiid OR t.definitionid = d.definitionid)
            ) cc ON true
            LEFT JOIN definition_images_flag di ON di.definition_id = d.definitionid
            WHERE d.langid != 1 
              AND (d.langid = ANY($2) OR $2 IS NULL) 
              AND d.definition != ''
              AND d.embedding IS NOT NULL
            {additional_conditions}
            ORDER BY d.embedding <=> $1::vector
            LIMIT 1000
        ),
        ranked_results AS (
            SELECT DISTINCT ON (definitionid) *
            FROM vector_search
            WHERE score > 0 AND similarity < 0.4
        )
        SELECT r.*
        FROM ranked_results r
        ORDER BY r.similarity ASC
        LIMIT ${limit_param_index} OFFSET ${offset_param_index}"#
    );

    let mut definitions: Vec<DefinitionDetail> = Vec::new();
    // Execute main query with the final parameter list (including limit/offset)
    let rows = transaction.query(&query_string, &query_params).await?;

    println!("{}", rows.len());
    // Process each definition
    for row in rows {
        let def_id: i32 = row.get("definitionid");

        let word: String = row.get("valsiword");
        definitions.push(DefinitionDetail {
            embedding: None,
            similarity: row.get::<_, Option<f64>>("similarity"),
            definitionid: def_id,
            valsiword: word.clone(),
            valsiid: row.get("valsiid"),
            langid: row.get("langid"),
            definition: row.get("definition"),
            notes: row.get("notes"),
            etymology: row.get("etymology"),
            selmaho: row.get("selmaho"),
            jargon: row.get("jargon"),
            definitionnum: row.get("definitionnum"),
            langrealname: row.get("langrealname"),
            username: row.get("username"),
            time: row.get("time"),
            type_name: row.get("type_name"),
            score: row.get("score"),
            comment_count: if params.include_comments {
                Some(row.get("comment_count"))
            } else {
                None
            },
            gloss_keywords: None,
            place_keywords: None,
            user_vote: None,
            owner_only: row.get("owner_only"),
            can_edit: false,
            created_at: row.get("created_at"),
            has_image: row.get("has_image"),
            sound_url: None,
            metadata: None,
        });
    }

    // Decomposition is fetched after the main query results
    let decomposition = get_source_words(&params.search_term, &transaction).await?;

    transaction.commit().await?;

    Ok(DefinitionResponse {
        definitions,
        decomposition,
        total,
    })
}

// Helper function to fetch keywords (extracted and adapted from search_definitions)
async fn fetch_keywords(
    transaction: &Transaction<'_>,
    def_ids: &[i32],
) -> Result<
    (
        HashMap<i32, Vec<KeywordMapping>>, // Gloss keywords
        HashMap<i32, Vec<KeywordMapping>>, // Place keywords
    ),
    Box<dyn std::error::Error>,
> {
    let mut gloss_map: HashMap<i32, Vec<KeywordMapping>> = HashMap::new();
    let mut place_map: HashMap<i32, Vec<KeywordMapping>> = HashMap::new();

    if def_ids.is_empty() {
        return Ok((gloss_map, place_map));
    }

    // Fetch gloss keywords
    let gloss_rows = transaction
        .query(
            "SELECT k.definitionid, n.word, n.meaning
             FROM keywordmapping k
             JOIN natlangwords n ON k.natlangwordid = n.wordid
             WHERE k.definitionid = ANY($1) AND k.place = 0",
            &[&def_ids],
        )
        .await?;

    for row in gloss_rows {
        let def_id: i32 = row.get("definitionid");
        let mapping = KeywordMapping {
            word: row.get("word"),
            meaning: row.get("meaning"),
        };
        gloss_map.entry(def_id).or_default().push(mapping);
    }

    // Fetch place keywords
    let place_rows = transaction
        .query(
            "SELECT k.definitionid, n.word, n.meaning
             FROM keywordmapping k
             JOIN natlangwords n ON k.natlangwordid = n.wordid
             WHERE k.definitionid = ANY($1) AND k.place > 0
             ORDER BY k.place",
            &[&def_ids],
        )
        .await?;

    for row in place_rows {
        let def_id: i32 = row.get("definitionid");
        let mapping = KeywordMapping {
            word: row.get("word"),
            meaning: row.get("meaning"),
        };
        place_map.entry(def_id).or_default().push(mapping);
    }

    Ok((gloss_map, place_map))
}

pub async fn search_definitions(
    pool: &Pool,
    params: SearchDefinitionsParams,
    redis_cache: &RedisCache,
) -> Result<DefinitionResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let offset = (params.page - 1) * params.per_page;
    let like_pattern = format!("%{}%", params.search_term);
    let word_boundary_pattern = format!(r"\y{}\y", params.search_term);

    // Convert Option<Vec<i32>> to Option<&[i32]> for Postgres
    let languages_slice: Option<&[i32]> = params.languages.as_deref();

    // Add table alias to sort_by
    let sort_column = match params.sort_by.as_str() {
        "word" => "r.valsiword",
        "type" => "r.type_name",
        "date" => "r.time",
        "score" => "r.score",
        _ => "r.valsiword",
    };
    let sort_order = match params.sort_order.as_str() {
        s if s.eq_ignore_ascii_case("desc") => "DESC",
        _ => "ASC",
    };

    let mut query_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        &params.search_term,
        &like_pattern,
        &word_boundary_pattern,
        &languages_slice,
        &params.per_page,
        &offset,
    ];

    // Build dynamic conditions
    let mut conditions = vec![];

    // Add selmaho condition if present
    if let Some(selmaho) = &params.selmaho {
        conditions.push(format!("AND d.selmaho = ${}", query_params.len() + 1));
        query_params.push(selmaho);
    }

    // Add username condition if present
    if let Some(username) = &params.username {
        conditions.push(format!("AND u.username = ${}", query_params.len() + 1));
        query_params.push(username);
    }

    let word_type_value;
    if let Some(word_type) = params.word_type {
        word_type_value = word_type;
        conditions.push(format!("AND v.typeid = ${}", query_params.len() + 1));
        query_params.push(&word_type_value);
    }

    // Add source_langid condition if present, otherwise default to 1 (Lojban)
    let source_langid_value = params.source_langid.unwrap_or(1);
    conditions.push(format!("AND v.source_langid = ${}", query_params.len() + 1));
    query_params.push(&source_langid_value);

    let additional_conditions = conditions.join(" ");

    // Build query with language filter and selmaho
    // Optimized to use JOINs instead of subqueries for better performance
    let query_string = if params.include_comments {
        format!(
            r#"
        WITH vote_scores AS (
            SELECT definitionid, COALESCE(SUM(value), 0) as score
            FROM definitionvotes
            GROUP BY definitionid
        ),
        definition_images_flag AS (
            SELECT DISTINCT definition_id
            FROM definition_images
        ),
        base_data AS (
            SELECT 
                d.definitionid, d.valsiid, d.langid, d.definition, d.notes, d.etymology, d.created_at,
                d.selmaho, d.jargon, d.definitionnum, d.time, d.owner_only,
                v.word as valsiword,
                u.username,
                l.realname as langrealname,
                vt.descriptor as type_name,
                COALESCE(dv.score, 0) as score,
                COALESCE(cc.comment_count, 0) as comment_count,
                (di.definition_id IS NOT NULL) as has_image,
                CASE
                    WHEN v.word = $1 THEN 12
                    WHEN v.word ILIKE $1 THEN 11
                    WHEN v.word ~* $3 THEN 10
                    WHEN v.rafsi IS NOT NULL AND $1 = ANY(string_to_array(v.rafsi, ' ')) THEN 9
                    WHEN v.word ILIKE $2 THEN 8
                    WHEN d.definition ~ $3 THEN 6
                    WHEN d.notes ~ $3 THEN 4
                    WHEN d.selmaho ~ $3 THEN 3
                    WHEN d.definition ILIKE $2 OR
                         d.notes ILIKE $2 OR
                         d.selmaho ILIKE $2 THEN 1
                    ELSE 0
                END as rank
            FROM definitions d
            JOIN valsi v ON d.valsiid = v.valsiid
            JOIN valsitypes vt ON v.typeid = vt.typeid
            JOIN users u ON d.userid = u.userid
            JOIN languages l ON d.langid = l.langid
            LEFT JOIN vote_scores dv ON dv.definitionid = d.definitionid
            LEFT JOIN LATERAL (
                SELECT COUNT(c.commentid) as comment_count
                FROM threads t
                LEFT JOIN comments c ON c.threadid = t.threadid
                WHERE (t.valsiid = v.valsiid OR t.definitionid = d.definitionid)
            ) cc ON true
            LEFT JOIN definition_images_flag di ON di.definition_id = d.definitionid
            WHERE ($1 = '' OR
                  v.word ILIKE $2 OR
                  v.source_langid = 1 AND
                  d.definition ILIKE $2 OR
                  d.notes ILIKE $2 OR
                  d.selmaho ILIKE $2 OR
                  $1 = ANY(string_to_array(v.rafsi, ' '))
                  )
                  AND (d.langid = ANY($4) OR $4 IS NULL)
                  {additional_conditions}
        ),
        ranked_results AS (
            SELECT DISTINCT ON (definitionid) *
            FROM base_data
            WHERE rank > 0
        )
        SELECT r.*
        FROM ranked_results r
        ORDER BY r.rank DESC, r.score DESC, {} {}
        LIMIT $5 OFFSET $6"#,
            sort_column, sort_order
        )
    } else {
        format!(
            r#"
        WITH vote_scores AS (
            SELECT definitionid, COALESCE(SUM(value), 0) as score
            FROM definitionvotes
            GROUP BY definitionid
        ),
        definition_images_flag AS (
            SELECT DISTINCT definition_id
            FROM definition_images
        ),
        base_data AS (
            SELECT 
                d.definitionid, d.valsiid, d.langid, d.definition, d.notes, d.etymology, d.created_at,
                d.selmaho, d.jargon, d.definitionnum, d.time, d.owner_only,
                v.word as valsiword,
                u.username,
                l.realname as langrealname,
                vt.descriptor as type_name,
                COALESCE(dv.score, 0) as score,
                (di.definition_id IS NOT NULL) as has_image,
                CASE
                    WHEN v.word = $1 THEN 12
                    WHEN v.word ILIKE $1 THEN 11
                    WHEN v.word ~* $3 THEN 10
                    WHEN v.rafsi IS NOT NULL AND $1 = ANY(string_to_array(v.rafsi, ' ')) THEN 9
                    WHEN v.word ILIKE $2 THEN 8
                    WHEN d.definition ~ $3 THEN 6
                    WHEN d.notes ~ $3 THEN 4
                    WHEN d.selmaho ~ $3 THEN 3
                    WHEN d.definition ILIKE $2 OR
                         d.notes ILIKE $2 OR
                         d.selmaho ILIKE $2 THEN 1
                    ELSE 0
                END as rank
            FROM definitions d
            JOIN valsi v ON d.valsiid = v.valsiid
            JOIN valsitypes vt ON v.typeid = vt.typeid
            JOIN users u ON d.userid = u.userid
            JOIN languages l ON d.langid = l.langid
            LEFT JOIN vote_scores dv ON dv.definitionid = d.definitionid
            LEFT JOIN definition_images_flag di ON di.definition_id = d.definitionid
            WHERE ($1 = '' OR
                  v.word ILIKE $2 OR
                  v.source_langid = 1 AND
                  d.definition ILIKE $2 OR
                  d.notes ILIKE $2 OR
                  d.selmaho ILIKE $2 OR
                  $1 = ANY(string_to_array(v.rafsi, ' '))
                  )
                  AND (d.langid = ANY($4) OR $4 IS NULL)
                  {additional_conditions}
        ),
        ranked_results AS (
            SELECT DISTINCT ON (definitionid) *
            FROM base_data
            WHERE rank > 0
        )
        SELECT r.*
        FROM ranked_results r
        ORDER BY r.rank DESC, r.score DESC, {} {}
        LIMIT $5 OFFSET $6"#,
            sort_column, sort_order
        )
    };

    let mut definitions: Vec<DefinitionDetail> = Vec::new();
    let rows = transaction.query(&query_string, &query_params).await?;

    // Get all definition IDs for keyword fetching
    let def_ids: Vec<i32> = rows.iter().map(|row| row.get("definitionid")).collect();

    // Use the new helper function
    let (gloss_keywords_map, place_keywords_map) = fetch_keywords(&transaction, &def_ids).await?;

    let words: Vec<String> = rows
        .iter()
        .map(|row| row.get::<_, String>("valsiword"))
        .collect();

    let sound_urls = check_sound_urls(&words, redis_cache).await;

    // Process each definition
    for row in rows {
        let def_id: i32 = row.get("definitionid");

        let word: String = row.get("valsiword");
        definitions.push(DefinitionDetail {
            similarity: None,
            definitionid: def_id,
            valsiword: word.clone(),
            valsiid: row.get("valsiid"),
            langid: row.get("langid"),
            definition: row.get("definition"),
            notes: row.get("notes"),
            etymology: row.get("etymology"),
            selmaho: row.get("selmaho"),
            jargon: row.get("jargon"),
            definitionnum: row.get("definitionnum"),
            langrealname: row.get("langrealname"),
            username: row.get("username"),
            time: row.get("time"),
            type_name: row.get("type_name"),
            score: row.get("score"),
            comment_count: if params.include_comments {
                Some(row.get("comment_count"))
            } else {
                None
            },
            gloss_keywords: gloss_keywords_map.get(&def_id).cloned(), // Use map
            place_keywords: place_keywords_map.get(&def_id).cloned(), // Use map
            user_vote: None, // Placeholder - needs user_id context
            owner_only: row.get("owner_only"),
            can_edit: false, // Placeholder - needs user_id context
            created_at: row.get("created_at"),
            has_image: row.get("has_image"),
            sound_url: sound_urls.get(&word).cloned().flatten(), // Fixed type mismatch
            embedding: None,
            metadata: None,
        });
    }

    // Base conditions now include source_langid check ($5)
    let base_conditions = r#"($1 = '' OR
                  v.word ILIKE $2 OR
                  d.definition ILIKE $2 OR
                  d.notes ILIKE $2 OR
                  d.selmaho ILIKE $2 OR
                  $1 = ANY(string_to_array(v.rafsi, ' ')))
                  AND (d.langid = ANY($4) OR $4 IS NULL)
                  AND v.source_langid = $5"#; // Added source_langid condition

    // Build dynamic conditions with correct parameter numbering
    let mut conditions = vec![];
    let mut current_param_num = 6; // Start from 6 since we now use 1-5 in base

    if params.selmaho.is_some() {
        conditions.push(format!("AND d.selmaho = ${}", current_param_num));
        current_param_num += 1;
    }

    // Add username condition if present
    if params.username.is_some() {
        conditions.push(format!("AND u.username = ${}", current_param_num));
        current_param_num += 1;
    }

    // word_type condition needs to increment param_num too
    if params.word_type.is_some() {
        conditions.push(format!("AND v.typeid = ${}", current_param_num));
    }

    let additional_conditions = conditions.join(" ");

    let count_query = format!(
        r#"
    WITH ranked_results AS (
        SELECT d.definitionid,
            CASE
                WHEN v.word = $1 THEN 12
                WHEN v.word ILIKE $1 THEN 11
                WHEN v.word ~* $3 THEN 10
                WHEN v.rafsi IS NOT NULL AND $1 = ANY(string_to_array(v.rafsi, ' ')) THEN 9
                WHEN v.word ILIKE $2 THEN 8
                WHEN d.definition ~ $3 THEN 6
                WHEN d.notes ~ $3 THEN 4
                WHEN d.selmaho ~ $3 THEN 3
                WHEN d.definition ILIKE $2 OR
                     d.notes ILIKE $2 OR
                     d.selmaho ILIKE $2 THEN 1
                ELSE 0
            END as rank
        FROM definitions d
        JOIN valsi v ON d.valsiid = v.valsiid
        JOIN valsitypes vt ON v.typeid = vt.typeid
        JOIN users u ON d.userid = u.userid
         JOIN languages l ON d.langid = l.langid
         WHERE {base_conditions} {additional_conditions}
    )
    SELECT COUNT(DISTINCT definitionid)
    FROM ranked_results
    WHERE rank > 0"#
    );

    // Create params for count query, adding source_langid after base params
    let mut count_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        &params.search_term,    // $1
        &like_pattern,          // $2
        &word_boundary_pattern, // $3
        &languages_slice,       // $4
        &source_langid_value,   // $5 (Now part of base_conditions)
    ];

    // Add conditional parameters in the correct order, matching additional_conditions logic
    if let Some(selmaho) = &params.selmaho {
        count_params.push(selmaho); // $6 if present
    }
    if let Some(username) = &params.username {
        count_params.push(username); // $7 if selmaho present, else $6
    }
    let word_type_value; // Temporary storage needed outside the if block
    if let Some(word_type) = params.word_type {
        word_type_value = word_type;
        count_params.push(&word_type_value); // $8/$7/$6 depending on others
    }
    // source_langid_value is already added above

    let total: i64 = transaction
        .query_one(&count_query, &count_params) // Use the constructed count_params
        .await?
        .get(0);

    let decomposition = get_source_words(&params.search_term, &transaction).await?;

    transaction.commit().await?;

    Ok(DefinitionResponse {
        definitions,
        decomposition,
        total,
    })
}

async fn get_source_words(
    word: &str,
    transaction: &tokio_postgres::Transaction<'_>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let parts = jvokaha(word).unwrap_or_else(|_| {
        debug!("Failed to decompose word '{}'", word);
        Vec::new()
    });
    let rafsi_parts: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();

    // First try to find exact word matches
    let exact_word_rows = transaction
        .query(
            "SELECT word FROM valsi WHERE word = ANY($1::text[])",
            &[&rafsi_parts],
        )
        .await?;

    let mut exact_word_map: HashMap<String, String> = HashMap::new();
    for row in exact_word_rows {
        let word: String = row.get("word");
        exact_word_map.insert(word.clone(), word);
    }

    // Then try to find rafsi matches
    let rows = transaction.query(
        "SELECT word, rafsi FROM valsi WHERE rafsi LIKE ANY(ARRAY(SELECT '%' || rafsi || '%' FROM unnest($1::text[])))",
        &[&rafsi_parts]
    ).await?;

    debug!("{:#?}", rafsi_parts);
    let mut rafsi_map: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let word: String = row.get("word");
        if let Some(db_rafsi) = row.get::<_, Option<String>>("rafsi") {
            let db_parts: Vec<String> =
                db_rafsi.split_whitespace().map(|s| s.to_string()).collect();
            for rafsi in db_parts {
                rafsi_map.entry(rafsi).or_default().push(word.clone());
            }
        }
    }

    let source_words: Vec<String> = parts
        .iter()
        .filter_map(|rafsi| {
            // First try to find rafsi match
            rafsi_map
                .get(rafsi)
                .and_then(|words| words.first().cloned())
                // If no rafsi match found, try exact word match
                .or_else(|| exact_word_map.get(rafsi).cloned())
        })
        .collect();
    debug!("{:#?}", source_words);

    Ok(source_words)
}

pub async fn check_sound_urls(
    words: &[String],
    redis_cache: &RedisCache,
) -> HashMap<String, Option<String>> {
    let unique_words: Vec<String> = words
        .iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .cloned()
        .collect();

    let client = reqwest::Client::new();
    let base_url = "https://raw.githubusercontent.com/La-Lojban/sutysisku-lojban-corpus-downloader/gh-pages/data/sance";
    let semaphore = Arc::new(tokio::sync::Semaphore::new(10));
    let mut results = HashMap::new();

    let futures: Vec<_> = unique_words
        .iter()
        .map(|word| {
            let url = format!("{}/{}.ogg", base_url, word);
            let client = client.clone();
            let semaphore = semaphore.clone();
            let word = word.clone();
            let cache_key = format!("sound_url:{}", word);

            async move {
                let exists = (redis_cache
                    .get_or_set(
                        &cache_key,
                        || async {
                            let _permit = semaphore.acquire().await.map_err(|e| {
                                format!("Semaphore acquire failed for sound URL check: {}", e)
                            })?;
                            let resp = client.head(&url).send().await;
                            Ok(resp.map(|r| r.status().is_success()).unwrap_or(false))
                        },
                        Some(std::time::Duration::from_secs(100 * 24 * 60 * 60)),
                    )
                    .await)
                    .unwrap_or(false);

                (
                    word.clone(),
                    if exists {
                        Some(format!("{}/{}.ogg", base_url, word))
                    } else {
                        None
                    },
                )
            }
        })
        .collect();

    for result in futures::future::join_all(futures).await {
        results.insert(result.0, result.1);
    }

    results
}

pub async fn get_entry_details(
    pool: &Pool,
    id_or_word: &str,
) -> Result<ValsiDetail, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let result = transaction
        .query_opt(
            "SELECT v.valsiid, v.word, vt.descriptor as type_name, v.rafsi, v.source_langid,
             (SELECT COUNT(c.commentid)
              FROM threads t
              LEFT JOIN comments c ON t.threadid = c.threadid
              WHERE t.valsiid = v.valsiid AND t.definitionid = 0) as comment_count
             FROM valsi v
             JOIN valsitypes vt ON v.typeid = vt.typeid
             WHERE CASE
                WHEN $1 ~ '^\\d+$' THEN v.valsiid = $1::int AND v.source_langid = 1
                ELSE v.word = $2 AND v.source_langid = 1
            END",
            &[&id_or_word, &id_or_word],
        )
        .await?;

    match result {
        Some(row) => {
            let mut detail = ValsiDetail {
                source_langid: row.get("source_langid"),
                valsiid: row.get("valsiid"),
                word: row.get("word"),
                type_name: row.get("type_name"),
                rafsi: row.get("rafsi"),
                comment_count: row.get("comment_count"),
                decomposition: None,
            };

            if detail.type_name.to_lowercase() == "lujvo" {
                match get_source_words(&detail.word, &transaction).await {
                    Ok(words) => detail.decomposition = Some(words),
                    Err(e) => log::error!("Failed to decompose lujvo {}: {}", detail.word, e),
                }
            }

            transaction.commit().await?;
            Ok(detail)
        }
        None => {
            transaction.commit().await?;
            Err("Valsi not found".into())
        }
    }
}

pub async fn add_definition(
    pool: &Pool,
    claims: &Claims,
    parsers: Arc<HashMap<i32, Peg>>,
    request: &AddDefinitionRequest,
    redis_cache: &RedisCache,
    send_notifications: bool,
) -> Result<(String, i32), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let definition = add_definition_in_transaction(
        &transaction,
        claims,
        parsers,
        request,
        redis_cache,
        send_notifications,
    )
    .await;

    transaction.commit().await?;

    definition
}

async fn add_definition_in_transaction(
    transaction: &Transaction<'_>,
    claims: &Claims,
    parsers: Arc<HashMap<i32, Peg>>,
    request: &AddDefinitionRequest,
    redis_cache: &RedisCache,
    send_notifications: bool,
) -> Result<(String, i32), Box<dyn std::error::Error>> {
    let sanitized_definition = sanitize_html(&request.definition);
    let sanitized_notes = request.notes.as_ref().map(|n| sanitize_html(n));
    let sanitized_etymology = request.etymology.as_ref().map(|e| sanitize_html(e));
    let sanitized_selmaho = request.selmaho.as_ref().map(|s| sanitize_html(s));
    let sanitized_jargon = request.jargon.as_ref().map(|j| sanitize_html(j));

    let combined_text = format!(
        "{} {} {}",
        sanitized_definition,
        sanitized_notes.as_deref().unwrap_or(""),
        sanitized_etymology.as_deref().unwrap_or("")
    );

    let options = MathJaxValidationOptions { use_tectonic: true };

    // Use provided source_langid or default to 1 (Lojban)
    let source_langid = request.source_langid.unwrap_or(1);

    let (word, word_type) = match source_langid {
        1 | 58 => {
            // Lojban or Loglan
            validate_mathjax(&combined_text, options)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            let res = analyze_word(&parsers, &request.word, source_langid, transaction).await?;
            (res.text, res.word_type)
        }
        _ => (sanitize_html(&request.word), "phrase".to_string()),
    };

    let type_id = match word_type.as_str() {
        "gismu" => 1,
        "cmavo" => 2,
        "cmevla" => 3,
        "lujvo" => 4,
        "fu'ivla" => 5,
        "cmavo-compound" => 6,
        "experimental gismu" => 7,
        "experimental cmavo" => 8,
        "bu-letteral" => 9,
        "zei-lujvo" => 10,
        "phrase" => 15,
        _ => 0,
    };

    // Get or create valsi, considering source_langid
    let valsi_id = match transaction
        .query_opt(
            "SELECT valsiid FROM valsi WHERE word = $1 AND source_langid = $2",
            &[&word, &source_langid], // Use determined source_langid
        )
        .await?
    {
        Some(row) => row.get::<_, i32>("valsiid"),
        None => transaction
            .query_one(
                "INSERT INTO valsi (word, typeId, userId, time, source_langid)
                     VALUES ($1, $2, $3, $4, $5)
                     RETURNING valsiid",
                &[
                    &word,
                    &(type_id as i16),
                    &claims.sub,
                    &(Utc::now().timestamp() as i32),
                    &source_langid,
                ],
            )
            .await?
            .get::<_, i32>("valsiid"),
    };

    // Get next definition number
    let definitionnum = transaction
        .query_one(
            "SELECT COALESCE(MAX(definitionnum), 0) + 1
             FROM definitions
             WHERE valsiId = $1 AND langId = $2",
            &[&valsi_id, &request.lang_id],
        )
        .await?
        .get::<_, i32>(0);

    // Get next definition ID
    let definition_id = transaction
        .query_one("SELECT nextval('definitions_definitionid_seq')", &[])
        .await?
        .get::<_, i64>(0) as i32;

    // Add definition
    transaction
        .execute(
            "INSERT INTO definitions
         (definitionid, langId, valsiId, definitionnum, definition, notes, etymology,
          selmaho, jargon, userId, time, owner_only, metadata)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
            &[
                &definition_id,
                &request.lang_id,
                &valsi_id,
                &definitionnum,
                &sanitized_definition,
                &sanitized_notes,
                &sanitized_etymology,
                &sanitized_selmaho,
                &sanitized_jargon,
                &claims.sub,
                &(Utc::now().timestamp() as i32),
                &request.owner_only.unwrap_or(false),
                &request
                    .metadata
                    .clone()
                    .unwrap_or_else(|| serde_json::json!({})),
            ],
        )
        .await?;

    if let Some(image) = &request.image {
        // Decode base64 image data
        let image_data = BASE64
            .decode(&image.data)
            .map_err(|e| format!("Invalid base64 image data: {}", e))?;

        // Insert image
        transaction
            .execute(
                "INSERT INTO definition_images (definition_id, image_data, mime_type)
                 VALUES ($1, $2, $3)",
                &[&definition_id, &image_data, &image.mime_type],
            )
            .await?;
    }

    // Add gloss keywords
    if let Some(gloss_keywords) = &request.gloss_keywords {
        for keyword in gloss_keywords {
            let sanitized_word = sanitize_html(&keyword.word);
            let sanitized_meaning = keyword.meaning.as_ref().map(|m| sanitize_html(m));
            // Add natlangword if it doesn't exist
            transaction
                .execute(
                    "INSERT INTO natlangwords (langid, word, meaning, meaningNum, userId, time)
                 SELECT $1, $2, $3,
                        COALESCE((
                            SELECT MAX(meaningNum) + 1
                            FROM natlangwords
                            WHERE langid = $1 AND word = $2
                        ), 1),
                        $4, $5
                 WHERE NOT EXISTS (
                     SELECT 1 FROM natlangwords
                     WHERE langid = $1
                     AND word = $2
                     AND COALESCE(meaning, '') = COALESCE($3, '')
                 )",
                    &[
                        &request.lang_id,
                        &sanitized_word,
                        &sanitized_meaning,
                        &claims.sub,
                        &(Utc::now().timestamp() as i32),
                    ],
                )
                .await?;

            // Create keywordmapping
            transaction
                .execute(
                    "INSERT INTO keywordmapping (definitionid, place, natlangwordid)
                 SELECT $1, 0, wordid
                 FROM natlangwords
                 WHERE langid = $2 AND word = $3
                 AND COALESCE(meaning, '') = COALESCE($4, '')
                 LIMIT 1",
                    &[
                        &definition_id,
                        &request.lang_id,
                        &sanitized_word,
                        &sanitized_meaning,
                    ],
                )
                .await?;
        }
    }

    // Add place keywords
    if let Some(place_keywords) = &request.place_keywords {
        for (i, keyword) in place_keywords.iter().enumerate() {
            let sanitized_word = sanitize_html(&keyword.word);
            let sanitized_meaning = keyword.meaning.as_ref().map(|m| sanitize_html(m));
            // Add natlangword if it doesn't exist
            transaction
                .execute(
                    "INSERT INTO natlangwords (langid, word, meaning, meaningNum, userId, time)
                 SELECT $1, $2, $3,
                        COALESCE((
                            SELECT MAX(meaningNum) + 1
                            FROM natlangwords
                            WHERE langid = $1 AND word = $2
                        ), 1),
                        $4, $5
                 WHERE NOT EXISTS (
                     SELECT 1 FROM natlangwords
                     WHERE langid = $1
                     AND word = $2
                     AND COALESCE(meaning, '') = COALESCE($3, '')
                 )",
                    &[
                        &request.lang_id,
                        &sanitized_word,
                        &sanitized_meaning,
                        &claims.sub,
                        &(Utc::now().timestamp() as i32),
                    ],
                )
                .await?;

            // Create keywordmapping
            transaction
                .execute(
                    "INSERT INTO keywordmapping (definitionid, place, natlangwordid)
                 SELECT $1, $2, wordid
                 FROM natlangwords
                 WHERE langid = $3 AND word = $4
                 AND COALESCE(meaning, '') = COALESCE($5, '')
                 LIMIT 1",
                    &[
                        &definition_id,
                        &((i + 1) as i32),
                        &request.lang_id,
                        &sanitized_word,
                        &sanitized_meaning,
                    ],
                )
                .await?;
        }
    }

    transaction
        .execute(
            "INSERT INTO definition_versions (
                definition_id, langid, valsiid, definition, notes, etymology, selmaho, jargon,
                gloss_keywords, place_keywords, user_id, message
            )
            SELECT
                d.definitionid, d.langid, d.valsiid, d.definition, d.notes, d.etymology, d.selmaho, d.jargon,
                COALESCE(
                    (SELECT jsonb_agg(to_jsonb(kw))
                     FROM (
                         SELECT n.word, n.meaning
                         FROM keywordmapping k
                         JOIN natlangwords n ON k.natlangwordid = n.wordid
                         WHERE k.definitionid = d.definitionid AND k.place = 0
                     ) kw
                    ), '[]'::jsonb
                ),
                COALESCE(
                    (SELECT jsonb_agg(to_jsonb(kw) ORDER BY kw.place)
                     FROM (
                         SELECT n.word, n.meaning, k.place
                         FROM keywordmapping k
                         JOIN natlangwords n ON k.natlangwordid = n.wordid
                         WHERE k.definitionid = d.definitionid AND k.place > 0
                     ) kw
                    ), '[]'::jsonb
                ),
                $2, 'Updated version'
            FROM definitions d
            WHERE d.definitionid = $1",
            &[&definition_id, &claims.sub],
        )
        .await?;

    let triggers = [
        SubscriptionTrigger::Comment,
        SubscriptionTrigger::Definition,
        SubscriptionTrigger::Edit,
    ];

    for trigger in &triggers {
        transaction
            .execute(
                "INSERT INTO valsi_subscriptions
                     (valsi_id, user_id, trigger_type, source_definition_id)
                     VALUES ($1, $2, $3, $4)
                     ON CONFLICT (valsi_id, user_id, trigger_type) DO NOTHING",
                &[&valsi_id, &claims.sub, trigger, &definition_id],
            )
            .await?;
    }

    // Add vote from creator
    let vote_size: f32 = transaction
        .query_one(
            "SELECT votesize FROM users WHERE userid = $1",
            &[&claims.sub],
        )
        .await?
        .get(0);

    transaction
        .execute(
            "INSERT INTO definitionvotes (valsiid, langid, definitionid, value, userid, time)
             VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &valsi_id,
                &request.lang_id,
                &definition_id,
                &vote_size,
                &claims.sub,
                &(Utc::now().timestamp() as i32),
            ],
        )
        .await?;

    // Only send notifications if enabled
    if send_notifications {
        let valsi_word: String = transaction
            .query_one("SELECT word FROM valsi WHERE valsiid = $1", &[&valsi_id])
            .await?
            .get("word");

        let url = format!("{}/valsi/{}", env::var("FRONTEND_URL")?, valsi_word,);

        transaction
            .execute(
                "SELECT notify_valsi_subscribers($1, 'definition', $2, $3, $4)",
                &[
                    &valsi_id,
                    &format!("New definition added for {}", valsi_word),
                    &url,
                    &claims.sub,
                ],
            )
            .await?;
    }

    if let Err(e) = redis_cache.invalidate("search:*").await {
        log::error!("Failed to invalidate search cache: {}", e);
    }

    Ok((word_type, definition_id))
}

pub async fn get_definition(
    pool: &Pool,
    definition_id: i32,
    user_id: Option<i32>,
    redis_cache: &RedisCache,
) -> Result<Option<DefinitionDetail>, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Get main definition details
    let row = transaction.query_one(
        "
        WITH image_check AS (
            SELECT EXISTS (
                SELECT 1 FROM definition_images
                WHERE definition_id = $1
            ) as has_image
        )
        SELECT d.*, v.word as valsiword, v.valsiid as valsiid, v.source_langid,
                l.realname as langrealname, u.username,
                vt.descriptor as type_name,
                i.has_image,
                (SELECT COUNT(*) FROM threads t
                      LEFT JOIN comments c ON t.threadid = c.threadid
                      WHERE t.valsiid = v.valsiid AND t.definitionid = d.definitionid) as comment_count,
                CASE
                    WHEN $2::int IS NOT NULL THEN can_edit_definition(d.definitionid, $2)
                    ELSE false
                END as can_edit,
                COALESCE((SELECT SUM(value) FROM definitionvotes
                      WHERE definitionid = d.definitionid), 0) as score,
                CASE WHEN $2::int IS NOT NULL THEN
                    (SELECT value::int FROM definitionvotes
                     WHERE userid = $2 AND definitionid = d.definitionid)
                END as user_vote
         FROM definitions d
         JOIN valsi v ON d.valsiid = v.valsiid
         JOIN valsitypes vt ON v.typeid = vt.typeid
         JOIN languages l ON d.langid = l.langid
         CROSS JOIN image_check i
         JOIN users u ON d.userid = u.userid
         WHERE d.definitionid = $1
         AND v.source_langid = 1",
        &[&definition_id, &user_id],
    ).await?;

    let source_langid: i32 = row.get("source_langid");

    let (gloss_keywords, place_keywords) = if source_langid == 1 {
        let gloss_keywords_rows = transaction
            .query(
                "SELECT n.word, n.meaning
         FROM keywordmapping k
         JOIN natlangwords n ON k.natlangwordid = n.wordid
         WHERE k.definitionid = $1 AND k.place = 0",
                &[&definition_id],
            )
            .await?;

        let gloss_keywords: Vec<KeywordMapping> = gloss_keywords_rows
            .into_iter()
            .map(|row| KeywordMapping {
                word: row.get("word"),
                meaning: row.get("meaning"),
            })
            .collect();

        let place_keywords_rows = transaction
            .query(
                "SELECT n.word, n.meaning
                 FROM keywordmapping k
         JOIN natlangwords n ON k.natlangwordid = n.wordid
         WHERE k.definitionid = $1 AND k.place > 0
         ORDER BY k.place",
                &[&definition_id],
            )
            .await?;

        let place_keywords: Vec<KeywordMapping> = place_keywords_rows
            .into_iter()
            .map(|row| KeywordMapping {
                word: row.get("word"),
                meaning: row.get("meaning"),
            })
            .collect();

        (Some(gloss_keywords), Some(place_keywords))
    } else {
        (None, None)
    };

    transaction.commit().await?;

    let word: String = row.get("valsiword");
    let sound_urls = check_sound_urls(&[word.clone()], redis_cache).await;

    let definition = DefinitionDetail {
        similarity: None,
        embedding: None,
        sound_url: sound_urls.get(&word).and_then(|url| url.clone()),
        definitionid: row.get("definitionid"),
        valsiword: row.get("valsiword"),
        valsiid: row.get("valsiid"),
        langid: row.get("langid"),
        definition: row.get("definition"),
        notes: row.get("notes"),
        etymology: row.get("etymology"),
        selmaho: row.get("selmaho"),
        jargon: row.get("jargon"),
        definitionnum: row.get("definitionnum"),
        score: row.get("score"),
        langrealname: row.get("langrealname"),
        username: row.get("username"),
        time: row.get("time"),
        type_name: row.get("type_name"),
        comment_count: row.get("comment_count"),
        user_vote: row.get("user_vote"),
        gloss_keywords,
        place_keywords,
        owner_only: row.get("owner_only"),
        can_edit: row.get("can_edit"),
        created_at: row.get("created_at"),
        has_image: row.get("has_image"),
        metadata: None,
    };

    Ok(Some(definition))
}

pub async fn update_definition(
    pool: &Pool,
    definition_id: i32,
    user_id: i32,
    request: &UpdateDefinitionRequest,
    redis_cache: &RedisCache,
) -> Result<(), Box<dyn std::error::Error>> {
    let sanitized_definition = sanitize_html(&request.definition);
    let sanitized_notes = request.notes.as_ref().map(|n| sanitize_html(n));
    let sanitized_etymology = request.etymology.as_ref().map(|e| sanitize_html(e));
    let sanitized_selmaho = request.selmaho.as_ref().map(|s| sanitize_html(s));
    let sanitized_jargon = request.jargon.as_ref().map(|j| sanitize_html(j));

    // Combine all text fields for validation
    let combined_text = format!(
        "{} {} {}",
        sanitized_definition,
        sanitized_notes.as_deref().unwrap_or(""),
        sanitized_etymology.as_deref().unwrap_or("")
    );

    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Check if we need to create initial version
    let existing_versions: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM definition_versions WHERE definition_id = $1",
            &[&definition_id],
        )
        .await?
        .get(0);

    // If no versions exist, create initial version from current state
    if existing_versions == 0 {
        transaction.execute(
            r#"INSERT INTO definition_versions (
                created_at, definition_id, langid, valsiid, definition, notes, etymology, selmaho, jargon,
                gloss_keywords, place_keywords, user_id, message
            )
            SELECT
                COALESCE(d.time, d.created_at), d.definitionid, d.langid, d.valsiid, d.definition, d.notes, d.etymology, d.selmaho, d.jargon,
                COALESCE(
                    (SELECT jsonb_agg(jsonb_build_object('word', n.word, 'meaning', n.meaning))
                     FROM keywordmapping k
                     JOIN natlangwords n ON k.natlangwordid = n.wordid
                     WHERE k.definitionid = d.definitionid AND k.place = 0),
                    '[]'::jsonb
                ),
                COALESCE(
                    (SELECT jsonb_agg(jsonb_build_object('word', n.word, 'meaning', n.meaning, 'place', k.place))
                     FROM keywordmapping k
                     JOIN natlangwords n ON k.natlangwordid = n.wordid
                     WHERE k.definitionid = d.definitionid AND k.place > 0
                     ORDER BY k.place),
                    '[]'::jsonb
                ),
                d.userid, 'Initial version'
            FROM definitions d
            WHERE d.definitionid = $1"#,
            &[&definition_id],
        ).await?;
    }

    // Get current definition details including owner status and author
    let current_def = transaction
        .query_one(
            "SELECT d.userid, d.owner_only, u.username, v.source_langid
              FROM definitions d
              JOIN users u ON d.userid = u.userid
              JOIN valsi v ON d.valsiid = v.valsiid
              WHERE d.definitionid = $1",
            &[&definition_id],
        )
        .await?;

    let source_langid: Option<i32> = current_def.get("source_langid");

    let options = MathJaxValidationOptions { use_tectonic: true };

    // Only validate MathJax if source lang is Lojban (1) or not set
    if source_langid.is_none() || source_langid == Some(1) {
        validate_mathjax(&combined_text, options)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    }

    let is_author = current_def.get::<_, i32>("userid") == user_id;
    let current_owner_only = current_def.get::<_, bool>("owner_only");
    let author_username = current_def.get::<_, String>("username");

    // Calculate can_edit
    let can_edit = is_author || !current_owner_only;

    if !can_edit {
        return Err(format!(
            "You don't have permission to edit this definition owned by {}",
            author_username
        )
        .into());
    }

    // Determine owner_only value
    let owner_only = if is_author {
        // Author can change owner_only status
        request.owner_only.unwrap_or(false)
    } else {
        // Non-author must keep existing owner_only status
        current_owner_only
    };

    debug!(
        "Updating definition with params: {:?}",
        (
            &request.definition,
            &request.notes,
            &request.jargon,
            Utc::now().timestamp(),
            &request.selmaho,
            &owner_only,
            &request.etymology,
            &definition_id
        )
    );

    // Update main definition
    transaction
        .execute(
            "UPDATE definitions
             SET definition = $1, notes = $2, jargon = $3, time = $4,
                 selmaho = $5, owner_only = $6, etymology = $7,
                 embedding = NULL
             WHERE definitionid = $8",
            &[
                &sanitized_definition,
                &sanitized_notes,
                &sanitized_jargon,
                &(Utc::now().timestamp() as i32),
                &sanitized_selmaho,
                &owner_only,
                &sanitized_etymology,
                &definition_id,
            ],
        )
        .await?;

    if request.remove_image.unwrap_or(false) || request.image.is_some() {
        transaction
            .execute(
                "DELETE FROM definition_images WHERE definition_id = $1",
                &[&definition_id],
            )
            .await?;
    }

    if let Some(image) = &request.image {
        // Decode base64 image data
        let image_data = BASE64
            .decode(&image.data)
            .map_err(|e| format!("Invalid base64 image data: {}", e))?;

        // Insert or update the new image
        transaction
            .execute(
                "INSERT INTO definition_images (definition_id, image_data, mime_type)
                     VALUES ($1, $2, $3)
                     ON CONFLICT (definition_id)
                     DO UPDATE SET image_data = EXCLUDED.image_data,
                                  mime_type = EXCLUDED.mime_type",
                &[&definition_id, &image_data, &image.mime_type],
            )
            .await?;
    }

    // Clear existing keywords
    transaction
        .execute(
            "DELETE FROM keywordmapping WHERE definitionid = $1",
            &[&definition_id],
        )
        .await?;

    // Add gloss keywords
    if let Some(gloss_keywords) = &request.gloss_keywords {
        for keyword in gloss_keywords {
            let sanitized_word = sanitize_html(&keyword.word);
            let sanitized_meaning = keyword.meaning.as_ref().map(|m| sanitize_html(m));
            // First ensure the natlangword exists
            transaction
                .execute(
                    "INSERT INTO natlangwords (langid, word, meaning, meaningNum, userId, time)
                 SELECT $1, $2, $3,
                        COALESCE((
                            SELECT MAX(meaningNum) + 1
                            FROM natlangwords
                            WHERE langid = $1 AND word = $2
                        ), 1),
                        $4, $5
                 WHERE NOT EXISTS (
                     SELECT 1 FROM natlangwords
                     WHERE langid = $1
                     AND word = $2
                     AND COALESCE(meaning, '') = COALESCE($3, '')
                 )",
                    &[
                        &request.lang_id,
                        &sanitized_word,
                        &sanitized_meaning,
                        &user_id,
                        &(Utc::now().timestamp() as i32),
                    ],
                )
                .await?;

            // Then create keywordmapping
            transaction
                .execute(
                    "INSERT INTO keywordmapping (definitionid, place, natlangwordid)
                 SELECT $1, 0, wordid
                 FROM natlangwords
                 WHERE langid = $2 AND word = $3
                 AND COALESCE(meaning, '') = COALESCE($4, '')
                 LIMIT 1",
                    &[
                        &definition_id,
                        &request.lang_id,
                        &sanitized_word,
                        &sanitized_meaning,
                    ],
                )
                .await?;
        }
    }

    // Add place keywords
    if let Some(place_keywords) = &request.place_keywords {
        for (i, keyword) in place_keywords.iter().enumerate() {
            let sanitized_word = sanitize_html(&keyword.word);
            let sanitized_meaning = keyword.meaning.as_ref().map(|m| sanitize_html(m));
            // First ensure the natlangword exists
            transaction
                .execute(
                    "INSERT INTO natlangwords (langid, word, meaning, meaningNum, userId, time)
                 SELECT $1, $2, $3,
                        COALESCE((
                            SELECT MAX(meaningNum) + 1
                            FROM natlangwords
                            WHERE langid = $1 AND word = $2
                        ), 1),
                        $4, $5
                 WHERE NOT EXISTS (
                     SELECT 1 FROM natlangwords
                     WHERE langid = $1
                     AND word = $2
                     AND COALESCE(meaning, '') = COALESCE($3, '')
                 )",
                    &[
                        &request.lang_id,
                        &sanitized_word,
                        &sanitized_meaning,
                        &user_id,
                        &(Utc::now().timestamp() as i32),
                    ],
                )
                .await?;

            // Then create keywordmapping
            transaction
                .execute(
                    "INSERT INTO keywordmapping (definitionid, place, natlangwordid)
                 SELECT $1, $2, wordid
                 FROM natlangwords
                 WHERE langid = $3 AND word = $4
                 AND COALESCE(meaning, '') = COALESCE($5, '')
                 LIMIT 1",
                    &[
                        &definition_id,
                        &((i + 1) as i32),
                        &request.lang_id,
                        &sanitized_word,
                        &sanitized_meaning,
                    ],
                )
                .await?;
        }
    }

    // Create version with new state
    transaction
        .execute(
            "INSERT INTO definition_versions (
                definition_id, langid, valsiid, definition, notes, etymology, selmaho, jargon,
                gloss_keywords, place_keywords, user_id, message
            )
            SELECT
                d.definitionid, d.langid, d.valsiid, d.definition, d.notes, d.etymology, d.selmaho, d.jargon,
                (
                    SELECT COALESCE(json_agg(json_build_object(
                        'word', n.word,
                        'meaning', n.meaning
                    )), '[]'::json)
                    FROM keywordmapping k
                    JOIN natlangwords n ON k.natlangwordid = n.wordid
                    WHERE k.definitionid = d.definitionid AND k.place = 0
                )::jsonb,
                (
                    SELECT COALESCE(json_agg(json_build_object(
                        'word', n.word,
                        'meaning', n.meaning,
                        'place', k.place
                    ) ORDER BY k.place), '[]'::json)
                    FROM keywordmapping k
                    JOIN natlangwords n ON k.natlangwordid = n.wordid
                    WHERE k.definitionid = d.definitionid AND k.place > 0
                )::jsonb,
                $2, 'Updated version'
            FROM definitions d
            WHERE d.definitionid = $1",
            &[&definition_id, &user_id],
        )
        .await?;

    // Get valsi ID for voting
    let valsi_id = transaction
        .query_one(
            "SELECT valsiid FROM definitions WHERE definitionid = $1",
            &[&definition_id],
        )
        .await?
        .get::<_, i32>(0);

    // Get user's vote size
    let vote_size: f32 = transaction
        .query_one("SELECT votesize FROM users WHERE userid = $1", &[&user_id])
        .await?
        .get(0);

    // Add/update vote from editor
    transaction
        .execute(
            "INSERT INTO definitionvotes (valsiid, langid, definitionid, value, userid, time)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (valsiid, langid, userid, definitionid)
             DO UPDATE SET
                value = EXCLUDED.value,
                time = EXCLUDED.time",
            &[
                &valsi_id,
                &request.lang_id,
                &definition_id,
                &vote_size,
                &user_id,
                &(Utc::now().timestamp() as i32),
            ],
        )
        .await?;

    let valsi_word: String = transaction
        .query_one("SELECT word FROM valsi WHERE valsiid = $1", &[&valsi_id])
        .await?
        .get("word");

    let url = format!("{}/valsi/{}", env::var("FRONTEND_URL")?, valsi_word,);

    transaction
        .execute(
            "SELECT notify_valsi_subscribers($1, 'edit', $2, $3, $4)",
            &[
                &valsi_id,
                &format!("Definition updated for {}", valsi_word),
                &url,
                &user_id,
            ],
        )
        .await?;

    if let Err(e) = redis_cache.invalidate("search:*").await {
        log::error!("Failed to invalidate search cache: {}", e);
    }
    transaction.commit().await?;

    Ok(())
}

pub async fn list_definitions(
    pool: &Pool,
    query: &ListDefinitionsQuery,
    current_user_id: Option<i32>,
) -> Result<DefinitionListResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let offset = (page - 1) * per_page;

    let mut conditions: Vec<String> = vec!["TRUE".to_string()];
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let mut param_count = 2; // Start param count at 2 since $1 is current_user_id

    // Add source_langid filter (defaulting to 1 if not provided)
    let source_langid_value = query.source_langid.unwrap_or(1);
    conditions.push(format!("v.source_langid = ${}", param_count));
    params.push(&source_langid_value);
    param_count += 1;

    // Add search condition
    let search_pattern = query.search.as_ref().map(|s| format!("%{}%", s));
    if let Some(pattern) = &search_pattern {
        conditions.push(format!(
            "(v.word ILIKE ${} OR d.definition ILIKE ${} OR d.notes ILIKE ${})",
            param_count, param_count, param_count
        ));
        params.push(pattern);
        param_count += 1;
    }

    // Add user_id filter
    if let Some(user_id) = &query.user_id {
        conditions.push(format!("d.userid = ${}", param_count));
        params.push(user_id);
        param_count += 1;
    }

    // Add word_type filter
    if let Some(word_type) = &query.word_type {
        conditions.push(format!("v.typeid = ${}", param_count));
        params.push(word_type);
        param_count += 1;
    }

    // Add selmaho filter
    if let Some(selmaho) = &query.selmaho {
        conditions.push(format!("d.selmaho = ${}", param_count));
        params.push(selmaho);
        param_count += 1;
    }

    // Add language filter
    let languages_vec: Option<Vec<i32>> = query.languages.as_ref().and_then(|langs| {
        langs
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::parse::<i32>)
            .collect::<Result<Vec<_>, _>>()
            .ok()
    });
    let languages_slice = languages_vec.as_deref();
    if languages_slice.is_some() {
        conditions.push(format!("d.langid = ANY(${})", param_count));
        params.push(&languages_slice);
        param_count += 1;
    }

    let where_clause = conditions
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<&str>>()
        .join(" AND "); // Adjusted join

    // Determine sorting
    let sort_column = match query.sort_by.as_deref() {
        Some("updated_at") => "d.time", // Assuming 'time' is the update timestamp
        _ => "d.created_at",            // Default to creation date
    };
    let sort_order = query
        .sort_order
        .as_deref()
        .map(|s| s.to_uppercase())
        .filter(|s| s == "ASC" || s == "DESC")
        .unwrap_or_else(|| "DESC".to_string());

    // Build the main query
    let query_string = format!(
        r#"
        SELECT
            d.definitionid, d.valsiid, d.langid, d.definition, d.notes, d.etymology, d.created_at,
            d.selmaho, d.jargon, d.definitionnum, d.time, d.owner_only,
            v.word as valsiword,
            u.username,
            l.realname as langrealname,
            vt.descriptor as type_name,
            (SELECT COALESCE(SUM(value), 0) FROM definitionvotes WHERE definitionid = d.definitionid) as score,
            EXISTS(SELECT 1 FROM definition_images WHERE definition_id = d.definitionid) as has_image,
            CASE
                WHEN $1::int IS NOT NULL THEN can_edit_definition(d.definitionid, $1)
                ELSE false
            END as can_edit,
            CASE WHEN $1::int IS NOT NULL THEN
                (SELECT value::int FROM definitionvotes
                 WHERE userid = $1 AND definitionid = d.definitionid)
            END as user_vote
        FROM definitions d
        JOIN valsi v ON d.valsiid = v.valsiid
        JOIN valsitypes vt ON v.typeid = vt.typeid
        JOIN users u ON d.userid = u.userid
        JOIN languages l ON d.langid = l.langid
        WHERE {}
        ORDER BY {} {}, d.definitionid
        LIMIT ${} OFFSET ${}"#,
        where_clause,
        sort_column,
        sort_order,
        param_count,     // Limit param index
        param_count + 1  // Offset param index
    );

    // Add current_user_id to the beginning of params for can_edit and user_vote
    let user_id_value = current_user_id.unwrap_or(0);
    let mut final_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&user_id_value];
    final_params.extend(params);
    final_params.push(&per_page);
    final_params.push(&offset);

    let rows = transaction.query(&query_string, &final_params).await?;

    let definitions: Vec<DefinitionDetail> = rows
        .iter()
        .map(|row| DefinitionDetail {
            similarity: None,
            embedding: None,
            sound_url: None, // Sounds are typically for Lojban words
            definitionid: row.get("definitionid"),
            valsiword: row.get("valsiword"),
            valsiid: row.get("valsiid"),
            langid: row.get("langid"),
            definition: row.get("definition"),
            notes: row.get("notes"),
            etymology: row.get("etymology"),
            selmaho: row.get("selmaho"),
            jargon: row.get("jargon"),
            definitionnum: row.get("definitionnum"),
            langrealname: row.get("langrealname"),
            username: row.get("username"),
            time: row.get("time"),
            type_name: row.get("type_name"),
            score: row.get("score"),
            comment_count: None, // Not requested for this endpoint
            gloss_keywords: None,
            place_keywords: None,
            user_vote: row.get("user_vote"),
            owner_only: row.get("owner_only"),
            can_edit: row.get("can_edit"),
            created_at: row.get("created_at"),
            has_image: row.get("has_image"),
            metadata: None,
        })
        .collect();

    // Count total
    let count_query = format!(
        r#"
        SELECT COUNT(DISTINCT d.definitionid)
        FROM definitions d
        JOIN valsi v ON d.valsiid = v.valsiid
        JOIN users u ON d.userid = u.userid
        WHERE {} and $1::int is not null"#,
        where_clause
    );

    // Use the 'params' vector directly for the count query, excluding current_user_id and pagination params
    let count_params_slice = &final_params[0..final_params.len() - 2]; // Exclude limit, offset

    let total: i64 = transaction
        .query_one(&count_query, count_params_slice)
        .await?
        .get(0);

    transaction.commit().await?;

    Ok(DefinitionListResponse {
        definitions,
        decomposition: Vec::new(), // Decomposition not applicable here
        total,
        page,
        per_page,
    })
}

pub async fn list_non_lojban_definitions(
    pool: &Pool,
    query: NonLojbanDefinitionsQuery,
) -> Result<DefinitionListResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let offset = (page - 1) * per_page;

    let mut query_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let mut conditions = vec!["v.source_langid != 1".to_string()]; // Base condition: not Lojban

    // Add source_langid filter if provided
    if let Some(id) = &query.source_langid {
        conditions.push(format!(
            "v.source_langid = ${}::int",
            query_params.len() + 1
        ));
        query_params.push(id);
    }

    // Add search filter if provided
    let search_pattern = query.search.as_ref().map(|s| format!("%{}%", s));
    if let Some(pattern) = &search_pattern {
        conditions.push(format!(
            "(v.word ILIKE ${} OR d.definition ILIKE ${} OR d.notes ILIKE ${})",
            query_params.len() + 1,
            query_params.len() + 1,
            query_params.len() + 1
        ));
        query_params.push(pattern);
    }

    // Add username filter if provided
    if let Some(username) = &query.username {
        conditions.push(format!("u.username = ${}", query_params.len() + 1));
        query_params.push(username);
    }

    // Add definition language filter if provided
    let languages_vec: Option<Vec<i32>> = query.languages.as_ref().and_then(|langs| {
        langs
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::parse::<i32>)
            .collect::<Result<Vec<_>, _>>()
            .ok()
    });
    let languages_slice = languages_vec.as_deref();
    if languages_slice.is_some() {
        conditions.push(format!("d.langid = ANY(${})", query_params.len() + 1));
        query_params.push(&languages_slice);
    }

    let where_clause = conditions.join(" AND ");

    // Determine sorting
    let sort_by_col = match query.sort_by.as_deref() {
        Some("time") => "d.time",
        Some("score") => "score",
        _ => "lower(v.word)", // Default to word
    };
    let sort_order = query
        .sort_order
        .as_deref()
        .map(|s| s.to_uppercase())
        .filter(|s| s == "ASC" || s == "DESC")
        .unwrap_or_else(|| "ASC".to_string());

    let query_string = format!(
        r#"
        SELECT
            d.definitionid, d.valsiid, d.langid, d.definition, d.notes, d.etymology, d.created_at,
            d.selmaho, d.jargon, d.definitionnum, d.time, d.owner_only,
            v.word as valsiword,
            u.username,
            l.realname as langrealname,
            vt.descriptor as type_name,
            (SELECT COALESCE(SUM(value), 0) FROM definitionvotes WHERE definitionid = d.definitionid) as score,
            EXISTS(SELECT 1 FROM definition_images WHERE definition_id = d.definitionid) as has_image
        FROM definitions d
        JOIN valsi v ON d.valsiid = v.valsiid
        JOIN valsitypes vt ON v.typeid = vt.typeid
        JOIN users u ON d.userid = u.userid
        JOIN languages l ON d.langid = l.langid
        WHERE {}
        ORDER BY {} {}, d.definitionid
        LIMIT ${} OFFSET ${}"#,
        where_clause,
        sort_by_col,
        sort_order,
        query_params.len() + 1, // Limit param index
        query_params.len() + 2  // Offset param index
    );

    query_params.push(&per_page);
    query_params.push(&offset);

    let rows = transaction.query(&query_string, &query_params).await?;

    let definitions: Vec<DefinitionDetail> = rows
        .iter()
        .map(|row| DefinitionDetail {
            similarity: None,
            embedding: None,
            sound_url: None, // Sounds are typically for Lojban words
            definitionid: row.get("definitionid"),
            valsiword: row.get("valsiword"),
            valsiid: row.get("valsiid"),
            langid: row.get("langid"),
            definition: row.get("definition"),
            notes: row.get("notes"),
            etymology: row.get("etymology"),
            selmaho: row.get("selmaho"),
            jargon: row.get("jargon"),
            definitionnum: row.get("definitionnum"),
            langrealname: row.get("langrealname"),
            username: row.get("username"),
            time: row.get("time"),
            type_name: row.get("type_name"),
            score: row.get("score"),
            comment_count: None,  // Comments are typically on Lojban valsi
            gloss_keywords: None, // Keywords are typically for Lojban valsi
            place_keywords: None,
            user_vote: None, // User context not available here
            owner_only: row.get("owner_only"),
            can_edit: false, // User context not available here
            created_at: row.get("created_at"),
            has_image: row.get("has_image"),
            metadata: None,
        })
        .collect();

    // Count total
    let count_query = format!(
        r#"
        SELECT COUNT(DISTINCT d.definitionid)
        FROM definitions d
        JOIN valsi v ON d.valsiid = v.valsiid
        WHERE {}"#,
        where_clause
    );

    // Remove limit and offset params for count query
    let count_params = &query_params[..query_params.len() - 2];

    let total: i64 = transaction
        .query_one(&count_query, count_params)
        .await?
        .get(0);

    transaction.commit().await?;

    Ok(DefinitionListResponse {
        definitions,
        decomposition: Vec::new(), // Decomposition not applicable here
        total,
        page,
        per_page,
    })
}

pub async fn update_vote(
    pool: &Pool,
    redis_cache: &RedisCache,
    user_id: i32,
    definition_id: i32,
    downvote: bool,
) -> Result<(bool, String, Option<String>, Option<i32>), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Get valsi info for the definition
    let valsi_info = transaction
        .query_one(
            "SELECT d.valsiid as valsiid, langid, v.word
             FROM definitions d
             JOIN valsi v ON d.valsiid = v.valsiid
             WHERE definitionid = $1",
            &[&definition_id],
        )
        .await?;

    let valsi_id: i32 = valsi_info.get("valsiid");
    let lang_id: i32 = valsi_info.get("langid");
    let word: String = valsi_info.get("word");

    // Check if there's an existing vote and get its value
    let existing_vote = transaction
        .query_opt(
            "SELECT value FROM definitionvotes
             WHERE valsiid = $1 AND langid = $2 AND userid = $3 AND definitionid = $4",
            &[&valsi_id, &lang_id, &user_id, &definition_id],
        )
        .await?;

    // Get user's vote size
    let vote_size: f32 = transaction
        .query_one("SELECT votesize FROM users WHERE userid = $1", &[&user_id])
        .await?
        .get(0);

    // Delete existing vote
    transaction
        .execute(
            "DELETE FROM definitionvotes
             WHERE valsiid = $1 AND langid = $2 AND userid = $3 AND definitionid = $4",
            &[&valsi_id, &lang_id, &user_id, &definition_id],
        )
        .await?;

    // Determine whether to insert new vote based on existing vote and action
    let should_insert = match existing_vote {
        Some(row) => {
            let existing_value: f32 = row.get(0);
            // Don't insert if downvoting an upvote or upvoting a downvote
            !(downvote && existing_value == 1.0 || !downvote && existing_value == -1.0)
        }
        None => true, // Always insert if no existing vote
    };

    // Insert new vote if needed
    if should_insert {
        transaction
            .execute(
                "INSERT INTO definitionvotes
                 (valsiid, langid, definitionid, value, userid, time)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                &[
                    &valsi_id,
                    &lang_id,
                    &definition_id,
                    &(if downvote { -vote_size } else { vote_size }),
                    &user_id,
                    &(Utc::now().timestamp() as i32),
                ],
            )
            .await?;
    }

    // Update best guesses by recalculating scores
    transaction
        .execute(
            "SELECT reset_valsibestdefinition($1, $2)",
            &[&valsi_id, &lang_id],
        )
        .await?;

    // Get the new total vote value
    let score = transaction
        .query_one(
            "SELECT COALESCE(SUM(value)::int, 0) as total_vote
             FROM definitionvotes
             WHERE definitionid = $1",
            &[&definition_id],
        )
        .await?
        .get::<_, i32>("total_vote");

    if let Err(e) = redis_cache.invalidate("search:*").await {
        log::error!("Failed to invalidate search cache: {}", e);
    }

    // Commit transaction
    transaction.commit().await?;

    let message = if should_insert {
        "Vote recorded successfully"
    } else {
        "Vote removed successfully"
    };

    Ok((true, message.to_string(), Some(word), Some(score)))
}

pub async fn get_definitions_by_entry(
    pool: &Pool,
    id_or_word: &str,
    user_id: Option<i32>,
    preferred_langid: Option<i32>,
    preferred_username: Option<String>,
    redis_cache: &RedisCache,
) -> Result<Vec<DefinitionDetail>, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Build the base query that works for both ID and word lookups
    let base_query = "WITH definition_ranks AS (
            SELECT DISTINCT ON (d.definitionid)
                   d.*,
                   v.word as valsiword,
                   v.valsiid,
                   vt.descriptor as type_name,
                   l.realname as langrealname,
                   u.username,
                   EXISTS(SELECT 1 FROM definition_images WHERE definition_id = d.definitionid) as has_image,
                   CASE
                       WHEN $3::int IS NOT NULL THEN can_edit_definition(d.definitionid, $3)
                       ELSE false
                   END as can_edit,
                   CASE
                       WHEN $4::int IS NOT NULL AND $5::text IS NOT NULL AND d.langid = $4 AND u.username = $5 THEN 1
                       WHEN $4::int IS NOT NULL AND d.langid = $4 THEN 2
                       WHEN $5::text IS NOT NULL AND u.username = $5 THEN 3
                       ELSE 4
                   END as sort_rank
            FROM definitions d
            JOIN languages l ON d.langid = l.langid
            JOIN users u ON d.userid = u.userid
            JOIN valsi v ON d.valsiid = v.valsiid
            JOIN valsitypes vt ON v.typeid = vt.typeid
            WHERE CASE
                WHEN $1 ~ '^\\d+$' THEN v.valsiid = $1::int AND v.source_langid = 1
                ELSE v.word = $2 AND v.source_langid = 1
            END
        )
        SELECT r.*
        FROM definition_ranks r
        ORDER BY r.sort_rank, r.langid, r.time DESC".to_string();

    let mut definitions: Vec<DefinitionDetail> = transaction
        .query(
            &base_query,
            &[
                &id_or_word,         // $1
                &id_or_word,         // $2
                &user_id,            // $3
                &preferred_langid,   // $4
                &preferred_username, // $5
            ],
        )
        .await?
        .iter()
        .map(|row| DefinitionDetail {
            similarity: None,
            embedding: None,
            definitionid: row.get("definitionid"),
            valsiword: row.get("valsiword"),
            valsiid: row.get("valsiid"),
            langid: row.get("langid"),
            definition: row.get("definition"),
            notes: row.get("notes"),
            etymology: row.get("etymology"),
            selmaho: row.get("selmaho"),
            jargon: row.get("jargon"),
            definitionnum: row.get("definitionnum"),
            score: 0.0, // Will be updated later
            langrealname: row.get("langrealname"),
            username: row.get("username"),
            time: row.get("time"),
            type_name: row.get("type_name"),
            comment_count: None, // Will be updated later
            user_vote: None,     // Will be updated later
            gloss_keywords: None,
            place_keywords: None,
            owner_only: row.get("owner_only"),
            can_edit: row.get("can_edit"),
            created_at: row.get("created_at"),
            has_image: row.get("has_image"),
            sound_url: None,
            metadata: None,
        })
        .collect();

    // Rest of the function remains the same...
    if definitions.is_empty() {
        return Ok(definitions);
    }

    let valsi_id = definitions[0].valsiid;

    // Get all definition IDs
    let def_ids: Vec<i32> = definitions.iter().map(|d| d.definitionid).collect();

    // Get comment counts in bulk
    let comment_counts = transaction
        .query(
            "SELECT t.definitionid, COUNT(*) as count
             FROM threads t
             LEFT JOIN comments c ON t.threadid = c.threadid
             WHERE t.valsiid = $1 AND t.definitionid = ANY($2)
             GROUP BY t.definitionid",
            &[&valsi_id, &def_ids],
        )
        .await?;

    // Get vote scores in bulk
    let vote_scores = transaction
        .query(
            "SELECT definitionid, COALESCE(SUM(value), 0) as score
             FROM definitionvotes
             WHERE definitionid = ANY($1)
             GROUP BY definitionid",
            &[&def_ids],
        )
        .await?;

    // Get user votes if user_id is provided
    let user_votes = if let Some(uid) = user_id {
        transaction
            .query(
                "SELECT definitionid, value::int as vote
                 FROM definitionvotes
                 WHERE userid = $1 AND definitionid = ANY($2)",
                &[&uid, &def_ids],
            )
            .await?
    } else {
        Vec::new()
    };

    let words: Vec<String> = definitions.iter().map(|d| d.valsiword.clone()).collect();
    let sound_urls = check_sound_urls(&words, redis_cache).await;

    // Update definitions with the collected data
    for def in &mut definitions {
        def.sound_url = sound_urls.get(&def.valsiword).cloned().flatten(); // Use flatten to fix type mismatch
                                                                           // Update comment count
        if let Some(row) = comment_counts
            .iter()
            .find(|r| r.get::<_, i32>("definitionid") == def.definitionid)
        {
            def.comment_count = Some(row.get("count"));
        }

        // Update score
        if let Some(row) = vote_scores
            .iter()
            .find(|r| r.get::<_, i32>("definitionid") == def.definitionid)
        {
            def.score = row.get("score");
        }

        // Update user vote
        if let Some(row) = user_votes
            .iter()
            .find(|r| r.get::<_, i32>("definitionid") == def.definitionid)
        {
            def.user_vote = Some(row.get("vote"));
        }
    }

    transaction.commit().await?;
    Ok(definitions)
}

pub async fn get_recent_changes(
    pool: &Pool,
    days: i32,
    redis_cache: &RedisCache,
) -> Result<RecentChangesResponse, Box<dyn std::error::Error>> {
    use std::time::Duration as StdDuration;

    // Generate cache key based on days parameter
    let cache_key = format!("recent_changes:{}", days);

    // Cache for 5 minutes (300 seconds) - recent changes should be relatively fresh
    let cache_ttl = StdDuration::from_secs(300);

    // Use Redis cache with get_or_set pattern
    let response = redis_cache
        .get_or_set(
            &cache_key,
            || async {
                let mut client = pool.get().await?;
                let transaction = client.transaction().await?;

                let back_time = Utc::now() - Duration::days(days as i64);
                let back_timestamp = back_time.timestamp() as i32;

                let mut changes = Vec::new();

                // Get base changes first
                let base_changes = transaction
        .query(
            "
        WITH all_changes AS (
            -- Comments
            SELECT
                'comment' AS change_type,
                c.subject AS word,
                c.content AS content,
                t.valsiid,
                d.langid,
                t.natlangwordid,
                c.commentid,
                c.threadid as threadid,
                t.definitionid,
                u.username,
                c.time,
                l.realname AS language_name,
                NULL::integer as version_id,
                NULL::integer as prev_version_id
            FROM comments c
            JOIN threads t ON c.threadid = t.threadid
            JOIN users u ON c.userid = u.userid
            JOIN definitions d ON d.definitionid = t.definitionid
            LEFT JOIN languages l ON d.langid = l.langid
            WHERE c.time > $1 AND u.username != 'officialdata'

            UNION ALL

            -- Definitions
            SELECT
                'definition' AS change_type,
                v.word,
                to_jsonb(dv.message) as content,
                d.valsiid,
                d.langid,
                0 AS natlangwordid,
                0 AS commentid,
                0 AS threadid,
                d.definitionid,
                u.username,
                EXTRACT(EPOCH FROM dv.created_at)::integer as time,
                l.realname AS language_name,
                dv.version_id,
                LAG(dv.version_id) OVER (
                    PARTITION BY d.definitionid
                    ORDER BY dv.created_at
                ) as prev_version_id
            FROM definition_versions dv
            JOIN definitions d ON dv.definition_id = d.definitionid
            JOIN valsi v ON d.valsiid = v.valsiid
            JOIN users u ON dv.user_id = u.userid
            LEFT JOIN languages l ON d.langid = l.langid
            WHERE dv.created_at > to_timestamp($1) AND u.username != 'officialdata' AND v.source_langid = 1

            UNION ALL

            -- Valsi
            SELECT
                'valsi' AS change_type,
                v.word,
                '{}'::jsonb as content,
                v.valsiid,
                0 AS langid,
                0 AS natlangwordid,
                0 AS commentid,
                0 AS threadid,
                0 AS definitionid,
                u.username,
                v.time,
                NULL AS language_name,
                NULL::integer as version_id,
                NULL::integer as prev_version_id
            FROM valsi v
            JOIN users u ON v.userid = u.userid
            WHERE v.time > $1 AND u.username != 'officialdata' AND v.source_langid = 1

            UNION ALL

            -- Messages
            SELECT
                'message' AS change_type,
                COALESCE(m.subject, '') AS word,
                to_jsonb(COALESCE(m.content, '')) as content,
                0 AS valsiid,
                0 AS langid,
                0 AS natlangwordid,
                m.id AS commentid,
                0 AS threadid,
                0 AS definitionid,
                COALESCE(m.from_address, '') AS username,
                EXTRACT(EPOCH FROM m.sent_at)::integer AS time,
                NULL AS language_name,
                NULL::integer as version_id,
                NULL::integer as prev_version_id
            FROM messages m
            WHERE m.sent_at > to_timestamp($1)
            AND NOT EXISTS (
                SELECT 1
                FROM message_spam_votes msv
                WHERE msv.message_id = m.id
            )
        )
        SELECT * FROM all_changes
        ORDER BY time DESC",
            &[&back_timestamp],
        )
        .await?;

                // Process each change and add diffs for definitions
                for row in base_changes {
                    let change_type: String = row.get("change_type");
                    let mut change = RecentChange {
                        change_type: change_type.clone(),
                        word: row.get("word"),
                        content: row.get("content"),
                        valsi_id: if row.get::<_, i32>("valsiid") == 0 {
                            None
                        } else {
                            Some(row.get("valsiid"))
                        },
                        lang_id: if row.get::<_, i32>("langid") == 0 {
                            None
                        } else {
                            Some(row.get("langid"))
                        },
                        natlang_word_id: row
                            .get::<_, Option<i32>>("natlangwordid")
                            .filter(|&id| id != 0),
                        comment_id: row.get::<_, Option<i32>>("commentid").filter(|&id| id != 0),
                        thread_id: row.get::<_, Option<i32>>("threadid").filter(|&id| id != 0),
                        definition_id: row
                            .get::<_, Option<i32>>("definitionid")
                            .filter(|&id| id != 0),
                        username: row.get("username"),
                        time: row.get("time"),
                        language_name: row.get("language_name"),
                        diff: None,
                    };

                    // Add diff for definition changes
                    if change_type == "definition" {
                        if let (Some(version_id), prev_version_id) = (
                            row.get::<_, Option<i32>>("version_id"),
                            row.get::<_, Option<i32>>("prev_version_id"),
                        ) {
                            match if let Some(prev_id) = prev_version_id {
                                get_diff(&transaction, prev_id, version_id).await
                            } else {
                                let current = get_version_with_transaction(&transaction, version_id).await?;
                                let current_definition = current.content.definition.clone();

                                Ok(VersionDiff {
                                    old_content: VersionContent {
                                        definition: String::new(),
                                        notes: None,
                                        selmaho: None,
                                        jargon: None,
                                        gloss_keywords: None,
                                        place_keywords: None,
                                    },
                                    new_content: current.content,
                                    changes: vec![Change {
                                        field: "definition".to_string(),
                                        old_value: Some(String::new()),
                                        new_value: Some(current_definition),
                                        change_type: ChangeType::Added,
                                    }],
                                })
                            } {
                                Ok(diff) => {
                                    change.diff = Some(diff);
                                }
                                Err(e) => {
                                    log::error!("Failed to get diff for version {}: {}", version_id, e);
                                }
                            }
                        }
                    }

                    changes.push(change);
                }

                let total = changes.len() as i64;

                transaction.commit().await?;

                Ok(RecentChangesResponse { changes, total })
            },
            Some(cache_ttl),
        )
        .await?;

    Ok(response)
}

pub async fn get_user_vote(
    pool: &Pool,
    user_id: i32,
    definition_id: i32,
) -> Result<Option<i32>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let vote = client
        .query_opt(
            "SELECT value FROM definitionvotes
             WHERE userid = $1 AND definitionid = $2",
            &[&user_id, &definition_id],
        )
        .await?;

    Ok(vote.map(|row| row.get::<_, f32>(0) as i32))
}

pub async fn get_bulk_user_votes(
    pool: &Pool,
    user_id: i32,
    definition_ids: &[i32],
) -> Result<std::collections::HashMap<String, Option<i32>>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let votes = client
        .query(
            "SELECT definitionid, value::int as vote 
             FROM definitionvotes 
             WHERE userid = $1 AND definitionid = ANY($2)",
            &[&user_id, &definition_ids],
        )
        .await?;

    let mut vote_map = std::collections::HashMap::new();
    for row in votes {
        let def_id: i32 = row.get("definitionid");
        let vote: i32 = row.get("vote");
        vote_map.insert(def_id.to_string(), Some(vote));
    }

    // Add null entries for definitions without votes
    for def_id in definition_ids {
        vote_map.entry(def_id.to_string()).or_insert(None);
    }

    Ok(vote_map)
}

pub async fn get_sitemap(
    pool: &Pool,
    redis_cache: &RedisCache,
) -> Result<String, Box<dyn std::error::Error>> {
    redis_cache
        .get_or_set(
            "sitemap",
            || async {
                let words = get_all_valsi_words(pool).await?;
                let base_url =
                    env::var("FRONTEND_URL").unwrap_or_else(|_| "https://example.com".to_string());

                let mut xml = String::new();
                xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
                xml.push_str(r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);

                for (word, lastmod) in words {
                    xml.push_str("<url>");
                    xml.push_str(&format!("<loc>{}/valsi/{}</loc>", base_url, word));
                    xml.push_str(&format!("<lastmod>{}</lastmod>", lastmod.to_rfc3339()));
                    xml.push_str("<changefreq>weekly</changefreq>");
                    xml.push_str("</url>");
                }

                xml.push_str("</urlset>");
                Ok(xml)
            },
            Some(std::time::Duration::from_secs(3600 * 24)), // Cache for 24 hours
        )
        .await
        .map_err(|e| e.into())
}

pub async fn get_all_valsi_words(
    pool: &Pool,
) -> Result<Vec<(String, DateTime<Utc>)>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let rows = client
        .query(
            "SELECT v.word, v.time as lastmod
         FROM valsi v
         ORDER BY v.time DESC",
            &[],
        )
        .await?;

    let mut words = Vec::new();
    for row in rows {
        let word: String = row.get("word");
        let timestamp: i32 = row.get("lastmod");
        let dt = match Utc.timestamp_opt(timestamp as i64, 0) {
            chrono::LocalResult::Single(dt) => dt,
            // Handle ambiguous times by using earliest possible instance
            chrono::LocalResult::Ambiguous(earliest, _) => earliest,
            // Default to UNIX epoch for invalid timestamps
            chrono::LocalResult::None => Utc.timestamp_opt(0, 0).single().ok_or_else(|| {
                Box::<dyn std::error::Error>::from("Invalid fallback timestamp 0")
            })?,
        };
        words.push((word, dt));
    }
    Ok(words)
}

pub async fn list_valsi_types(pool: &Pool) -> Result<Vec<ValsiType>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let types = client
        .query(
            "SELECT typeid, descriptor
             FROM valsitypes
             ORDER BY typeid",
            &[],
        )
        .await?
        .into_iter()
        .map(ValsiType::from)
        .collect();

    Ok(types)
}

pub async fn bulk_import_definitions(
    pool: &Pool,
    claims: &Claims,
    parsers: Arc<HashMap<i32, Peg>>, // Accept the map
    params: BulkImportParams<'_>,
    broadcaster: &Broadcaster,
    redis_cache: &RedisCache,
    mut cancel_rx: mpsc::Receiver<bool>,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(params.csv_data.as_bytes());

    let mut success_count = 0;
    let mut error_count = 0;

    // Calculate total records without consuming the reader fully if possible
    // Note: This might still read the whole file depending on the CSV structure.
    // A more robust way might involve reading line-by-line twice or passing total count.
    let mut temp_rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(params.csv_data.as_bytes());
    let total_records = temp_rdr.records().count();

    // Send initial progress
    let _ = broadcaster
        .broadcast(
            &params.client_id,
            &serde_json::to_string(&json!({
                "type": "start",
                "total": total_records
            }))
            .unwrap_or_else(|e| {
                log::error!("Failed to serialize start event: {}", e);
                "{}".to_string()
            }),
        )
        .await;

    for (idx, result) in rdr.deserialize().enumerate() {
        // Check for cancellation before processing each record
        if let Ok(true) = cancel_rx.try_recv() {
            log::info!("Cancellation received for job {}", params.client_id);
            return Err("Import cancelled by user".into());
        }

        let record: (String, String, Option<String>, Option<String>) = match result {
            Ok(rec) => rec,
            Err(e) => {
                log::error!("CSV parsing error at line {}: {}", idx + 1, e);
                error_count += 1;
                let _ = broadcaster
                    .broadcast(
                        &params.client_id,
                        &serde_json::to_string(&json!({
                            "type": "progress",
                            "success": false,
                            "word": "N/A",
                            "error": format!("CSV parsing error: {}", e),
                            "current": idx + 1,
                            "total": total_records,
                            "success_count": success_count,
                            "error_count": error_count
                        }))
                        .unwrap_or_else(|e| {
                            log::error!("Failed to serialize progress error event: {}", e);
                            "{}".to_string()
                        }),
                    )
                    .await;
                continue; // Skip to the next record
            }
        };

        let (gismu, definition, notes, glosswords) = record;

        // Parse glosswords into KeywordMapping vec
        let gloss_keywords = glosswords
            .map(|gs| {
                gs.split(',')
                    .filter_map(|pair| {
                        let parts: Vec<&str> = pair.splitn(2, ';').collect();
                        if parts.is_empty() || parts[0].trim().is_empty() {
                            None
                        } else {
                            Some(KeywordMapping {
                                word: parts[0].trim().to_string(),
                                meaning: parts.get(1).map(|s| s.trim().to_string()),
                            })
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let request = AddDefinitionRequest {
            source_langid: None, // Assuming Lojban source for bulk import
            word: gismu.clone(),
            definition,
            notes,
            etymology: None,
            lang_id: params.lang_id,
            selmaho: None, // Selmaho might need to be derived or provided
            jargon: None,
            gloss_keywords: Some(gloss_keywords),
            place_keywords: None,
            owner_only: Some(false),
            image: None,
            metadata: Some(serde_json::json!({
                "bulk_import": true,
                "client_id": params.client_id,
                "import_time": params.import_time,
            })),
        };

        // Pass the parser map to add_definition
        match add_definition(pool, claims, parsers.clone(), &request, redis_cache, false).await {
            Ok(_) => {
                success_count += 1;
                let _ = broadcaster
                    .broadcast(
                        &params.client_id,
                        &serde_json::to_string(&json!({
                            "type": "progress",
                            "success": true,
                            "word": gismu.clone(),
                            "current": idx + 1,
                            "total": total_records,
                            "success_count": success_count,
                            "error_count": error_count
                        }))
                        .unwrap_or_else(|e| {
                            log::error!("Failed to serialize progress success event: {}", e);
                            "{}".to_string()
                        }),
                    )
                    .await;
            }
            Err(e) => {
                log::error!("Failed to import definition for '{}': {}", gismu, e);
                error_count += 1;
                let _ = broadcaster
                    .broadcast(
                        &params.client_id,
                        &serde_json::to_string(&json!({
                            "type": "progress",
                            "success": false,
                            "word": gismu,
                            "error": e.to_string(),
                            "current": idx + 1,
                            "total": total_records,
                            "success_count": success_count,
                            "error_count": error_count
                        }))
                        .unwrap_or_else(|e| {
                            log::error!("Failed to serialize progress error event: {}", e);
                            "{}".to_string()
                        }),
                    )
                    .await;
            }
        }
    }

    log::info!(
        "Bulk import finished for client {}. Success: {}, Errors: {}",
        params.client_id,
        success_count,
        error_count
    );
    Ok((success_count, error_count))
}

pub async fn delete_definition(
    pool: &Pool,
    definition_id: i32,
    user_id: i32,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Check if user is the author
    let is_author = transaction
        .query_one(
            "SELECT userid FROM definitions WHERE definitionid = $1",
            &[&definition_id],
        )
        .await?
        .get::<_, i32>("userid")
        == user_id;

    if !is_author {
        return Err("Only the author can delete their definition".into());
    }

    // Check if definition has any comments
    let has_comments = transaction
        .query_one(
            "SELECT EXISTS(
                SELECT 1 FROM threads
                WHERE definitionid = $1
                AND EXISTS(SELECT 1 FROM comments WHERE comments.threadid = threads.threadid)
            )",
            &[&definition_id],
        )
        .await?
        .get::<_, bool>(0);

    if has_comments {
        return Ok(false);
    }

    // Delete related records first
    transaction
        .execute(
            "DELETE FROM keywordmapping WHERE definitionid = $1",
            &[&definition_id],
        )
        .await?;

    transaction
        .execute(
            "DELETE FROM definitionvotes WHERE definitionid = $1",
            &[&definition_id],
        )
        .await?;

    transaction
        .execute(
            "DELETE FROM definition_images WHERE definition_id = $1",
            &[&definition_id],
        )
        .await?;

    transaction
        .execute(
            "DELETE FROM definition_versions WHERE definition_id = $1",
            &[&definition_id],
        )
        .await?;

    // Delete related subscriptions
    transaction
        .execute(
            "DELETE FROM valsi_subscriptions WHERE source_definition_id = $1",
            &[&definition_id],
        )
        .await?;

    // Delete the definition itself
    let deleted = transaction
        .execute(
            "DELETE FROM definitions WHERE definitionid = $1",
            &[&definition_id],
        )
        .await?;

    transaction.commit().await?;

    Ok(deleted > 0)
}

pub async fn add_definition_image(
    pool: &Pool,
    definition_id: i32,
    user_id: i32,
    image: &ImageData,
    description: Option<&str>,
) -> Result<i32, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Check if user can edit this definition
    let can_edit: bool = transaction
        .query_one(
            "SELECT can_edit_definition($1, $2)",
            &[&definition_id, &user_id],
        )
        .await?
        .get(0);

    if !can_edit {
        return Err("User not authorized to add images to this definition".into());
    }

    // Get next display order
    let display_order: i32 = transaction
        .query_one(
            "SELECT COALESCE(MAX(display_order), -1) + 1
             FROM definition_images
             WHERE definition_id = $1",
            &[&definition_id],
        )
        .await?
        .get(0);

    // Decode and process image
    let image_data = base64::engine::general_purpose::STANDARD
        .decode(&image.data)
        .map_err(|e| format!("Invalid base64 image data: {}", e))?;

    // Insert image
    let image_id = transaction
        .query_one(
            "INSERT INTO definition_images
             (definition_id, image_data, mime_type, description, display_order, created_by)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id",
            &[
                &definition_id,
                &image_data,
                &image.mime_type,
                &description,
                &display_order,
                &user_id,
            ],
        )
        .await?
        .get::<_, i32>("id");

    transaction.commit().await?;

    Ok(image_id)
}

pub async fn delete_bulk_definitions(
    pool: &Pool,
    client_id: &str,
) -> Result<(Vec<i32>, Vec<i32>), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Get all definitions with this client_id in metadata
    let definitions = transaction
        .query(
            "SELECT definitionid FROM definitions 
            WHERE metadata->>'client_id' = $1", // Use ->> to extract as text
            &[&client_id],
        )
        .await?;

    let mut deleted = Vec::new();
    let mut skipped = Vec::new();

    for row in definitions {
        let def_id: i32 = row.get("definitionid");

        // Check if definition has any comments
        let has_comments: bool = transaction
            .query_one(
                "SELECT EXISTS(
                    SELECT 1 FROM threads t
                    JOIN comments c ON t.threadid = c.threadid
                    WHERE t.definitionid = $1
                )",
                &[&def_id],
            )
            .await?
            .get(0);

        if has_comments {
            skipped.push(def_id);
            continue;
        }

        // Delete related records
        transaction
            .execute(
                "DELETE FROM keywordmapping WHERE definitionid = $1",
                &[&def_id],
            )
            .await?;

        transaction
            .execute(
                "DELETE FROM definitionvotes WHERE definitionid = $1",
                &[&def_id],
            )
            .await?;

        transaction
            .execute(
                "DELETE FROM definition_images WHERE definition_id = $1",
                &[&def_id],
            )
            .await?;

        transaction
            .execute(
                "DELETE FROM definition_versions WHERE definition_id = $1",
                &[&def_id],
            )
            .await?;

        // Delete the definition itself
        transaction
            .execute(
                "DELETE FROM definitions WHERE definitionid = $1",
                &[&def_id],
            )
            .await?;

        deleted.push(def_id);
    }

    transaction.commit().await?;
    Ok((deleted, skipped))
}

pub async fn get_definition_image(
    pool: &Pool,
    definition_id: i32,
    query: GetImageDefinitionQuery,
) -> Result<Option<(Vec<u8>, String)>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let result = if let Some(pos) = query.image_id {
        client
            .query_opt(
                "SELECT image_data, mime_type
             FROM definition_images
             WHERE definition_id = $1 AND id = $2",
                &[&definition_id, &pos],
            )
            .await?
    } else {
        client
            .query_opt(
                "SELECT image_data, mime_type
             FROM definition_images
             WHERE definition_id = $1
             ORDER BY display_order
             LIMIT 1",
                &[&definition_id],
            )
            .await?
    };

    Ok(result.map(|row| (row.get::<_, Vec<u8>>("image_data"), row.get("mime_type"))))
}

pub fn validate_image(image: &ImageData) -> Result<(), String> {
    // Validate mime type
    if !["image/jpeg", "image/png", "image/gif", "image/webp"].contains(&image.mime_type.as_str()) {
        return Err("Invalid image type. Supported types: JPEG, PNG, GIF, WebP".to_string());
    }

    let decoded_size = BASE64
        .decode(&image.data)
        .map_err(|_| "Invalid base64 data".to_string())?
        .len();

    if decoded_size > 5 * 1024 * 1024 {
        return Err("Image size exceeds 5MB limit".to_string());
    }

    Ok(())
}

pub async fn list_bulk_import_client_groups(
    pool: &Pool,
) -> Result<Vec<ClientIdGroup>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let rows = client
        .query(
            r#"
            SELECT
                metadata->>'client_id' AS client_id,
                COUNT(*) AS count
            FROM definitions
            WHERE metadata->>'bulk_import' = 'true' AND metadata->>'client_id' IS NOT NULL
            GROUP BY metadata->>'client_id'
            ORDER BY client_id
            "#,
            &[],
        )
        .await?;

    let mut groups = Vec::new();
    for row in rows {
        groups.push(ClientIdGroup {
            client_id: row.get("client_id"),
            count: row.get("count"),
        });
    }

    Ok(groups)
}

pub async fn list_definitions_by_client_id(
    pool: &Pool,
    client_id: &str,
    page: i64,
    per_page: i64,
    user_id: Option<i32>, // Needed for can_edit
) -> Result<DefinitionListResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let offset = (page - 1) * per_page;

    // Base query - similar to list_definitions but with metadata and client_id filter
    let query_string = r#"
        SELECT
            d.definitionid, d.valsiid, d.langid, d.definition, d.notes, d.etymology, d.created_at,
            d.selmaho, d.jargon, d.definitionnum, d.time, d.owner_only, d.metadata,
            v.word as valsiword,
            u.username,
            l.realname as langrealname,
            vt.descriptor as type_name,
            EXISTS(SELECT 1 FROM definition_images WHERE definition_id = d.definitionid) as has_image,
            (d.userid = $4) as can_edit
        FROM definitions d
        JOIN valsi v ON d.valsiid = v.valsiid
        JOIN valsitypes vt ON v.typeid = vt.typeid
        JOIN users u ON d.userid = u.userid
        JOIN languages l ON d.langid = l.langid
        WHERE d.metadata->>'client_id' = $1
        ORDER BY d.created_at DESC -- Or another sensible default order
        LIMIT $2 OFFSET $3
    "#;

    let rows = transaction
        .query(query_string, &[&client_id, &per_page, &offset, &user_id])
        .await?;

    let def_ids: Vec<i32> = rows.iter().map(|row| row.get("definitionid")).collect();

    let (gloss_keywords_map, place_keywords_map) = fetch_keywords(&transaction, &def_ids).await?;

    let mut definitions: Vec<DefinitionDetail> = Vec::new();
    for row in rows {
        let def_id: i32 = row.get("definitionid");
        let word: String = row.get("valsiword");

        definitions.push(DefinitionDetail {
            definitionid: def_id,
            valsiword: word.clone(),
            valsiid: row.get("valsiid"),
            langid: row.get("langid"),
            definition: row.get("definition"),
            notes: row.get("notes"),
            etymology: row.get("etymology"),
            selmaho: row.get("selmaho"),
            jargon: row.get("jargon"),
            definitionnum: row.get("definitionnum"),
            langrealname: row.get("langrealname"),
            username: row.get("username"),
            time: row.get("time"),
            created_at: row.get("created_at"),
            type_name: row.get("type_name"),
            score: 0.,
            comment_count: None,
            gloss_keywords: gloss_keywords_map.get(&def_id).cloned(),
            place_keywords: place_keywords_map.get(&def_id).cloned(),
            user_vote: None,
            owner_only: row.get("owner_only"),
            can_edit: row.get("can_edit"),
            has_image: row.get("has_image"),
            sound_url: None,
            embedding: None,
            similarity: None,
            metadata: row.get("metadata"),
        });
    }

    // Get total count
    let count_query = r#"
        SELECT COUNT(*)
        FROM definitions d
        WHERE d.metadata->>'client_id' = $1
    "#;
    let total_row = transaction.query_one(count_query, &[&client_id]).await?;
    let total: i64 = total_row.get(0);

    transaction.commit().await?;

    Ok(DefinitionListResponse {
        definitions,
        total,
        page,
        per_page,
        decomposition: Vec::new(),
    })
}
