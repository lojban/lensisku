//! Kitten TTS Nano 0.8 ONNX inference (phoneme path), ported from `KittenTTS/kittentts/onnx_model.py`.
#![allow(clippy::expect_used)] // fixed tokenizer pattern

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use ndarray::Array2;
use ndarray_npy::NpzReader;
use once_cell::sync::Lazy;
use ort::execution_providers::CPUExecutionProvider;
use ort::session::{Session, SessionInputValue};
use ort::value::Tensor;
use regex::Regex;

use super::lojban_ipa::lojban_to_ipa;

const HF_BASE: &str =
    "https://huggingface.co/KittenML/kitten-tts-nano-0.8/resolve/main";
const MODEL_FILENAME: &str = "kitten_tts_nano_v0_8.onnx";
const VOICES_FILENAME: &str = "voices.npz";
const TRIM_TAIL_SAMPLES: usize = 5000;
pub const SAMPLE_RATE_HZ: u32 = 24_000;

/// Exact character order for token ids (from Kitten TTS `TextCleaner`).
static KITTEN_SYMBOL_TABLE: Lazy<HashMap<char, i64>> = Lazy::new(|| {
    const S: &str = include_str!("kitten_tts_symbols.txt");
    S.chars().enumerate().map(|(i, c)| (c, i as i64)).collect()
});

static TOKENIZE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?u)\w+|[^\w\s]").expect("tokenize regex")
});

fn basic_english_tokenize(text: &str) -> Vec<String> {
    TOKENIZE_RE
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// IPA string chunking (matches `chunk_phoneme_text` in Python; lengths are Unicode scalar counts).
fn chunk_phoneme_text(text: &str, max_len: usize) -> Vec<String> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }
    if text.chars().count() <= max_len {
        return vec![text.to_string()];
    }
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec![text.to_string()];
    }
    let mut chunks = Vec::new();
    let mut temp = String::new();
    for w in words {
        let sep = usize::from(!temp.is_empty());
        let w_len = w.chars().count();
        if temp.chars().count() + sep + w_len <= max_len {
            if !temp.is_empty() {
                temp.push(' ');
            }
            temp.push_str(w);
        } else {
            if !temp.is_empty() {
                chunks.push(std::mem::take(&mut temp));
            }
            if w_len > max_len {
                let chars: Vec<char> = w.chars().collect();
                for piece in chars.chunks(max_len) {
                    chunks.push(piece.iter().collect());
                }
            } else {
                temp.push_str(w);
            }
        }
    }
    if !temp.is_empty() {
        chunks.push(temp);
    }
    chunks
}

fn text_cleaner_indexes(phonemes_joined: &str) -> Vec<i64> {
    let mut indexes = Vec::new();
    for ch in phonemes_joined.chars() {
        if let Some(&idx) = KITTEN_SYMBOL_TABLE.get(&ch) {
            indexes.push(idx);
        }
    }
    indexes
}

fn phoneme_tokens(ipa: &str) -> Vec<i64> {
    let tokens_list = basic_english_tokenize(ipa);
    let joined = tokens_list.join(" ");
    let mut tokens = text_cleaner_indexes(&joined);
    tokens.insert(0, 0);
    tokens.push(10);
    tokens.push(0);
    tokens
}

fn cache_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join("lensisku/kitten-tts-nano-0.8");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".cache/lensisku/kitten-tts-nano-0.8");
    }
    std::env::temp_dir().join("lensisku/kitten-tts-nano-0.8")
}

/// Ensures ONNX + `voices.npz` exist under [`cache_dir`]. Safe to call every batch; the download
/// work runs **once** per process (Hugging Face files are written only if missing).
pub fn ensure_model_files_cached() -> Result<(), String> {
    static READY: OnceLock<Result<(), String>> = OnceLock::new();
    READY
        .get_or_init(|| {
            let dir = cache_dir();
            let model_path = dir.join(MODEL_FILENAME);
            let voices_path = dir.join(VOICES_FILENAME);
            download_file_blocking(
                &format!("{HF_BASE}/{MODEL_FILENAME}"),
                &model_path,
            )?;
            download_file_blocking(
                &format!("{HF_BASE}/{VOICES_FILENAME}"),
                &voices_path,
            )?;
            Ok(())
        })
        .clone()
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

pub struct KittenTts {
    session: Session,
    /// expr-voice-* → style rows (n × dim)
    voices: HashMap<String, Array2<f32>>,
    voice_aliases: HashMap<String, String>,
}

impl KittenTts {
    /// Load ONNX session and voice tensors from disk. Call [`ensure_model_files_cached`] first if
    /// files may be missing; otherwise this reads from cache only.
    pub fn load_blocking() -> Result<Self, String> {
        ensure_model_files_cached()?;

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
        let names = npz.names().map_err(|e| e.to_string())?;
        let mut voices = HashMap::new();
        for name in names {
            let arr: Array2<f32> = npz.by_name(&name).map_err(|e| e.to_string())?;
            voices.insert(name, arr);
        }

        let voice_aliases: HashMap<String, String> = [
            ("Bella", "expr-voice-2-f"),
            ("Jasper", "expr-voice-2-m"),
            ("Luna", "expr-voice-3-f"),
            ("Bruno", "expr-voice-3-m"),
            ("Rosie", "expr-voice-4-f"),
            ("Hugo", "expr-voice-4-m"),
            ("Kiki", "expr-voice-5-f"),
            ("Leo", "expr-voice-5-m"),
        ]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();

        Ok(KittenTts {
            session,
            voices,
            voice_aliases,
        })
    }

    fn resolve_voice(&self, voice: &str) -> Result<String, String> {
        let v = self
            .voice_aliases
            .get(voice)
            .cloned()
            .unwrap_or_else(|| voice.to_string());
        if !self.voices.contains_key(&v) {
            return Err(format!("unknown voice embedding: {v}"));
        }
        Ok(v)
    }

    /// Synthesize Lojban `word` to Ogg Opus bytes (RFC 7845), Bruno voice, speed 1.0.
    pub fn lojban_word_to_ogg_opus(&mut self, word: &str) -> Result<Vec<u8>, String> {
        let ipa = lojban_to_ipa(word);
        self.ipa_to_ogg_opus(&ipa, "Bruno", 1.0)
    }

    pub fn ipa_to_ogg_opus(
        &mut self,
        ipa: &str,
        voice_display: &str,
        speed: f32,
    ) -> Result<Vec<u8>, String> {
        let voice = self.resolve_voice(voice_display)?;
        let style_table = self
            .voices
            .get(&voice)
            .ok_or_else(|| format!("missing voice tensor {voice}"))?
            .clone();

        let mut chunks: Vec<f32> = Vec::new();
        for chunk_text in chunk_phoneme_text(ipa, 400) {
            let audio = self.run_chunk(&chunk_text, &style_table, speed)?;
            chunks.extend(audio);
        }

        pcm_f32_to_ogg_opus(&chunks)
    }

    fn run_chunk(
        &mut self,
        text: &str,
        style_table: &Array2<f32>,
        speed: f32,
    ) -> Result<Vec<f32>, String> {
        let tokens = phoneme_tokens(text);
        let n = tokens.len();
        let input_shape = vec![1i64, n as i64];

        let max_ref = style_table.shape()[0].saturating_sub(1);
        let ref_id = text.chars().count().min(max_ref);
        let style_row = style_table.row(ref_id);
        let style_vec: Vec<f32> = style_row.iter().copied().collect();
        let dim = style_vec.len();
        let style_shape = vec![1i64, dim as i64];

        let mut inputs: HashMap<&str, SessionInputValue<'_>> = HashMap::new();
        inputs.insert(
            "input_ids",
            SessionInputValue::from(
                Tensor::<i64>::from_array((input_shape, tokens)).map_err(|e| e.to_string())?,
            ),
        );
        inputs.insert(
            "style",
            SessionInputValue::from(
                Tensor::<f32>::from_array((style_shape, style_vec)).map_err(|e| e.to_string())?,
            ),
        );
        inputs.insert(
            "speed",
            SessionInputValue::from(
                Tensor::<f32>::from_array((vec![1i64], vec![speed]))
                    .map_err(|e| e.to_string())?,
            ),
        );

        let outputs = self
            .session
            .run(inputs)
            .map_err(|e| e.to_string())?;

        let view = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| e.to_string())?;
        let flat: Vec<f32> = view.1.to_vec();
        let len = flat.len();
        let trim = TRIM_TAIL_SAMPLES.min(len);
        let trimmed = if len > trim {
            flat[..len - trim].to_vec()
        } else {
            flat
        };

        Ok(trimmed)
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

