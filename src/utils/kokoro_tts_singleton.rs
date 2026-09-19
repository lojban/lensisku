//! Single shared [`crate::utils::kokoro_tts::KokoroTts`] instance (lazy init) for HTTP synthesis.

use std::sync::Mutex;

use crate::utils::kokoro_tts::KokoroTts;
use crate::utils::lojban_ipa::lojban_to_ipa;

static ENGINE: Mutex<Option<KokoroTts>> = Mutex::new(None);

/// Synthesize Lojban `text` to Ogg Opus bytes using the given voice and speed.
pub fn synthesize_lojban_to_ogg_opus(
    text: &str,
    voice: &str,
    speed: f32,
) -> Result<Vec<u8>, String> {
    let mut guard = ENGINE.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(KokoroTts::load_blocking()?);
    }
    let eng = guard.as_mut().ok_or("TTS engine not initialized")?;
    let ipa = lojban_to_ipa(text);
    eng.ipa_to_ogg_opus(&ipa, voice, speed)
}
