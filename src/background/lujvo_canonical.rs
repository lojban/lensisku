//! Background drain: classify unchecked classical lujvo (`typeid = 4`,
//! `canonical_word IS NULL`) into canonical vs non-canonical via reconstruct_lujvo.

use crate::language::{classify_lujvo_spelling, load_owned_rafsi_maps};
use deadpool_postgres::Pool;
use log::{debug, error, info};
use std::time::Duration;
use tokio::time;

const BATCH_SIZE: i64 = 50;
const INTERVAL_SECS: u64 = 5 * 60;

pub fn spawn_lujvo_canonical_classification(pool: Pool) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(INTERVAL_SECS));
        loop {
            interval.tick().await;
            if let Err(e) = classify_unchecked_lujvo_batch(&pool).await {
                error!("lujvo canonical classification batch failed: {}", e);
            }
        }
    });
}

pub async fn classify_unchecked_lujvo_batch(
    pool: &Pool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        debug!("lujvo canonical classification: nothing pending");
        transaction.commit().await?;
        return Ok(());
    }

    let maps = load_owned_rafsi_maps(&transaction)
        .await
        .map_err(|e| e.to_string())?;
    let options = maps.options();

    let mut updated = 0usize;
    let mut non_canonical = 0usize;
    for row in &rows {
        let valsiid: i32 = row.get("valsiid");
        let word: String = row.get("word");
        let Some(class) = classify_lujvo_spelling(&word, &options) else {
            // Not classifiable — mark checked as self so we do not retry forever.
            transaction
                .execute(
                    "UPDATE valsi SET canonical_word = word WHERE valsiid = $1 AND canonical_word IS NULL",
                    &[&valsiid],
                )
                .await?;
            updated += 1;
            continue;
        };

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
    info!(
        "lujvo canonical classification: updated {updated} (non-canonical {non_canonical})"
    );
    Ok(())
}
