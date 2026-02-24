use crate::{
    db,
    error::{AppError, AppResult},
    export::service::export_all_dictionaries,
    mailarchive::{check_for_new_emails, import_maildir},
    muplis,
    notifications::run_email_notifications,
};
use chrono::Local;
use deadpool_postgres::Pool;
use log::{error, info};
use pgvector::Vector;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::Mutex,
    time::{self, sleep},
};

/// Types where definition notes are known to skew embeddings (e.g. boilerplate "experimental" text).
/// When we have no glosswords, we use only definition and exclude notes for these types.
fn skip_notes_for_embedding_type(type_name: &str) -> bool {
    matches!(
        type_name.to_lowercase().as_str(),
        "experimental cmavo"
            | "experimental gismu"
            | "obsolete cmavo"
            | "obsolete gismu"
            | "obsolete zei-lujvo"
    )
}

async fn calculate_missing_embeddings(pool: &Pool) -> AppResult<()> {
    if crate::embeddings::embeddings_disabled() {
        info!("Embeddings disabled (DISABLE_EMBEDDINGS); skipping calculation.");
        return Ok(());
    }
    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::Database(format!("Failed to get database connection: {}", e)))?;

    // Get all definitions needing embeddings, along with their valsi type, glosswords, and place keywords
    let rows = conn
        .query(
            "SELECT d.definitionid, d.definition, coalesce(d.notes, '') as notes, d.langid, vt.descriptor as type_name,
             (SELECT string_agg(n.word, ' ')
              FROM keywordmapping k
              JOIN natlangwords n ON k.natlangwordid = n.wordid
              WHERE k.definitionid = d.definitionid AND k.place = 0) as glosswords,
             (SELECT string_agg(n.word, ' ')
              FROM keywordmapping k
              JOIN natlangwords n ON k.natlangwordid = n.wordid
              WHERE k.definitionid = d.definitionid AND k.place > 0) as placewords
             FROM definitions d
             JOIN valsi v ON d.valsiid = v.valsiid
             JOIN valsitypes vt ON v.typeid = vt.typeid
             WHERE d.embedding IS NULL AND d.definition != '' AND d.langid != 1",
            &[],
        )
        .await
        .map_err(|e| AppError::Database(format!("Failed to query definitions: {}", e)))?;

    // Prepare all texts for embedding
    let mut all_texts: Vec<String> = Vec::new();
    let mut all_definition_ids: Vec<i32> = Vec::new();

    for row in &rows {
        let definition_id: i32 = row.get("definitionid");
        let definition: String = row.get("definition");
        let notes: String = row.get("notes");
        let glosswords: String = row
            .get::<_, Option<String>>("glosswords")
            .unwrap_or_default();
        let placewords: String = row
            .get::<_, Option<String>>("placewords")
            .unwrap_or_default();

        if row.get::<_, i32>("langid") == 1 {
            continue;
        }
        let type_name: String = row.get("type_name");

        // Combine text for embedding
        let mut text_parts = Vec::new();

        // If glosswords exist, use them as the primary source to avoid noise from lengthy definitions/notes.
        if !glosswords.trim().is_empty() {
            text_parts.push(glosswords);
        } else {
            // Fallback to definition (+ notes only when notes are not known to skew embeddings)
            let def_len = definition.len().max(1);
            text_parts.push(definition);
            let add_notes = !skip_notes_for_embedding_type(&type_name)
                && !notes.trim().is_empty()
                && !(type_name.eq_ignore_ascii_case("fu'ivla") && notes.len() > 2 * def_len);
            if add_notes {
                text_parts.push(notes);
            }
        }

        // Always include placewords as they capture key semantic roles
        if !placewords.trim().is_empty() {
            text_parts.push(placewords);
        }

        let mut combined_text = text_parts.join(" ");

        // Append " (name)" if the type is cmevla or obsolete cmevla
        if type_name == "cmevla" || type_name == "obsolete cmevla" {
            combined_text.push_str(" (name)");
        }

        let processed_text = combined_text.trim().to_string();
        if processed_text.is_empty() {
            continue;
        }
        all_texts.push(processed_text);
        all_definition_ids.push(definition_id);
    }

    // Process in chunks of 100
    for (texts_chunk, ids_chunk) in all_texts.chunks(100).zip(all_definition_ids.chunks(100)) {
        let transaction = conn
            .transaction()
            .await
            .map_err(|e| AppError::Database(format!("Failed to start transaction: {}", e)))?;

        info!(
            "Calculating embeddings for batch of {} definitions",
            texts_chunk.len()
        );

        // Generate embeddings in-process via fastembed (AllMiniLML6V2, mean pooling, L2-normalised)
        let embeddings = crate::embeddings::get_batch_embeddings(texts_chunk.to_vec()).await?;

        for (i, (embedding, processed_text)) in
            embeddings.into_iter().zip(texts_chunk.iter()).enumerate()
        {
            let definition_id = match ids_chunk.get(i) {
                Some(id) => id,
                None => {
                    log::warn!("Index out of bounds for definition_ids: {}", i);
                    continue;
                }
            };

            // fastembed already L2-normalises — no manual normalisation needed
            let pg_vector = Vector::from(embedding);

            transaction
                .execute(
                    "UPDATE definitions
                     SET embedding = $1,
                         metadata = COALESCE(metadata, '{}'::jsonb) || jsonb_build_object('processed_text', $3::text)
                     WHERE definitionid = $2",
                    &[&pg_vector, definition_id, processed_text],
                )
                .await
                .map_err(|e| AppError::Database(format!("Failed to update definition: {}", e)))?;
        }

        transaction
            .commit()
            .await
            .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;
    }

    Ok(())
}

pub async fn spawn_background_tasks(pool: Pool, maildir_path: String) {
    let pool_clone = pool.clone();
    let maildir_path_clone = maildir_path.clone();
    tokio::spawn(async move {
        if let Err(e) = import_maildir(&pool_clone, &maildir_path_clone).await {
            error!("Failed to import emails from Maildir: {:?}", e);
        }

        // Verify database has messages
        match db::get_message_count(&pool_clone).await {
            Ok(count) => {
                info!("Number of emails in the database: {}", count);
                if count == 0 {
                    error!(
                        "The messages table is empty. Please ensure the database is properly populated."
                    );
                }
            }
            Err(e) => {
                error!("Failed to get message count: {}", e);
            }
        }
    });

    // Embedding calculation task (in-process via fastembed — no external service needed)
    let embedding_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60 * 60)); // Run hourly
        loop {
            interval.tick().await;

            if let Err(e) = calculate_missing_embeddings(&embedding_pool).await {
                error!("Failed to calculate embeddings: {}", e);
            }
        }
    });

    // Update muplis data periodically
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(24 * 60 * 60)); // Daily
        loop {
            interval.tick().await;
            if let Err(e) = muplis::update_if_needed(&pool_clone).await {
                error!("Failed to update muplis data: {}", e);
            }
        }
    });

    // Check for new emails periodically
    let pool_clone = pool.clone();
    let maildir_path_clone = maildir_path.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1 * 24 * 60 * 60)); // 1 day
        loop {
            interval.tick().await;
            if let Err(e) = check_for_new_emails(&pool_clone, &maildir_path_clone).await {
                error!("Failed to check for new emails: {}", e);
            }
        }
    });

    // Spawn email notification processor
    let email_pool = pool.clone();
    tokio::spawn(async move {
        run_email_notifications(email_pool).await;
    });

    // Cache dictionary exports
    let export_lock = Arc::new(Mutex::new(()));
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        loop {
            let now = Local::now();
            let next_midnight = match (now + chrono::Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
            {
                Some(time) => time,
                None => {
                    error!("Failed initial midnight calculation, trying alternative approach");
                    match now
                        .naive_local()
                        .date()
                        .checked_add_days(chrono::Days::new(1))
                    {
                        Some(next_date) => match next_date.and_hms_opt(0, 0, 0) {
                            Some(time) => time,
                            None => {
                                error!("Failed to set time on next date, using 24hr incremental approach");
                                now.naive_local().checked_add_signed(chrono::Duration::hours(24))
                                    .and_then(|dt| dt.date().and_hms_opt(0, 0, 0))
                                    .unwrap_or_else(|| {
                                        error!("All fallback methods failed, using current time + 1 minute");
                                        now.naive_local() + chrono::Duration::minutes(1)
                                    })
                            }
                        },
                        None => {
                            error!("Date overflow, using current time + 1 minute");
                            now.naive_local() + chrono::Duration::minutes(1)
                        }
                    }
                }
            };
            let duration_until_midnight = next_midnight.signed_duration_since(now.naive_local());
            let sleep_duration = Duration::from_secs(duration_until_midnight.num_seconds() as u64);

            sleep(sleep_duration).await;

            // Acquire lock before starting export
            let _lock = export_lock.lock().await;
            if let Err(e) = export_all_dictionaries(&pool_clone).await {
                error!("Failed to export dictionaries: {}", e);
            }
        }
    });
}
