use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use deadpool_postgres::Pool;
use log::{error, info};
use tokio::time::{sleep, Duration};

const BATCH_LIMIT: i64 = 20;

pub fn spawn_valsi_sound_generation(pool: Pool) {
    let running = Arc::new(AtomicBool::new(false));
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(5 * 60)).await;

            if std::env::var("DISABLE_VALSI_TTS")
                .ok()
                .as_deref()
                == Some("1")
            {
                continue;
            }

            if running
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
            {
                log::debug!("valsi TTS: previous run still in progress, skipping this interval");
                continue;
            }

            let pool = pool.clone();
            let running = running.clone();
            tokio::spawn(async move {
                let result = run_valsi_sound_batch(&pool).await;
                running.store(false, Ordering::Release);
                match result {
                    Ok(n) if n > 0 => {
                        info!("valsi TTS: generated {} sound(s)", n);
                    }
                    Ok(_) => {}
                    Err(e) => error!("valsi TTS batch failed: {}", e),
                }
            });
        }
    });
}

async fn run_valsi_sound_batch(pool: &Pool) -> Result<usize, String> {
    let rows: Vec<(i32, String)> = {
        let client = pool
            .get()
            .await
            .map_err(|e| format!("db pool: {e}"))?;
        let rows = client
            .query(
                "SELECT v.valsiid, v.word
                 FROM valsi v
                 LEFT JOIN valsi_sounds vs ON vs.valsi_id = v.valsiid
                 WHERE v.source_langid = 1
                   AND vs.valsi_id IS NULL
                   AND coalesce(
                       array_length(
                           regexp_split_to_array(trim(both from v.word), '[[:space:]]+'),
                           1
                       ),
                       0
                     ) <= 5
                 ORDER BY v.valsiid
                 LIMIT $1",
                &[&BATCH_LIMIT],
            )
            .await
            .map_err(|e| e.to_string())?;
        rows
            .into_iter()
            .map(|row| (row.get("valsiid"), row.get("word")))
            .collect()
    };

    if rows.is_empty() {
        return Ok(0);
    }

    // One blocking task: load ONNX into RAM, synthesize every row, then drop the session so memory
    // is released until the next interval. `ensure_model_files_cached` inside `load_blocking` only
    // downloads HF artifacts once per process.
    let synthesized: Vec<(i32, Vec<u8>, String)> =
        tokio::task::spawn_blocking(move || {
            let engine = crate::utils::kitten_tts::KittenTts::load_blocking()?;
            let mut out = Vec::new();
            for (valsi_id, word) in rows {
                if word.split_whitespace().count() > 5 {
                    continue;
                }
                let ogg = engine.lojban_word_to_ogg_opus(&word)?;
                out.push((valsi_id, ogg, word));
            }
            Ok::<_, String>(out)
        })
        .await
        .map_err(|e| format!("join: {e}"))??;

    let mut done = 0usize;
    for (valsi_id, ogg, word) in synthesized {
        let client = pool
            .get()
            .await
            .map_err(|e| format!("db pool: {e}"))?;
        let insert = client
            .execute(
                "INSERT INTO valsi_sounds (valsi_id, sound_data, mime_type)
                 VALUES ($1, $2, 'audio/ogg')",
                &[&valsi_id, &ogg],
            )
            .await;

        match insert {
            Ok(_) => {
                info!(
                    "valsi TTS: inserted sound for valsi_id {} ({})",
                    valsi_id, word
                );
                done += 1;
            }
            Err(e) => {
                error!(
                    "valsi TTS: failed to insert sound for valsi_id {} ({}): {}",
                    valsi_id, word, e
                );
            }
        }
    }

    Ok(done)
}
