use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use deadpool_postgres::Pool;
use log::{error, info, warn};
use tokio::time::{sleep, Duration};

const BATCH_LIMIT: i64 = 20;

/// Placeholder mime for valsi that cannot be synthesized (empty/out-of-vocab IPA).
/// Keeps `valsi_sounds` claimed so the job does not retry forever; not served as audio.
const TTS_SKIP_MIME: &str = "application/x-lensisku-tts-skip";

pub fn spawn_valsi_sound_generation(pool: Pool) {
    let running = Arc::new(AtomicBool::new(false));
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(5 * 60)).await;

            if std::env::var("DISABLE_VALSI_TTS").ok().as_deref() == Some("1") {
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
                let result = async {
                    let mut total = 0usize;
                    loop {
                        let (inserted, had_rows) = run_valsi_sound_batch(&pool).await?;
                        total += inserted;
                        if !had_rows {
                            return Ok::<usize, String>(total);
                        }
                    }
                }
                .await;
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

/// One batch of at most [`BATCH_LIMIT`] valsi. Returns `(inserted_count, had_rows)` where `had_rows`
/// is false iff the SELECT returned no work — callers should stop draining only when `had_rows` is false.
async fn run_valsi_sound_batch(pool: &Pool) -> Result<(usize, bool), String> {
    let rows: Vec<(i32, String)> = {
        let client = pool.get().await.map_err(|e| format!("db pool: {e}"))?;
        let rows = client
            .query(
                "SELECT v.valsiid, v.word
                 FROM valsi v
                 LEFT JOIN valsi_sounds vs ON vs.valsi_id = v.valsiid
                 WHERE v.source_langid = 1
                   AND vs.valsi_id IS NULL
                   AND length(trim(both from v.word)) > 0
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
        rows.into_iter()
            .map(|row| (row.get("valsiid"), row.get("word")))
            .collect()
    };

    if rows.is_empty() {
        return Ok((0, false));
    }

    // One blocking task: load ONNX into RAM, synthesize every row, then drop the session so memory
    // is released until the next interval. `ensure_model_files_cached` inside `load_blocking` only
    // downloads HF artifacts once per process.
    // Per-word failures are skipped (not `?`) so one empty-IPA valsi cannot stall the whole queue.
    let outcomes: Vec<(i32, Vec<u8>, String, bool)> = tokio::task::spawn_blocking(move || {
        let mut engine = crate::utils::kokoro_tts::KokoroTts::load_blocking()?;
        let mut out = Vec::new();
        for (valsi_id, word) in rows {
            if word.split_whitespace().count() > 5 {
                continue;
            }
            match engine.lojban_word_to_ogg_opus(&word) {
                Ok(ogg) => out.push((valsi_id, ogg, word, true)),
                Err(e) => {
                    warn!(
                        "valsi TTS: skip valsi_id {} ({:?}): {}",
                        valsi_id, word, e
                    );
                    // Claim the row with an empty skip marker so ORDER BY valsiid cannot wedge.
                    out.push((valsi_id, Vec::new(), word, false));
                }
            }
        }
        Ok::<_, String>(out)
    })
    .await
    .map_err(|e| format!("join: {e}"))??;

    let mut done = 0usize;
    for (valsi_id, ogg, word, ok) in outcomes {
        let client = pool.get().await.map_err(|e| format!("db pool: {e}"))?;
        let mime = if ok {
            "audio/ogg".to_string()
        } else {
            TTS_SKIP_MIME.to_string()
        };
        let insert = client
            .execute(
                "INSERT INTO valsi_sounds (valsi_id, sound_data, mime_type)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (valsi_id) DO NOTHING",
                &[&valsi_id, &ogg, &mime],
            )
            .await;

        match insert {
            Ok(_) if ok => {
                info!(
                    "valsi TTS: inserted sound for valsi_id {} ({})",
                    valsi_id, word
                );
                done += 1;
            }
            Ok(_) => {
                info!(
                    "valsi TTS: marked unspeakable valsi_id {} ({:?})",
                    valsi_id, word
                );
            }
            Err(e) => {
                error!(
                    "valsi TTS: failed to insert sound for valsi_id {} ({}): {}",
                    valsi_id, word, e
                );
            }
        }
    }

    Ok((done, true))
}
