//! Kokoro-82M German Martin ONNX (phoneme / IPA path) via Hugging Face download + local cache.
//!
//! Model: `Godelaune/Kokoro-82M-ONNX-German-Martin`; Martin NPZ and cstr GGUF voice styles.
//! Victoria uses this shared German backbone, not the Victoria fine-tuned checkpoint.
//! Inference follows `kokoro-onnx` (`is_phonemes=True`): IPA → Kokoro vocab tokens → style row by
//! token length → pad tokens → Ogg Opus.
#![allow(clippy::expect_used)] // fixed vocab / regex tables

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use ndarray::Array3;
use ndarray_npy::NpzReader;
use once_cell::sync::Lazy;
use ort::execution_providers::CPUExecutionProvider;
use ort::session::{Session, SessionInputValue};
use ort::value::Tensor;

use super::lojban_ipa::lojban_to_ipa;

const HF_BASE: &str = "https://huggingface.co/Godelaune/Kokoro-82M-ONNX-German-Martin/resolve/main";
const MODEL_FILENAME: &str = "kokoro-martin.onnx";
const VOICES_FILENAME: &str = "voices-martin.npz";
const VOICE_KEY: &str = "martin";
// Unquantized style vectors, pinned to a verified revision of cstr/kokoro-voices-GGUF.
const GGUF_BASE: &str = "https://huggingface.co/cstr/kokoro-voices-GGUF/resolve/1c98db113c69e3feef9644cf0aaec5c0bea02a8e";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KokoroVoice {
    Martin,
    Victoria,
    Eva,
    Bernd,
}

impl KokoroVoice {
    const GGUF_VOICES: [Self; 3] = [Self::Victoria, Self::Eva, Self::Bernd];

    pub fn parse(voice: &str) -> Result<Self, String> {
        match voice.trim().to_ascii_lowercase().as_str() {
            "martin" => Ok(Self::Martin),
            "victoria" => Ok(Self::Victoria),
            "eva" => Ok(Self::Eva),
            "bernd" => Ok(Self::Bernd),
            _ => Err(format!(
                "unknown voice embedding: {voice} (available: Martin, Victoria, Eva, Bernd)"
            )),
        }
    }

    fn gguf_filename(self) -> Option<&'static str> {
        match self {
            Self::Martin => None,
            Self::Victoria => Some("kokoro-voice-df_victoria.gguf"),
            Self::Eva => Some("kokoro-voice-df_eva.gguf"),
            Self::Bernd => Some("kokoro-voice-dm_bernd.gguf"),
        }
    }
}
/// Matches `kokoro_onnx.config.MAX_PHONEME_LENGTH`.
const MAX_PHONEME_LENGTH: usize = 510;
pub const SAMPLE_RATE_HZ: u32 = 24_000;

/// Kokoro DEFAULT_VOCAB: `CODEPOINT_HEX\\tID` per line (from `kokoro_onnx.config`).
static KOKORO_VOCAB: Lazy<HashMap<char, i64>> = Lazy::new(|| {
    const S: &str = include_str!("kokoro_tts_vocab.txt");
    let mut map = HashMap::new();
    for line in S.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (cp_hex, id_str) = line
            .split_once('\t')
            .expect("kokoro vocab line must be CODEPOINT_HEX\\tID");
        let cp = u32::from_str_radix(cp_hex, 16).expect("kokoro vocab codepoint hex");
        let id: i64 = id_str.parse().expect("kokoro vocab id");
        let ch = char::from_u32(cp).expect("kokoro vocab unicode scalar");
        map.insert(ch, id);
    }
    map
});

fn cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("KOKORO_TTS_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join("lensisku/kokoro-martin");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".cache/lensisku/kokoro-martin");
    }
    std::env::temp_dir().join("lensisku/kokoro-martin")
}

/// Cache the ONNX model and all voices at startup. Existing cache paths stay compatible.
pub fn ensure_model_files_cached() -> Result<(), String> {
    let mut errors = Vec::new();
    if let Err(error) = ensure_martin_cached() {
        errors.push(format!("Martin: {error}"));
    }
    for voice in KokoroVoice::GGUF_VOICES {
        if let Err(error) = ensure_gguf_cached(voice) {
            errors.push(format!("{voice:?}: {error}"));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn ensure_martin_cached() -> Result<(), String> {
    static READY: Mutex<bool> = Mutex::new(false);
    let mut ready = READY.lock().map_err(|e| e.to_string())?;
    if !*ready {
        let dir = cache_dir();
        download_file_blocking(
            &format!("{HF_BASE}/{MODEL_FILENAME}"),
            &dir.join(MODEL_FILENAME),
        )?;
        download_file_blocking(
            &format!("{HF_BASE}/{VOICES_FILENAME}"),
            &dir.join(VOICES_FILENAME),
        )?;
        *ready = true;
    }
    Ok(())
}

fn ensure_gguf_cached(voice: KokoroVoice) -> Result<(), String> {
    static READY: Mutex<Vec<KokoroVoice>> = Mutex::new(Vec::new());
    let mut ready = READY.lock().map_err(|e| e.to_string())?;
    if !ready.contains(&voice) {
        let filename = voice.gguf_filename().ok_or("voice does not use GGUF")?;
        download_file_blocking(
            &format!("{GGUF_BASE}/{filename}"),
            &cache_dir().join(filename),
        )?;
        ready.push(voice);
    }
    Ok(())
}

fn download_file_blocking(url: &str, dest: &std::path::Path) -> Result<(), String> {
    if dest.exists() {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = dest.with_extension("part");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())?;
    let bytes = client
        .get(url)
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .bytes()
        .map_err(|e| e.to_string())?;
    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, dest).map_err(|e| e.to_string())?;
    Ok(())
}

/// Split IPA into batches of at most [`MAX_PHONEME_LENGTH`] Unicode scalars (kokoro-onnx style).
fn split_phonemes(phonemes: &str) -> Vec<String> {
    let phonemes = phonemes.trim();
    if phonemes.is_empty() {
        return Vec::new();
    }
    let parts: Vec<&str> = {
        let mut out = Vec::new();
        let mut start = 0;
        for (i, ch) in phonemes.char_indices() {
            if matches!(ch, '.' | ',' | '!' | '?' | ';') {
                if start < i {
                    out.push(phonemes[start..i].trim());
                }
                out.push(&phonemes[i..i + ch.len_utf8()]);
                start = i + ch.len_utf8();
            }
        }
        if start < phonemes.len() {
            out.push(phonemes[start..].trim());
        }
        out.into_iter().filter(|s| !s.is_empty()).collect()
    };

    let mut batches = Vec::new();
    let mut current = String::new();
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let extra = if current.is_empty() || matches!(part, "." | "," | "!" | "?" | ";") {
            0
        } else {
            1
        };
        if current.chars().count() + extra + part.chars().count() >= MAX_PHONEME_LENGTH {
            if !current.is_empty() {
                batches.push(std::mem::take(&mut current));
            }
            current = part.to_string();
        } else if matches!(part, "." | "," | "!" | "?" | ";") {
            current.push_str(part);
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(part);
        }
    }
    if !current.is_empty() {
        batches.push(current);
    }
    batches
}

fn tokenize_phonemes(phonemes: &str) -> Vec<i64> {
    phonemes
        .chars()
        .filter_map(|ch| KOKORO_VOCAB.get(&ch).copied())
        .collect()
}

pub struct KokoroTts {
    session: Session,
    /// Voice style table: `[510, 1, 256]`.
    voice_styles: Array3<f32>,
    gguf_styles: HashMap<KokoroVoice, Array3<f32>>,
}

impl KokoroTts {
    /// Load the shared German ONNX model and Martin styles, downloading missing files.
    /// GGUF styles are loaded on demand so their availability cannot disable Martin.
    pub fn load_blocking() -> Result<Self, String> {
        ensure_martin_cached()?;

        let dir = cache_dir();
        let model_path = dir.join(MODEL_FILENAME);
        let voices_path = dir.join(VOICES_FILENAME);

        let session = Session::builder()
            .map_err(|e| e.to_string())?
            .with_execution_providers([CPUExecutionProvider::default().build()])
            .map_err(|e| e.to_string())?
            .commit_from_file(&model_path)
            .map_err(|e| e.to_string())?;

        let voices_file = std::fs::File::open(&voices_path).map_err(|e| e.to_string())?;
        let mut npz = NpzReader::new(voices_file).map_err(|e| e.to_string())?;
        let voice_styles: Array3<f32> = npz.by_name(VOICE_KEY).map_err(|e| e.to_string())?;
        if voice_styles.shape() != [510, 1, 256] {
            return Err(format!(
                "unexpected voice shape {:?}, expected [510, 1, 256]",
                voice_styles.shape()
            ));
        }

        Ok(KokoroTts {
            session,
            voice_styles,
            gguf_styles: HashMap::new(),
        })
    }

    /// Synthesize Lojban `word` to Ogg Opus bytes (RFC 7845), Martin voice, speed 0.8.
    pub fn lojban_word_to_ogg_opus(&mut self, word: &str) -> Result<Vec<u8>, String> {
        let ipa = lojban_to_ipa(word);
        self.ipa_to_ogg_opus(&ipa, "Martin", 0.8)
    }

    pub fn ipa_to_ogg_opus(
        &mut self,
        ipa: &str,
        voice_display: &str,
        speed: f32,
    ) -> Result<Vec<u8>, String> {
        let voice = KokoroVoice::parse(voice_display)?;
        if let Some(filename) = voice.gguf_filename() {
            if let std::collections::hash_map::Entry::Vacant(entry) = self.gguf_styles.entry(voice)
            {
                ensure_gguf_cached(voice)?;
                let bytes = std::fs::read(cache_dir().join(filename)).map_err(|e| e.to_string())?;
                entry.insert(super::kokoro_voice_gguf::read_voice_pack(&bytes)?);
            }
        }
        if !(0.5..=2.0).contains(&speed) {
            return Err("speed should be between 0.5 and 2.0".to_string());
        }

        let mut chunks: Vec<f32> = Vec::new();
        for batch in split_phonemes(ipa) {
            let audio = self.run_chunk(&batch, voice, speed)?;
            chunks.extend(audio);
        }
        if chunks.is_empty() {
            return Err("no audio produced (empty phonemes?)".to_string());
        }
        pcm_f32_to_ogg_opus(&chunks)
    }

    fn run_chunk(
        &mut self,
        phonemes: &str,
        voice: KokoroVoice,
        speed: f32,
    ) -> Result<Vec<f32>, String> {
        let mut phonemes = phonemes.to_string();
        if phonemes.chars().count() > MAX_PHONEME_LENGTH {
            phonemes = phonemes.chars().take(MAX_PHONEME_LENGTH).collect();
        }
        let tokens = tokenize_phonemes(&phonemes);
        if tokens.len() > MAX_PHONEME_LENGTH {
            return Err(format!(
                "phoneme token count {} exceeds max {MAX_PHONEME_LENGTH}",
                tokens.len()
            ));
        }
        if tokens.is_empty() {
            return Ok(Vec::new());
        }

        let styles = match voice {
            KokoroVoice::Martin => &self.voice_styles,
            _ => self
                .gguf_styles
                .get(&voice)
                .ok_or("GGUF voice styles not loaded")?,
        };
        let style_idx = tokens.len().min(styles.shape()[0].saturating_sub(1));
        let style_row = styles.slice(ndarray::s![style_idx, 0, ..]);
        let style_vec: Vec<f32> = style_row.iter().copied().collect();
        if style_vec.len() != 256 {
            return Err(format!("style dim {}, expected 256", style_vec.len()));
        }

        let mut padded = Vec::with_capacity(tokens.len() + 2);
        padded.push(0);
        padded.extend_from_slice(&tokens);
        padded.push(0);
        let n = padded.len();

        let mut inputs: HashMap<&str, SessionInputValue<'_>> = HashMap::new();
        inputs.insert(
            "tokens",
            SessionInputValue::from(
                Tensor::<i64>::from_array((vec![1i64, n as i64], padded))
                    .map_err(|e| e.to_string())?,
            ),
        );
        inputs.insert(
            "style",
            SessionInputValue::from(
                Tensor::<f32>::from_array((vec![1i64, 256], style_vec))
                    .map_err(|e| e.to_string())?,
            ),
        );
        inputs.insert(
            "speed",
            SessionInputValue::from(
                Tensor::<f32>::from_array((vec![1i64], vec![speed])).map_err(|e| e.to_string())?,
            ),
        );

        let outputs = self.session.run(inputs).map_err(|e| e.to_string())?;
        let view = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| e.to_string())?;
        Ok(view.1.to_vec())
    }
}

fn f32_to_i16_pcm(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| (f64::from(s).clamp(-1.0, 1.0) * 32767.0).round() as i16)
        .collect()
}

/// Mono float32 samples (model rate [`SAMPLE_RATE_HZ`]) → Ogg Opus (`audio/ogg`).
fn pcm_f32_to_ogg_opus(samples: &[f32]) -> Result<Vec<u8>, String> {
    let pcm = f32_to_i16_pcm(samples);
    if SAMPLE_RATE_HZ != 24000 {
        return Err("internal: SAMPLE_RATE_HZ must match ogg_opus::encode rate".to_string());
    }
    ogg_opus::encode::<24000, 1>(&pcm).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{split_phonemes, tokenize_phonemes, KokoroTts, KOKORO_VOCAB};

    #[test]
    fn recognizes_all_voices() {
        for (name, voice) in [
            ("martin", super::KokoroVoice::Martin),
            (" VICTORIA ", super::KokoroVoice::Victoria),
            ("Eva", super::KokoroVoice::Eva),
            ("BERND", super::KokoroVoice::Bernd),
        ] {
            assert_eq!(super::KokoroVoice::parse(name).unwrap(), voice);
        }
        assert!(super::KokoroVoice::parse("unknown").is_err());
    }

    #[test]
    #[ignore = "requires cached Martin ONNX/NPZ and Victoria/Eva/Bernd GGUF files"]
    fn synthesize_all_voices_from_cache() {
        let dir = super::cache_dir();
        for name in [super::MODEL_FILENAME, super::VOICES_FILENAME]
            .into_iter()
            .chain(
                super::KokoroVoice::GGUF_VOICES
                    .into_iter()
                    .filter_map(|voice| voice.gguf_filename()),
            )
        {
            assert!(dir.join(name).exists(), "missing cached {name}");
        }
        super::ensure_model_files_cached().expect("startup cache preparation");
        let mut engine = KokoroTts::load_blocking().expect("load German ONNX model");
        let mut outputs = Vec::new();
        for voice in ["Martin", "Victoria", "Eva", "Bernd"] {
            let audio = engine
                .ipa_to_ogg_opus("ʃɔɪ ɹoː doː.", voice, 0.8)
                .expect(voice);
            assert!(audio.starts_with(b"OggS"));
            let (pcm, _) =
                ogg_opus::decode::<_, 24000>(std::io::Cursor::new(audio)).expect("decode");
            assert!(
                pcm.iter().any(|sample| sample.unsigned_abs() > 100),
                "{voice} is silent"
            );
            for previous in &outputs {
                assert_ne!(&pcm, previous, "{voice} reused another voice's audio");
            }
            outputs.push(pcm);
        }
        assert_eq!(engine.gguf_styles.len(), 3);
    }

    #[test]
    fn vocab_has_core_ipa() {
        assert_eq!(KOKORO_VOCAB.get(&'ʃ'), Some(&131));
        assert_eq!(KOKORO_VOCAB.get(&'ɹ'), Some(&123));
        assert_eq!(KOKORO_VOCAB.get(&'ː'), Some(&158));
        assert_eq!(KOKORO_VOCAB.get(&'ˈ'), Some(&156));
        assert_eq!(KOKORO_VOCAB.get(&' '), Some(&16));
        // IPA eng/script-g only — Latin `g` is absent and would be dropped in tokenize.
        assert_eq!(KOKORO_VOCAB.get(&'ɡ'), Some(&92));
        assert!(KOKORO_VOCAB.get(&'g').is_none());
    }

    #[test]
    fn tokenize_coi_ro_do() {
        let tokens = tokenize_phonemes("ʃɔɪ ɹoː doː.");
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0], 131); // ʃ
    }

    #[test]
    fn split_keeps_short() {
        let batches = split_phonemes("ʃɔɪ ɹoː doː.");
        assert_eq!(batches.len(), 1);
    }

    #[test]
    fn smoke_synthesize_when_cached() {
        let dir = super::cache_dir();
        if !dir.join(super::MODEL_FILENAME).exists() {
            eprintln!("skip smoke: model not cached at {}", dir.display());
            return;
        }
        let mut eng = KokoroTts::load_blocking().expect("load kokoro");
        let ogg = eng
            .ipa_to_ogg_opus("ʃɔɪ ɹoː doː.", "Martin", 1.0)
            .expect("synthesize");
        assert!(ogg.len() > 100, "ogg too small: {}", ogg.len());
    }
}
