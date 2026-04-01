//! Single shared [`crate::utils::kitten_tts::KittenTts`] instance (lazy init) for HTTP synthesis.

use std::sync::{Mutex, OnceLock};

use crate::utils::kitten_tts::KittenTts;
use crate::utils::lojban_ipa::lojban_to_ipa;

static ENGINE: OnceLock<Result<Mutex<KittenTts>, String>> = OnceLock::new();

fn engine() -> Result<&'static Mutex<KittenTts>, String> {
    let init = ENGINE.get_or_init(|| KittenTts::load_blocking().map(Mutex::new));
    init.as_ref().map_err(|e| e.clone())
}

/// Synthesize Lojban `text` to Ogg Opus bytes using the given voice and speed.
pub fn synthesize_lojban_to_ogg_opus(text: &str, voice: &str, speed: f32) -> Result<Vec<u8>, String> {
    let mutex = engine()?;
    let mut eng = mutex.lock().map_err(|e| e.to_string())?;
    let ipa = lojban_to_ipa(text);
    eng.ipa_to_ogg_opus(&ipa, voice, speed)
}
