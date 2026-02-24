use std::{collections::HashMap, sync::Arc};

use crate::language::dto::*;
use crate::language::models::{Language, LojbanToken};
use camxes_rs::peg::{grammar::Peg, parsing::ParseResult};
use deadpool_postgres::{Pool, Transaction};
use log::warn;
use regex::Regex;
use vlazba::gismu_utils::GismuMatcher;
use vlazba::jvokaha::jvokaha;
use vlazba::jvozba::tools::RafsiOptions;
use vlazba::reconstruct_lujvo;

use super::models::{MathJaxValidationError, MathJaxValidationOptions};

pub async fn get_languages(pool: &Pool) -> Result<Vec<Language>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let languages = client
        .query(
            "SELECT langid, tag, englishname, lojbanname, realname, forlojban, url
             FROM languages 
             WHERE langid > 0 
             ORDER BY realname",
            &[],
        )
        .await?
        .into_iter()
        .map(Language::from)
        .collect();

    Ok(languages)
}

// TODO: Adapt this function or create a new one if parsing for non-Lojban languages is needed.
// Currently, it assumes the Lojban parser (ID 1).
pub fn parse_lojban(parsers: &Arc<HashMap<i32, Peg>>, input: &str) -> LojbanParseResponse {
    // Default to Lojban parser
    let parser = match parsers.get(&1) {
        Some(p) => p,
        None => {
            return LojbanParseResponse {
                success: false,
                tokens: vec![],
                error: Some("Lojban parser not available".to_string()),
            }
        }
    };

    let ParseResult(_, _, result) = parser.parse(input);

    match result {
        Ok(tokens) => {
            let mut lojban_tokens: Vec<LojbanToken> =
                tokens.into_iter().map(LojbanToken::from).collect();

            fn fill_text(token: &mut LojbanToken, input: &str) {
                token.text = input[token.start..token.end].to_string();
                for child in &mut token.children {
                    fill_text(child, input);
                }
            }

            for token in &mut lojban_tokens {
                fill_text(token, input);
            }

            LojbanParseResponse {
                success: true,
                tokens: lojban_tokens,
                error: None,
            }
        }
        Err(err) => LojbanParseResponse {
            success: false,
            tokens: vec![],
            error: Some(format!("Parsing failed: {:?}", err)),
        },
    }
}

fn fill_text(token: &mut LojbanToken, input: &str) {
    token.text = input[token.start..token.end].to_string();
    for child in &mut token.children {
        fill_text(child, input);
    }
}

pub async fn analyze_word(
    parsers: &Arc<HashMap<i32, Peg>>,
    word: &str,
    source_langid: i32, // Now required
    transaction: &Transaction<'_>,
) -> Result<AnalyzeWordResponse, Box<dyn std::error::Error>> {
    // Select the appropriate parser
    let parser = match parsers.get(&source_langid) {
        Some(p) => p,
        None => {
            // Fallback or error if the requested language parser isn't loaded
            warn!(
                "Parser for source_langid {} not found, falling back to Lojban (1)",
                source_langid
            );
            parsers.get(&1).ok_or_else(|| {
                format!(
                    "Parser not found for source_langid {} and Lojban parser is unavailable",
                    source_langid
                )
            })?
        }
    };

    let ParseResult(_, _, result) = parser.parse(word);

    match result {
        Ok(tokens) => {
            let mut parsed_tokens: Vec<LojbanToken> =
                tokens.into_iter().map(LojbanToken::from).collect();

            for token in &mut parsed_tokens {
                fill_text(token, word);
            }

            let texts = extract_token_text(&parsed_tokens);
            let word_type = analyze_word_type(&parsed_tokens);

            let reconstructed = match word_type.as_str() {
                "cmavo-compound" => {
                    let mut result = String::new();
                    for (i, text) in texts.iter().enumerate() {
                        if i > 0 {
                            let first_char = text.chars().next().unwrap_or(' ');
                            if "aeiouAEIOU".contains(first_char) {
                                result.push(' ');
                            }
                        }
                        result.push_str(text);
                    }
                    result
                }
                "phrase" => texts.join(" "),
                _ => texts.first().cloned().unwrap_or_default(),
            };

            let (recommended, problems) = if word_type.as_str() == "lujvo" {
                match jvokaha(word) {
                    Ok(_) => {
                        let cmavo = fetch_cmavo_rafsi(&transaction).await.ok();
                        let cmavo_exp = fetch_experimental_cmavo_rafsi(&transaction).await.ok();
                        let gismu = fetch_gismu_rafsi(&transaction).await.ok();
                        let gismu_exp = fetch_experimental_gismu_rafsi(&transaction).await.ok();

                        let reconstructed = reconstruct_lujvo(
                            word,
                            true,
                            &RafsiOptions {
                                exp_rafsi: true,
                                custom_cmavo: cmavo.as_ref(),
                                custom_cmavo_exp: cmavo_exp.as_ref(),
                                custom_gismu: gismu.as_ref(),
                                custom_gismu_exp: gismu_exp.as_ref(),
                            },
                        );

                        match reconstructed {
                            Ok(options) => (Some(options), None),
                            Err(_) => (None, None),
                        }
                    }
                    Err(_) => (None, None),
                }
            } else if ["gismu", "experimental gismu"].contains(&word_type.as_str()) {
                // 1 = gismu type ID
                let gismu_regular = fetch_gismu_data(&transaction, 1)
                    .await
                    .ok()
                    .unwrap_or_default();
                // 7 = experimental gismu type ID
                let gismu_exp = fetch_gismu_data(&transaction, 7)
                    .await
                    .ok()
                    .unwrap_or_default();
                let mut problems_map = HashMap::new();
                let matcher_regular = Arc::new(GismuMatcher::new(&gismu_regular, None));
                let matcher_exp = Arc::new(GismuMatcher::new(&gismu_exp, None));

                // Process regular gismu matches
                problems_map.insert("regular".to_string(), matcher_regular.gimka(word));

                // Process experimental gismu matches
                problems_map.insert("experimental".to_string(), matcher_exp.gimka(word));

                (None, Some(problems_map))
            } else {
                (None, None)
            };

            let response = AnalyzeWordResponse {
                success: true,
                word_type: word_type.clone(),
                text: reconstructed,
                recommended,
                problems,
                error: None,
            };

            Ok(response)
        }
        Err(e) => Ok(AnalyzeWordResponse {
            success: false,
            word_type: String::new(),
            text: String::new(),
            recommended: None,
            problems: None,
            error: Some(format!("Failed to parse word: {:?}", e)),
        }),
    }
}

pub async fn validate_mathjax_handler(text: &str) -> MathJaxValidationResponse {
    let options = MathJaxValidationOptions {
        use_tectonic: false, // Simple validation for language service
    };
    match validate_mathjax(text, &options).await {
        Ok(_) => MathJaxValidationResponse {
            valid: true,
            error: None,
        },
        Err(e) => MathJaxValidationResponse {
            valid: false,
            error: Some(e.to_string()),
        },
    }
}

pub fn extract_token_text(tokens: &[LojbanToken]) -> Vec<String> {
    let mut text_fields: Vec<String> = Vec::new();

    fn extract_text(token: &LojbanToken, text_fields: &mut Vec<String>) {
        match token.kind.as_str() {
            "non_terminal_lujvo_core"
            | "non_terminal_cmavo"
            | "non_terminal_cmevla"
            | "non_terminal_gismu"
            | "non_terminal_fuhivla" => {
                text_fields.push(token.text.clone());
            }
            _ => {
                for child in &token.children {
                    extract_text(child, text_fields);
                }
            }
        }
    }

    for token in tokens {
        extract_text(token, &mut text_fields);
    }

    text_fields
}

pub fn analyze_word_type(tokens: &[LojbanToken]) -> String {
    let mut any_words: Vec<String> = Vec::new();

    fn analyze_token(token: &LojbanToken, any_words: &mut Vec<String>) {
        match token.kind.as_str() {
            "non_terminal_lujvo_core"
            | "non_terminal_cmavo"
            | "non_terminal_cmevla"
            | "non_terminal_gismu"
            | "non_terminal_fuhivla" => {
                any_words.push(token.kind.clone());
            }
            _ => {
                for child in &token.children {
                    analyze_token(child, any_words);
                }
            }
        }
    }

    for token in tokens {
        analyze_token(token, &mut any_words);
    }

    // Early returns for single word cases
    if any_words.len() == 1 {
        match any_words[0].as_str() {
            "non_terminal_lujvo_core" => return "lujvo".to_string(),
            "non_terminal_cmavo" => return "cmavo".to_string(),
            "non_terminal_cmevla" => return "cmevla".to_string(),
            "non_terminal_gismu" => return "gismu".to_string(),
            "non_terminal_fuhivla" => return "fu'ivla".to_string(),
            _ => return "nalvla".to_string(),
        }
    }

    // Handle multiple words case
    if any_words.len() > 1 {
        // Check if all words are of valid types
        let all_valid = any_words.iter().all(|word| {
            matches!(
                word.as_str(),
                "non_terminal_lujvo_core"
                    | "non_terminal_cmavo"
                    | "non_terminal_cmevla"
                    | "non_terminal_gismu"
                    | "non_terminal_fuhivla"
            )
        });

        // Check if it might be a cmavo compound
        let all_cmavo = any_words.iter().all(|word| word == "non_terminal_cmavo");

        if all_valid {
            if all_cmavo {
                return "cmavo-compound".to_string();
            }
            return "phrase".to_string();
        }
    }

    "nalvla".to_string()
}

pub async fn validate_mathjax(
    text: &str,
    options: &MathJaxValidationOptions,
) -> Result<(), MathJaxValidationError> {
    if text.trim().is_empty() {
        return Ok(());
    }
    // Check for balanced delimiters
    check_balanced_delimiters(text)?;

    // Check common syntax patterns
    check_syntax_patterns(text)?;

    // validate with Tectonic
    if options.use_tectonic {
        validate_with_tectonic(text).await?;
    }

    Ok(())
}

/// Validate LaTeX/MathJax in multiple named fields. Returns `Ok(())` if all are valid,
/// or `Err((field_name, error))` for the first field that fails.
pub async fn validate_mathjax_fields(
    fields: &[(&str, &str)],
    options: &MathJaxValidationOptions,
) -> Result<(), (String, MathJaxValidationError)> {
    for (name, text) in fields {
        if let Err(e) = validate_mathjax(text, options).await {
            return Err(((*name).to_string(), e));
        }
    }
    Ok(())
}

fn check_balanced_delimiters(text: &str) -> Result<(), MathJaxValidationError> {
    let mut stack = Vec::new();
    let mut in_math = false;
    let mut i = 0;

    let chars: Vec<char> = text.chars().collect();
    while i < chars.len() {
        match chars[i] {
            '$' => {
                // Single dollar handling
                if in_math {
                    if stack.pop() != Some("$") {
                        return Err(MathJaxValidationError::Balance(
                            "Mismatched math delimiters".into(),
                        ));
                    }
                    in_math = false;
                } else {
                    stack.push("$");
                    in_math = true;
                }
                i += 1;
            }
            '\\' => {
                if i + 1 < chars.len() {
                    match chars[i + 1] {
                        '(' => {
                            if in_math {
                                return Err(MathJaxValidationError::Syntax(
                                    "Nested math environments not allowed".into(),
                                ));
                            }
                            stack.push("\\(");
                            in_math = true;
                            i += 2;
                        }
                        ')' => {
                            if !in_math || stack.pop() != Some("\\(") {
                                return Err(MathJaxValidationError::Balance(
                                    "Mismatched \\( \\) delimiters".into(),
                                ));
                            }
                            in_math = false;
                            i += 2;
                        }
                        '[' => {
                            if in_math {
                                return Err(MathJaxValidationError::Syntax(
                                    "Nested math environments not allowed".into(),
                                ));
                            }
                            stack.push("\\[");
                            in_math = true;
                            i += 2;
                        }
                        ']' => {
                            if !in_math || stack.pop() != Some("\\[") {
                                return Err(MathJaxValidationError::Balance(
                                    "Mismatched \\[ \\] delimiters".into(),
                                ));
                            }
                            in_math = false;
                            i += 2;
                        }
                        '{' => {
                            if in_math {
                                stack.push("{");
                            }
                            i += 2;
                        }
                        '}' => {
                            if in_math && stack.pop() != Some("{") {
                                return Err(MathJaxValidationError::Balance(
                                    "Mismatched {} delimiters".into(),
                                ));
                            }
                            i += 2;
                        }
                        _ => i += 2,
                    }
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    if !stack.is_empty() {
        return Err(MathJaxValidationError::Balance(format!(
            "Unclosed delimiters: {:?}",
            stack
        )));
    }

    Ok(())
}

fn check_syntax_patterns(text: &str) -> Result<(), MathJaxValidationError> {
    // Invalid or incomplete fraction
    if text.contains("\\frac") && !text.contains("\\frac{") {
        return Err(MathJaxValidationError::Syntax(
            "Invalid fraction syntax - missing arguments".into(),
        ));
    }

    // Check for missing arguments in common commands
    let commands_requiring_args = ["\\sqrt", "\\sum", "\\int", "\\lim", "\\sup", "\\inf"];
    for cmd in commands_requiring_args {
        if text.contains(cmd)
            && !text.contains(&format!("{}{{", cmd))
            && !text.contains(&format!("{}[", cmd))
        {
            return Err(MathJaxValidationError::Syntax(format!(
                "Missing arguments for {} command",
                cmd
            )));
        }
    }

    Ok(())
}

fn mathjax_to_latex(expr: &str) -> Result<String, MathJaxValidationError> {
    Regex::new(r"\$([^\$]+)\$")
        .map_err(|e| MathJaxValidationError::Syntax(format!("Invalid regex pattern: {}", e)))?;

    // Validate matching dollar signs
    let dollar_count = expr.chars().filter(|&c| c == '$').count();
    if dollar_count % 2 != 0 {
        return Err(MathJaxValidationError::Syntax(
            "Unmatched dollar signs in expression".to_string(),
        ));
    }

    // Simply return the input since LaTeX already understands $...$ syntax
    Ok(expr.to_string())
}

async fn fetch_rafsi_data(
    transaction: &Transaction<'_>,
    type_id: i16,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    let rows = transaction
        .query(
            "SELECT word, rafsi FROM valsi WHERE typeid = $1 AND rafsi IS NOT NULL",
            &[&type_id],
        )
        .await?;

    let mut result = HashMap::new();
    for row in rows {
        let word: String = row.get("word");
        let rafsi: String = row.get("rafsi");
        result.insert(
            word,
            rafsi.split_whitespace().map(|s| s.to_string()).collect(),
        );
    }
    Ok(result)
}

async fn fetch_gismu_data(
    transaction: &Transaction<'_>,
    type_id: i16,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let rows = transaction
        .query("SELECT word FROM valsi WHERE typeid = $1", &[&type_id])
        .await?;

    let mut result: Vec<String> = Vec::new();
    for row in rows {
        let word: String = row.get("word");
        result.push(word);
    }
    Ok(result)
}

async fn fetch_cmavo_rafsi(
    transaction: &Transaction<'_>,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    fetch_rafsi_data(transaction, 2).await // 2 = cmavo type ID
}

async fn fetch_experimental_cmavo_rafsi(
    transaction: &Transaction<'_>,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    fetch_rafsi_data(transaction, 8).await // 8 = experimental cmavo type ID
}

async fn fetch_gismu_rafsi(
    transaction: &Transaction<'_>,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    fetch_rafsi_data(transaction, 1).await // 1 = gismu type ID
}

async fn fetch_experimental_gismu_rafsi(
    transaction: &Transaction<'_>,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    fetch_rafsi_data(transaction, 7).await // 7 = experimental gismu type ID
}

/// Extract the last TeX error from a .log file for a user-friendly message.
/// Looks for "! <message>" and optional "l.<num> <context>" lines.
fn extract_latex_error_from_log(log: &str) -> Option<String> {
    let mut last_error = None;
    let mut last_line_ref = None;
    for line in log.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("! ") {
            last_error = Some(trimmed[2..].trim().to_string());
            last_line_ref = None;
        } else if let Some(rest) = trimmed.strip_prefix("l.") {
            let (num, ctx) = if let Some(num_end) = rest.find(' ') {
                (rest[..num_end].trim(), rest[num_end..].trim())
            } else {
                (rest.trim(), "")
            };
            let ctx_short = if ctx.len() > 60 {
                format!("{}...", &ctx[..57])
            } else {
                ctx.to_string()
            };
            last_line_ref = Some((num.to_string(), ctx_short));
        }
    }
    match (last_error, last_line_ref) {
        (Some(msg), Some((num, ctx))) if !ctx.is_empty() => {
            Some(format!("{} (line {}: …{}…)", msg, num, ctx))
        }
        (Some(msg), Some((num, _))) => Some(format!("{} (line {})", msg, num)),
        (Some(msg), None) => Some(msg),
        (None, _) => None,
    }
}

/// Run LaTeX via the tectonic driver with keep_logs(true) so we can read the
/// .log on failure and surface the actual TeX error (e.g. "Misplaced alignment tab character &").
fn compile_latex_and_capture_log(latex_document: String) -> Result<(), MathJaxValidationError> {
    use tectonic::driver::{OutputFormat, ProcessingSessionBuilder};
    use tectonic::status::NoopStatusBackend;

    let mut status = NoopStatusBackend::default();
    let config = tectonic::config::PersistentConfig::open(false)
        .map_err(|e| MathJaxValidationError::Tectonic(format!("Failed to open config: {}", e)))?;
    let bundle = config
        .default_bundle(false, &mut status)
        .map_err(|e| MathJaxValidationError::Tectonic(format!("Failed to load bundle: {}", e)))?;
    let format_cache_path = config.format_cache_path().map_err(|e| {
        MathJaxValidationError::Tectonic(format!("Failed to get format cache path: {}", e))
    })?;

    let mut sb = ProcessingSessionBuilder::default();
    sb.bundle(bundle)
        .primary_input_buffer(latex_document.as_bytes())
        .tex_input_name("texput.tex")
        .format_name("latex")
        .format_cache_path(format_cache_path)
        .keep_logs(true)
        .keep_intermediates(false)
        .print_stdout(false)
        .output_format(OutputFormat::Pdf)
        .do_not_write_output_files();

    let mut sess = sb.create(&mut status).map_err(|e| {
        MathJaxValidationError::Tectonic(format!("Failed to create session: {}", e))
    })?;

    if let Err(_e) = sess.run(&mut status) {
        let files = sess.into_file_data();
        let log_content = files
            .get("texput.log")
            .map(|f| String::from_utf8_lossy(&f.data).into_owned())
            .unwrap_or_default();
        let specific = extract_latex_error_from_log(&log_content)
            .unwrap_or_else(|| "see LaTeX log for details".to_string());
        return Err(MathJaxValidationError::Tectonic(specific));
    }

    Ok(())
}

async fn validate_with_tectonic(expr: &str) -> Result<(), MathJaxValidationError> {
    let latex_content = mathjax_to_latex(expr)?;

    let latex_document = format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath}}
\usepackage{{amssymb}}
\begin{{document}}
{}
\end{{document}}"#,
        latex_content
    );

    let result = tokio::task::spawn_blocking(move || compile_latex_and_capture_log(latex_document))
        .await
        .map_err(|e| MathJaxValidationError::Tectonic(format!("Thread join error: {}", e)))?;

    result
}

pub async fn analyze_word_in_pool(
    parsers: Arc<HashMap<i32, Peg>>,
    word: &str,
    source_langid: i32,
    pool: &Pool,
) -> Result<AnalyzeWordResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    // Dereference the Arc before passing the reference
    let response = analyze_word(&parsers, word, source_langid, &transaction).await?;
    transaction.commit().await?;
    Ok(response)
}
