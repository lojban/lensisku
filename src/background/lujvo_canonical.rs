//! Background drain: classify unchecked classical lujvo (`typeid = 4`,
//! `canonical_word IS NULL`) into canonical vs non-canonical via reconstruct_lujvo.
//!
//! Kept deliberately gentle: `reconstruct_lujvo` / jvozba is CPU+RAM heavy, and
//! flipping `typeid` fires `sync_definition_cache_fields` (per-definition rewrite).
//! Classify outside a DB transaction; write in small batches; load rafsi maps once
//! per tick.

use crate::language::{classify_lujvo_spelling, load_owned_rafsi_maps, LujvoClassification};
use deadpool_postgres::Pool;
use log::{debug, error, info};
use std::time::Duration;
use tokio::time;

/// Rows to classify and write per tick (also the SELECT limit).
const BATCH_SIZE: i64 = 50;
const INTERVAL_SECS: u64 = 120;

pub fn spawn_lujvo_canonical_classification(pool: Pool) {
    tokio::spawn(async move {
        if std::env::var("DISABLE_LUJVO_CANONICAL")
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false)
        {
            info!("DISABLE_LUJVO_CANONICAL set; skipping lujvo canonical classification");
            return;
        }

        let mut interval = time::interval(Duration::from_secs(INTERVAL_SECS));
        // Don't slam the DB the instant the server starts (migrations / warm-up).
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(e) = classify_unchecked_lujvo_batch(&pool).await {
                error!("lujvo canonical classification batch failed: {}", e);
            }
        }
    });
}

/// Returns `(updated, non_canonical)`.
pub async fn classify_unchecked_lujvo_batch(
    pool: &Pool,
) -> Result<(usize, usize), Box<dyn std::error::Error + Send + Sync>> {
    // 1) Short read: pending ids + load maps once.
    let (pending, maps) = {
        let mut client = pool.get().await?;
        let transaction = client.transaction().await?;
        let rows = transaction
            .query(
                "SELECT valsiid, word FROM valsi
                 WHERE typeid = 4 AND source_langid = 1 AND canonical_word IS NULL
                 ORDER BY valsiid
                 LIMIT $1",
                &[&BATCH_SIZE],
            )
            .await?;
        if rows.is_empty() {
            transaction.commit().await?;
            debug!("lujvo canonical classification: nothing pending");
            return Ok((0, 0));
        }
        let pending: Vec<(i32, String)> = rows
            .iter()
            .map(|r| (r.get("valsiid"), r.get("word")))
            .collect();
        let maps = load_owned_rafsi_maps(&transaction)
            .await
            .map_err(|e| e.to_string())?;
        transaction.commit().await?;
        (pending, maps)
    };

    // 2) CPU-heavy classify with no DB connection held.
    let options = maps.options();
    let mut plans: Vec<(i32, LujvoClassification)> = Vec::with_capacity(pending.len());
    let mut self_checked: Vec<i32> = Vec::new();
    for (valsiid, word) in pending {
        match classify_lujvo_spelling(&word, &options) {
            Some(class) => plans.push((valsiid, class)),
            None => self_checked.push(valsiid),
        }
    }

    // 3) Short writes (typeid changes still fire definition-cache sync — keep batches small).
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    let mut updated = 0usize;
    let mut non_canonical = 0usize;

    for valsiid in self_checked {
        transaction
            .execute(
                "UPDATE valsi SET canonical_word = word WHERE valsiid = $1 AND canonical_word IS NULL",
                &[&valsiid],
            )
            .await?;
        updated += 1;
    }

    for (valsiid, class) in plans {
        if class.type_id == 17 {
            non_canonical += 1;
        }
        transaction
            .execute(
                "UPDATE valsi SET typeid = $1, canonical_word = $2
                 WHERE valsiid = $3 AND typeid = 4 AND canonical_word IS NULL",
                &[&class.type_id, &class.canonical_word, &valsiid],
            )
            .await?;
        updated += 1;
    }

    transaction.commit().await?;

    if updated == 0 {
        debug!("lujvo canonical classification: nothing pending");
    } else {
        info!(
            "lujvo canonical classification: updated {updated} (non-canonical {non_canonical})"
        );
    }
    Ok((updated, non_canonical))
}
