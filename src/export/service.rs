use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::DateTime;
use chrono::Datelike;
use chrono::Utc;
use deadpool_postgres::Pool;
use deadpool_postgres::Transaction;
use log::info;
use log::{debug, error};
use std::error::Error;
use std::io::{Cursor, Write};
use tempfile::tempdir;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use xml::writer::{EventWriter, XmlEvent};
use zip::write::{SimpleFileOptions, ZipWriter};

use super::models::CachedExport;
use super::models::CollectionExportItem;
use super::models::DictionaryEntry;
use super::models::NaturalEntry;
use super::models::SearchExportJson;
use super::models::SearchExportQuery;
use super::models::User;
use super::models::ValsiRow;
use super::models::{ExportFormat, ExportOptions, SEARCH_EXPORT_ROW_CAP};
use crate::jbovlaste::KeywordMapping;
use std::collections::HashMap;

pub async fn generate_pdf(
    latex_content: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Create a temporary directory for working files
    let dir = tempdir()?;
    let dir_path = dir.path();

    // Create temporary file for LaTeX content
    let file_path = dir_path.join("output.tex");
    std::fs::write(&file_path, latex_content)?;

    debug!("Created temporary directory at: {:?}", dir_path);
    debug!("LaTeX file written to: {:?}", file_path);

    // Set HOME to temp dir to avoid permission issues
    let mut command = Command::new("xelatex");
    command
        .current_dir(dir_path)
        .env("HOME", dir_path)
        // .arg("-no-shell-escape") // Arbitrary command execution is prevented by xelatex by default. See https://github.com/tectonic-typesetting/tectonic/issues/38
        .arg("-interaction=nonstopmode")
        .arg("-halt-on-error")
        .arg(
            file_path
                .file_name()
                .ok_or_else(|| format!("Missing filename in temporary file path: {:?}", file_path))?
                .to_str()
                .ok_or_else(|| format!("Invalid UTF-8 in temporary file path: {:?}", file_path))?,
        );

    // Run xelatex and capture output with a hard timeout so a hung TeX
    // process cannot freeze the async runtime / startup.
    debug!("Executing command: {:?}", command);
    let output = match timeout(Duration::from_secs(300), command.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => {
            error!("xelatex failed to run: {}", e);
            return Err(Box::new(e));
        }
        Err(_) => {
            error!("xelatex timed out after 300s");
            return Err("xelatex timed out after 300s".into());
        }
    };

    // Log outputs
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    debug!("xelatex stdout:\n{}", stdout);
    if !stderr.is_empty() {
        error!("xelatex stderr:\n{}", stderr);
    }

    if !output.status.success() {
        // List directory contents for debugging
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            debug!("Directory contents after xelatex run:");
            for entry in entries.flatten() {
                debug!("  {:?}", entry.path());
            }
        }

        // Check if xelatex is installed
        let which_output = Command::new("which")
            .arg("xelatex")
            .output()
            .await
            .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
            .unwrap_or_else(|_| "not found".to_string());
        error!("xelatex path: {}", which_output);

        let error_msg = format!(
            "xelatex failed with status {}.\nCommand: {:?}\nWorking directory: {:?}\nStdout:\n{}\nStderr:\n{}",
            output.status,
            command,
            dir_path,
            stdout,
            stderr
        );
        error!("{}", error_msg);
        return Err(error_msg.into());
    }

    // Read the generated PDF
    let pdf_path = dir_path.join("output.pdf");
    debug!("Attempting to read PDF from: {:?}", pdf_path);

    match std::fs::read(&pdf_path) {
        Ok(content) => {
            debug!("Successfully read PDF of size {} bytes", content.len());
            Ok(content)
        }
        Err(e) => {
            let error_msg = format!("Failed to read generated PDF: {}", e);
            error!("{}", error_msg);
            error!("PDF path: {:?}", pdf_path);
            Err(Box::new(e))
        }
    }
}

// Constants
const JAPANESE: &str = "ja";
const GUASPI: &str = "art-guaspi";

fn escape_all(term: &str) -> String {
    let mut result = term.to_string();
    result = result.replace('\\', "\\textbackslash{}");
    result = result.replace('{', "\\{");
    result = result.replace('}', "\\}");
    result = result.replace('~', "\\textasciitilde{}");
    result = result.replace('^', "\\textasciicircum{}");
    result = result.replace('/', "\\slash{}");

    for c in ['#', '%', '&', '$', '_'] {
        result = result.replace(c, &format!("\\{}", c));
    }

    result
}

fn escape_tex(term: &str, escape_carets: bool) -> String {
    let mut result = term.to_string();
    result = result.replace('\\', "\\textbackslash{}");
    result = result.replace('>', "\\textgreater{}");
    result = result.replace('<', "\\textless{}");
    result = result.replace('–', "\\textendash{}");
    result = result.replace('—', "\\textemdash{}");
    result = result.replace('~', "\\textasciitilde{}");

    if escape_carets {
        result = result.replace('^', "\\textasciicircum{}");
    }

    result = result.replace('/', "\\slash{}");

    for c in ['#', '%', '&'] {
        result = result.replace(c, &format!("\\{}", c));
    }

    result
}

fn generate_title(escaped_lang: &str, collection_id: Option<i32>) -> String {
    if collection_id.is_some() {
        "lo vlaste".to_string()
    } else {
        let vlaste_languages = if escaped_lang == "lojban" {
            "la .lojban.".to_string()
        } else {
            format!("la .lojban. jo'u la'o zoi {} zoi", escaped_lang)
        };
        format!("lo vlaste be fu {}", vlaste_languages)
    }
}

fn format_lojban_heading(word: &str, valsi_type: &str) -> String {
    let escaped_word = escape_all(word);
    let heading = if valsi_type.starts_with("experimental") || valsi_type.starts_with("obsolete") {
        format_lojban_experimental_heading(&escaped_word)
    } else {
        format_normal_heading(&escaped_word)
    };
    format!("{}{}", heading, markboth(&escaped_word))
}

fn format_normal_heading(escaped_word: &str) -> String {
    format!("\n\n{{\\sffamily\\bfseries {}}}", escaped_word)
}

fn format_lojban_experimental_heading(escaped_word: &str) -> String {
    format!("\n\n{{\\sffamily\\bfseries $\\triangle$ {}}}", escaped_word)
}

fn markboth(escaped_word: &str) -> String {
    format!("\\markboth{{{}}}{{{}}}", escaped_word, escaped_word)
}

fn format_rafsi(rafsi: &Option<String>) -> String {
    match rafsi {
        Some(r) => {
            let trimmed = r.trim();
            if trimmed.is_empty() {
                String::new()
            } else {
                format!(
                    "\\enspace {{\\ttfamily\\bfseries[{}]}} ",
                    escape_all(trimmed)
                )
            }
        }
        None => String::new(),
    }
}

fn format_selmaho(selmaho: &Option<String>) -> String {
    match selmaho {
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                String::new()
            } else {
                format!(
                    "\\enspace {{\\sffamily\\bfseries[{}]}} ",
                    escape_all(trimmed)
                )
            }
        }
        None => String::new(),
    }
}

fn format_definition(definition: &str, lang: &str) -> String {
    let carets_are_literal = lang == GUASPI;
    format!(" {}", escape_tex(definition, carets_are_literal))
}

fn format_notes(notes: &Option<String>) -> String {
    match notes {
        Some(n) if !n.is_empty() => {
            if sniff_tex(n) {
                format!(" \\textemdash{{}} {}", escape_tex(n, false))
            } else {
                format!(" \\textemdash{{}} {}", escape_all(n))
            }
        }
        _ => String::new(),
    }
}

fn sniff_tex(text: &str) -> bool {
    text.contains('$')
}

fn format_natural_heading(word: &str) -> String {
    let escaped_word = escape_all(word);
    let heading = format_normal_heading(&escaped_word);
    format!("{}{}", heading, markboth(&escaped_word))
}

fn format_meaning(meaning: &Option<String>) -> String {
    meaning
        .as_ref()
        .map(|m| format!("\\textit{{({})}} ", escape_all(m)))
        .unwrap_or_default()
}

fn format_valsi(valsi: &str) -> String {
    format!(" {}", escape_all(valsi))
}

fn format_place(place: i32) -> String {
    if place > 0 {
        format!("$_{{{}}}$", place)
    } else {
        String::new()
    }
}
fn latex_header(title: &str, lang: &str) -> String {
    let now = chrono::Local::now();
    let jbo_date = format!(
        "de'i li {} pi'e {} pi'e {}",
        now.year(),
        now.month(),
        now.day()
    );

    format!("{}\n\\title{{{}}}\n\\author{{lo jboce'u}}\n\\date{{{}}}\n\n\\begin{{document}}\n\n\\maketitle",
        latex_preamble(lang),
        title,
        jbo_date
    )
}

fn latex_preamble(lang: &str) -> String {
    format!(
        "{}{}{}",
        latex_preamble_intro(),
        latex_preamble_fonts(lang),
        latex_preamble_outro()
    )
}

fn latex_preamble_intro() -> String {
    r#"%!TEX encoding = UTF-8 Unicode
%!TEX TS-program = xelatex
\documentclass[notitlepage,twocolumn,a4paper,10pt]{book}
\renewcommand\chaptername{ni'o ni'o}

\usepackage{underscore}

\usepackage{fancyhdr} % important, lets us actually pull this stuff off.
\pagestyle{fancy}     % turns on the magic provided by fancyhdr

% Packages from http://linuxlibertine.sourceforge.net/Libertine-XeTex-EN.pdf
\usepackage{xunicode} % for XeTeX!
\usepackage{fontspec} % for XeTeX!
\usepackage{xltxtra} % for XeTeX!

% Font definitions mostly from http://linuxlibertine.sourceforge.net/Libertine-XeTex-EN.pdf
\defaultfontfeatures{Scale=MatchLowercase}% to adjust all used fonts to the same x-height"#
        .to_string()
}

fn latex_preamble_fonts(lang: &str) -> String {
    format!(
        "{}{}",
        latex_preamble_roman_fonts(),
        latex_preamble_cjk_fonts(lang)
    )
}

fn latex_preamble_roman_fonts() -> String {
    r#"
\setromanfont[Mapping=tex-text]{Linux Libertine O}
\setsansfont[Mapping=tex-text]{Linux Biolinum O}"#
        .to_string()
}

fn latex_preamble_cjk_fonts(lang: &str) -> String {
    match lang {
        JAPANESE => r#"
\usepackage{xeCJK}
\setCJKmainfont{Noto Serif CJK JP}
\setCJKsansfont{Noto Sans CJK JP}
\setCJKmonofont{Noto Sans Mono CJK JP}"#
            .to_string(),
        "hi" => r#"
\usepackage{ucharclasses}
\newfontfamily\devanagarifont{Noto Serif Devanagari}
\setTransitionsFor{Devanagari}{\devanagarifont}{\rmfamily}"#
            .to_string(),
        _ => r#"
\usepackage{xeCJK}
\setCJKmainfont[Mapping=tex-text]{Noto Sans CJK SC}"#
            .to_string(),
    }
}

fn latex_preamble_outro() -> String {
    r#"
\fancyhead{}          % empty out the header
\fancyfoot{}          % empty out the footer
\fancyhead[LE,LO]{\rightmark} % left side, odd and even pages
\fancyhead[RE,RO]{\leftmark}  % right side, odd and even pages
\fancyfoot[LE,RO]{\thepage}   % left side even, right side odd

\setlength{\parindent}{1 em}"#
        .to_string()
}

fn latex_footer() -> String {
    "\n\\end{document}".to_string()
}

pub async fn verify_language_exists(
    transaction: &mut Transaction<'_>,
    lang: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let row = transaction
        .query_one("SELECT COUNT(*) FROM languages WHERE tag = $1", &[&lang])
        .await?;

    Ok(row.get::<_, i64>(0) > 0)
}

pub async fn verify_collection_access(
    transaction: &mut Transaction<'_>,
    collection_id: i32,
    user_id: Option<i32>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let row = transaction
        .query_one(
            "SELECT user_id, is_public FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await?;

    let is_public: bool = row.get("is_public");
    let owner_id: i32 = row.get("user_id");

    Ok(is_public || user_id == Some(owner_id))
}

const DEFAULT_SOURCE_LANGID: i32 = 1;
const DEFAULT_SOURCE_LANGUAGE_TAG: &str = "jbo";

/// Resolve a language tag to its `(langid, tag)` pair.
/// Defaults to Lojban (`langid = 1`, tag `jbo`) when tag is None or blank.
pub async fn resolve_source_language(
    transaction: &mut Transaction<'_>,
    source_lang: Option<&str>,
) -> Result<(i32, String), Box<dyn std::error::Error + Send + Sync>> {
    match source_lang {
        Some(tag) if !tag.is_empty() => {
            let row = transaction
                .query_opt("SELECT langid, tag FROM languages WHERE tag = $1", &[&tag])
                .await?;
            match row {
                Some(r) => Ok((r.get(0), r.get(1))),
                None => Err(format!("Unknown source language tag: {}", tag).into()),
            }
        }
        _ => Ok((
            DEFAULT_SOURCE_LANGID,
            DEFAULT_SOURCE_LANGUAGE_TAG.to_string(),
        )),
    }
}

pub async fn export_with_access_check(
    pool: &Pool,
    lang: &str,
    format: ExportFormat,
    options: &ExportOptions,
    user_id: Option<i32>,
) -> Result<(Vec<u8>, String, String), Box<dyn std::error::Error + Send + Sync>> {
    let mut client = pool.get().await?;
    let mut transaction = client.transaction().await?;

    if !verify_language_exists(&mut transaction, lang).await? {
        return Err("Invalid language tag".into());
    }
    if let Some(collection_id) = options.collection_id {
        if !verify_collection_access(&mut transaction, collection_id, user_id).await? {
            return Err("Access denied".into());
        }
    }
    let (source_langid, source_language_tag) =
        resolve_source_language(&mut transaction, options.source_lang.as_deref()).await?;

    transaction.commit().await?;
    export_dictionary(
        pool,
        lang,
        format,
        options,
        options.collection_id,
        source_langid,
        &source_language_tag,
        true,
    )
    .await
}

/// `use_dictionary_cache`: when true, serve a recent row from `cached_dictionary_exports` (up to 4 days)
/// instead of regenerating. Background refresh must pass `false` so exports actually rebuild after the
/// daily skip window, instead of re-reading stale cache rows.
#[allow(clippy::too_many_arguments)]
pub async fn export_dictionary(
    pool: &Pool,
    lang: &str,
    format: ExportFormat,
    options: &ExportOptions,
    collection_id: Option<i32>,
    source_langid: i32,
    source_language_tag: &str,
    use_dictionary_cache: bool,
) -> Result<(Vec<u8>, String, String), Box<dyn std::error::Error + Send + Sync>> {
    let positive_scores_only = options.positive_scores_only.unwrap_or(true);

    // For collection exports, bypass cache
    if collection_id.is_some() {
        return generate_export(
            pool,
            lang,
            format,
            options,
            collection_id,
            source_langid,
            source_language_tag,
        )
        .await;
    }

    if use_dictionary_cache {
        let mut client = pool.get().await?;
        let transaction = client.transaction().await?;

        // Try to get from cache first using transaction. The cache key must include
        // every option that affects output: target language, source language, format,
        // and the positive_scores_only flag.
        if let Some(row) = transaction
            .query_opt(
                "SELECT content, content_type, filename
                 FROM cached_dictionary_exports
                 WHERE language_tag = $1
                   AND source_language_tag = $2
                   AND format = $3
                   AND positive_scores_only = $4
                   AND created_at > NOW() - INTERVAL '4 days'",
                &[
                    &lang,
                    &source_language_tag,
                    &format.to_string(),
                    &positive_scores_only,
                ],
            )
            .await?
        {
            // Commit transaction since we successfully found a cached result
            transaction.commit().await?;
            return Ok((
                row.get("content"),
                row.get("content_type"),
                row.get("filename"),
            ));
        }

        // Commit transaction since we'll generate a new export
        transaction.commit().await?;
    }

    // If not in cache, or batch refresh bypassed cache read, generate
    let result = generate_export(
        pool,
        lang,
        format,
        options,
        collection_id,
        source_langid,
        source_language_tag,
    )
    .await?;

    // Cache the result of on-demand dictionary exports so the next identical
    // request does not pay the xelatex generation cost again.
    if use_dictionary_cache && collection_id.is_none() {
        let format_str = format.to_string();
        match pool.get().await {
            Ok(c) => {
                if let Err(e) = c
                    .execute(
                        "INSERT INTO cached_dictionary_exports
                         (language_tag, source_language_tag, format, positive_scores_only, content, content_type, filename)
                         VALUES ($1, $2, $3, $4, $5, $6, $7)
                         ON CONFLICT (language_tag, source_language_tag, format, positive_scores_only)
                         DO UPDATE SET
                            content = EXCLUDED.content,
                            content_type = EXCLUDED.content_type,
                            filename = EXCLUDED.filename,
                            created_at = CURRENT_TIMESTAMP",
                        &[
                            &lang,
                            &source_language_tag,
                            &format_str,
                            &positive_scores_only,
                            &result.0.as_slice(),
                            &result.1,
                            &result.2,
                        ],
                    )
                    .await
                {
                    error!("Failed to cache on-demand {} export for {}: {}", format, lang, e);
                }
            }
            Err(e) => {
                error!("Failed to get DB pool to cache on-demand {} export for {}: {}", format, lang, e);
            }
        }
    }

    Ok(result)
}

fn zip_tsv_content(
    tsv_content: &str,
    filename: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    zip_tsv_files(&[(filename, tsv_content)])
}

fn zip_tsv_files(
    files: &[(&str, &str)],
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut zip_buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut zip_buffer));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for (filename, tsv_content) in files {
            if tsv_content.is_empty() {
                continue;
            }
            zip.start_file(*filename, options)?;
            zip.write_all(tsv_content.as_bytes())?;
        }
        zip.finish()?;
    }
    Ok(zip_buffer)
}

fn build_export_filename(
    collection_id: Option<i32>,
    source_language_tag: &str,
    lang: &str,
    extension: &str,
) -> String {
    match collection_id {
        Some(id) => format!("collection-{}-{}.{}", id, lang, extension),
        None => format!("dictionary-{}-{}.{}", source_language_tag, lang, extension),
    }
}

async fn generate_export(
    pool: &Pool,
    lang: &str,
    format: ExportFormat,
    options: &ExportOptions,
    collection_id: Option<i32>,
    source_langid: i32,
    source_language_tag: &str,
) -> Result<(Vec<u8>, String, String), Box<dyn std::error::Error + Send + Sync>> {
    let filename = build_export_filename(
        collection_id,
        source_language_tag,
        lang,
        format.file_extension(),
    );
    let content_type = format.content_type().to_string();

    let mut client = pool.get().await?;
    let mut transaction = client.transaction().await?;

    let content = match format {
        ExportFormat::Pdf => {
            let latex = generate_latex(
                &mut transaction,
                lang,
                options,
                collection_id,
                source_langid,
            )
            .await?;
            transaction.commit().await?;
            generate_pdf(&latex).await?
        }
        ExportFormat::LaTeX => {
            let latex = generate_latex(
                &mut transaction,
                lang,
                options,
                collection_id,
                source_langid,
            )
            .await?;
            transaction.commit().await?;
            latex.into_bytes()
        }
        ExportFormat::Xml => {
            let xml = generate_xml(
                &mut transaction,
                lang,
                options,
                collection_id,
                source_langid,
            )
            .await?;
            transaction.commit().await?;
            xml.into_bytes()
        }
        ExportFormat::Json => {
            let json = generate_json(
                &mut transaction,
                lang,
                options,
                collection_id,
                source_langid,
            )
            .await?;
            transaction.commit().await?;
            json.into_bytes()
        }
        ExportFormat::Tsv => {
            let tsv = generate_tsv(
                &mut transaction,
                lang,
                options,
                collection_id,
                source_langid,
            )
            .await?;
            transaction.commit().await?;
            // Determine the TSV filename (without .zip extension)
            let tsv_filename =
                build_export_filename(collection_id, source_language_tag, lang, "tsv");
            zip_tsv_content(&tsv, &tsv_filename)?
        }
    };

    Ok((content, content_type, filename))
}

async fn fetch_keywords_for_export(
    transaction: &mut Transaction<'_>,
    def_ids: &[i32],
) -> Result<
    (
        HashMap<i32, Vec<KeywordMapping>>, // Gloss keywords
        HashMap<i32, Vec<KeywordMapping>>, // Place keywords
    ),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let mut gloss_map: HashMap<i32, Vec<KeywordMapping>> = HashMap::new();
    let mut place_map: HashMap<i32, Vec<KeywordMapping>> = HashMap::new();

    if def_ids.is_empty() {
        return Ok((gloss_map, place_map));
    }

    // Fetch gloss keywords (place = 0)
    let gloss_rows = transaction
        .query(
            "SELECT k.definitionid, n.word, n.meaning
             FROM keywordmapping k
             JOIN natlangwords n ON k.natlangwordid = n.wordid
             WHERE k.definitionid = ANY($1) AND k.place = 0",
            &[&def_ids],
        )
        .await?;

    for row in gloss_rows {
        let def_id: i32 = row.get("definitionid");
        let mapping = KeywordMapping {
            word: row.get("word"),
            meaning: row.get("meaning"),
        };
        gloss_map.entry(def_id).or_default().push(mapping);
    }

    // Fetch place keywords (place > 0)
    let place_rows = transaction
        .query(
            "SELECT k.definitionid, n.word, n.meaning
             FROM keywordmapping k
             JOIN natlangwords n ON k.natlangwordid = n.wordid
             WHERE k.definitionid = ANY($1) AND k.place > 0
             ORDER BY k.definitionid, k.place",
            &[&def_ids],
        )
        .await?;

    for row in place_rows {
        let def_id: i32 = row.get("definitionid");
        let mapping = KeywordMapping {
            word: row.get("word"),
            meaning: row.get("meaning"),
        };
        place_map.entry(def_id).or_default().push(mapping);
    }

    Ok((gloss_map, place_map))
}

async fn generate_xml(
    transaction: &mut Transaction<'_>,
    lang: &str,
    options: &ExportOptions,
    collection_id: Option<i32>,
    source_langid: i32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(id) = collection_id {
        // Handle collection export
        let query = "
            SELECT
                ci.item_id, ci.definition_id, ci.notes as collection_note, ci.position,
                ci.free_content_front, ci.free_content_back, 
                ci.langid as language_id, ci.owner_user_id, ci.license,
                v.word, d.definition, d.notes as definition_notes, d.jargon, t.descriptor as word_type,
                c.rafsi, c.selmaho,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_mime,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_mime
            FROM collection_items ci
            LEFT JOIN definitions d ON ci.definition_id = d.definitionid
            LEFT JOIN valsi v ON d.valsiid = v.valsiid
            LEFT JOIN valsitypes t ON v.typeid = t.typeid
            LEFT JOIN convenientdefinitions c ON c.definitionid = d.definitionid
            WHERE ci.collection_id = $1
            ORDER BY ci.position";

        let rows = transaction.query(query, &[&id]).await?;

        let entries: Vec<CollectionExportItem> = rows
            .into_iter()
            .map(|row| {
                let front_image_url =
                    row.get::<_, Option<Vec<u8>>>("front_image_data")
                        .and_then(|data| {
                            row.get::<_, Option<String>>("front_image_mime")
                                .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(data)))
                        });
                let back_image_url =
                    row.get::<_, Option<Vec<u8>>>("back_image_data")
                        .and_then(|data| {
                            row.get::<_, Option<String>>("back_image_mime")
                                .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(data)))
                        });
                CollectionExportItem::from_row(row, front_image_url, back_image_url)
            })
            .collect();
        return Ok(serde_json::to_string_pretty(&entries)?);
    }

    if let Some(id) = collection_id {
        // Handle collection export
        let query = "
            SELECT
                ci.item_id, ci.definition_id, ci.notes as collection_note, ci.position,
                ci.free_content_front, ci.free_content_back, 
                ci.langid as language_id, ci.owner_user_id, ci.license,
                v.word, d.definition, d.notes as definition_notes, d.jargon, t.descriptor as word_type,
                c.rafsi, c.selmaho,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_mime,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_mime
            FROM collection_items ci
            LEFT JOIN definitions d ON ci.definition_id = d.definitionid
            LEFT JOIN valsi v ON d.valsiid = v.valsiid
            LEFT JOIN valsitypes t ON v.typeid = t.typeid
            LEFT JOIN convenientdefinitions c ON c.definitionid = d.definitionid -- For rafsi/selmaho if needed
            WHERE ci.collection_id = $1
            ORDER BY ci.position";

        let rows = transaction.query(query, &[&id]).await?;

        let entries: Vec<CollectionExportItem> = rows
            .into_iter()
            .map(|row| {
                let front_image_url =
                    row.get::<_, Option<Vec<u8>>>("front_image_data")
                        .and_then(|data| {
                            row.get::<_, Option<String>>("front_image_mime")
                                .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(data)))
                        });
                let back_image_url =
                    row.get::<_, Option<Vec<u8>>>("back_image_data")
                        .and_then(|data| {
                            row.get::<_, Option<String>>("back_image_mime")
                                .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(data)))
                        });

                CollectionExportItem::from_row(row, front_image_url, back_image_url)
            })
            .collect();
        return Ok(serde_json::to_string_pretty(&entries)?);
    }

    let mut writer = EventWriter::new(Cursor::new(Vec::new()));

    writer.write(XmlEvent::StartDocument {
        version: xml::common::XmlVersion::Version10,
        encoding: Some("UTF-8"),
        standalone: None,
    })?;

    writer.write(XmlEvent::start_element("dictionary"))?;

    let lang_info = transaction
        .query_one(
            "SELECT langid, tag, realname FROM languages WHERE tag = $1",
            &[&lang],
        )
        .await?;

    writer.write(XmlEvent::start_element("metadata"))?;
    writer.write(XmlEvent::start_element("language"))?;
    writer.write(XmlEvent::Characters(
        &lang_info.get::<_, String>("realname"),
    ))?;
    writer.write(XmlEvent::end_element())?;
    writer.write(XmlEvent::end_element())?;

    let positive_scores_only = options.positive_scores_only.unwrap_or(true);

    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = bd.definitionid")
        .unwrap_or("");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let query = format!(
        "SELECT v.word, bd.definitionid, c.rafsi, c.selmaho, c.definition,
                c.notes, d.jargon, t.descriptor, bd.score
         FROM export_best_definitions($1, $3) bd
         JOIN valsi v ON v.valsiid = bd.valsiid
         JOIN convenientdefinitions c ON c.definitionid = bd.definitionid
         JOIN definitions d ON d.definitionid = bd.definitionid
         JOIN valsitypes t ON t.typeid = v.typeid
         {}
         WHERE v.source_langid = $2 {}
         ORDER BY lower(v.word)",
        collection_join, collection_condition
    );

    let langid = lang_info.get::<_, i32>("langid");
    let rows = transaction
        .query(&query, &[&langid, &source_langid, &positive_scores_only])
        .await?;

    // Collect all definition IDs
    let def_ids: Vec<i32> = rows
        .iter()
        .map(|row| row.get::<_, i32>("definitionid"))
        .collect();

    // Fetch gloss keywords and place keywords for all definitions
    let (gloss_map, place_map) = fetch_keywords_for_export(transaction, &def_ids).await?;

    writer.write(XmlEvent::start_element("entries"))?;
    for row in rows.iter() {
        let definition_id: i32 = row.get("definitionid");
        let gloss_keywords = gloss_map.get(&definition_id);
        let place_keywords = place_map.get(&definition_id);

        writer.write(XmlEvent::start_element("entry"))?;

        writer.write(XmlEvent::start_element("word"))?;
        writer.write(XmlEvent::Characters(&row.get::<_, String>("word")))?;
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::start_element("type"))?;
        writer.write(XmlEvent::Characters(&row.get::<_, String>("descriptor")))?;
        writer.write(XmlEvent::end_element())?;

        if let Some(rafsi) = row.get::<_, Option<String>>("rafsi") {
            writer.write(XmlEvent::start_element("rafsi"))?;
            writer.write(XmlEvent::Characters(&rafsi))?;
            writer.write(XmlEvent::end_element())?;
        }

        if let Some(selmaho) = row.get::<_, Option<String>>("selmaho") {
            writer.write(XmlEvent::start_element("selmaho"))?;
            writer.write(XmlEvent::Characters(&selmaho))?;
            writer.write(XmlEvent::end_element())?;
        }

        writer.write(XmlEvent::start_element("definition"))?;
        writer.write(XmlEvent::Characters(&row.get::<_, String>("definition")))?;
        writer.write(XmlEvent::end_element())?;

        if let Some(notes) = row.get::<_, Option<String>>("notes") {
            writer.write(XmlEvent::start_element("notes"))?;
            writer.write(XmlEvent::Characters(&notes))?;
            writer.write(XmlEvent::end_element())?;
        }

        if let Some(jargon) = row.get::<_, Option<String>>("jargon") {
            if !jargon.is_empty() {
                writer.write(XmlEvent::start_element("jargon"))?;
                writer.write(XmlEvent::Characters(&jargon))?;
                writer.write(XmlEvent::end_element())?;
            }
        }

        writer.write(XmlEvent::start_element("score"))?;
        writer.write(XmlEvent::Characters(
            &row.get::<_, i64>("score").to_string(),
        ))?;
        writer.write(XmlEvent::end_element())?;

        // Add gloss keywords
        if let Some(gloss_keywords) = gloss_keywords {
            if !gloss_keywords.is_empty() {
                writer.write(XmlEvent::start_element("gloss_keywords"))?;
                for keyword in gloss_keywords {
                    writer.write(XmlEvent::start_element("keyword"))?;
                    writer.write(XmlEvent::start_element("word"))?;
                    writer.write(XmlEvent::Characters(&keyword.word))?;
                    writer.write(XmlEvent::end_element())?;
                    if let Some(meaning) = &keyword.meaning {
                        writer.write(XmlEvent::start_element("meaning"))?;
                        writer.write(XmlEvent::Characters(meaning))?;
                        writer.write(XmlEvent::end_element())?;
                    }
                    writer.write(XmlEvent::end_element())?;
                }
                writer.write(XmlEvent::end_element())?;
            }
        }

        // Add place keywords
        if let Some(place_keywords) = place_keywords {
            if !place_keywords.is_empty() {
                writer.write(XmlEvent::start_element("place_keywords"))?;
                for keyword in place_keywords {
                    writer.write(XmlEvent::start_element("keyword"))?;
                    writer.write(XmlEvent::start_element("word"))?;
                    writer.write(XmlEvent::Characters(&keyword.word))?;
                    writer.write(XmlEvent::end_element())?;
                    if let Some(meaning) = &keyword.meaning {
                        writer.write(XmlEvent::start_element("meaning"))?;
                        writer.write(XmlEvent::Characters(meaning))?;
                        writer.write(XmlEvent::end_element())?;
                    }
                    writer.write(XmlEvent::end_element())?;
                }
                writer.write(XmlEvent::end_element())?;
            }
        }

        writer.write(XmlEvent::end_element())?;
    }
    writer.write(XmlEvent::end_element())?;
    writer.write(XmlEvent::end_element())?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| e.into())
}

// Helper function to create CollectionExportItem from a row
impl CollectionExportItem {
    fn from_row(
        row: tokio_postgres::Row,
        front_image_url: Option<String>,
        back_image_url: Option<String>,
    ) -> Self {
        CollectionExportItem {
            item_id: row.get("item_id"),
            position: row.get("position"),
            collection_note: row.get("collection_note"),
            definition_id: row.get("definition_id"),
            word: row.get("word"),
            word_type: row.get("word_type"),
            rafsi: row.get("rafsi"),
            selmaho: row.get("selmaho"),
            language_id: row.get("language_id"),
            owner_user_id: row.get("owner_user_id"),
            license: row.get("license"),
            definition: row.get("definition"),
            definition_notes: row.get("definition_notes"),
            jargon: row.get("jargon"),
            free_content_front: row.get("free_content_front"),
            free_content_back: row.get("free_content_back"),
            front_image_url,
            back_image_url,
            direction: None, // only set in full collection export
            level_index: None,
            position_in_level: None,
        }
    }
}

async fn generate_latex(
    transaction: &mut Transaction<'_>,
    lang: &str,
    options: &ExportOptions,
    collection_id: Option<i32>,
    source_langid: i32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(id) = collection_id {
        // Handle collection export
        let query = "
            SELECT
                ci.item_id, ci.definition_id, ci.notes as collection_note, ci.position,
                ci.free_content_front, ci.free_content_back, 
                ci.langid as language_id, ci.owner_user_id, ci.license,
                v.word, d.definition, d.notes as definition_notes, t.descriptor as word_type,
                c.rafsi, c.selmaho,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_mime,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_mime
            FROM collection_items ci
            LEFT JOIN definitions d ON ci.definition_id = d.definitionid
            LEFT JOIN valsi v ON d.valsiid = v.valsiid
            LEFT JOIN valsitypes t ON v.typeid = t.typeid
            LEFT JOIN convenientdefinitions c ON c.definitionid = d.definitionid
            WHERE ci.collection_id = $1
            ORDER BY ci.position";

        let rows = transaction.query(query, &[&id]).await?;

        let entries: Vec<CollectionExportItem> = rows
            .into_iter()
            .map(|row| {
                let front_image_url =
                    row.get::<_, Option<Vec<u8>>>("front_image_data")
                        .and_then(|data| {
                            row.get::<_, Option<String>>("front_image_mime")
                                .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(data)))
                        });
                let back_image_url =
                    row.get::<_, Option<Vec<u8>>>("back_image_data")
                        .and_then(|data| {
                            row.get::<_, Option<String>>("back_image_mime")
                                .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(data)))
                        });
                CollectionExportItem::from_row(row, front_image_url, back_image_url)
            })
            .collect();
        return Ok(serde_json::to_string_pretty(&entries)?);
    }

    let lang_row = transaction
        .query_one(
            "SELECT tag, realname FROM languages WHERE tag = $1",
            &[&lang],
        )
        .await?;

    let lang_realname: String = lang_row.get("realname");
    let escaped_lang = escape_all(&lang_realname);

    let mut title = generate_title(&escaped_lang, collection_id);
    if let Some(id) = collection_id {
        let collection_name = transaction
            .query_one(
                "SELECT name FROM collections WHERE collection_id = $1",
                &[&id],
            )
            .await?
            .get::<_, String>("name");
        title = format!("{} - {}", title, escape_all(&collection_name));
    }

    let content = if let Some(cid) = collection_id {
        // Generate LaTeX specifically for a collection
        generate_collection_latex(transaction, lang, cid, source_langid).await?
    } else {
        // Generate standard dictionary chapters
        generate_chapters(
            transaction,
            lang,
            &escaped_lang,
            None,
            options,
            source_langid,
        )
        .await?
    };

    Ok(format!(
        "{}\n{}\n{}",
        latex_header(&title, lang),
        content,
        latex_footer()
    ))
}

async fn generate_chapters(
    transaction: &mut Transaction<'_>,
    lang: &str,
    escaped_lang: &str,
    collection_id: Option<i32>,
    options: &ExportOptions,
    source_langid: i32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let lang_id: i32 = transaction
        .query_one("SELECT langid FROM languages WHERE tag = $1", &[&lang])
        .await?
        .get(0);

    if escaped_lang == "lojban" {
        generate_lojban_chapter(
            transaction,
            lang_id,
            lang,
            "lo smuni be bau la .lojban.",
            collection_id,
            options,
            source_langid,
        )
        .await
    } else {
        generate_lojban_and_natural_chapters(
            transaction,
            lang_id,
            lang,
            escaped_lang,
            collection_id,
            options,
            source_langid,
        )
        .await
    }
}

async fn generate_lojban_chapter(
    transaction: &mut Transaction<'_>,
    lang_id: i32,
    lang: &str,
    title: &str,
    collection_id: Option<i32>,
    options: &ExportOptions,
    source_langid: i32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let entries = generate_lojban_entries(
        transaction,
        lang_id,
        lang,
        collection_id,
        options,
        source_langid,
    )
    .await?;
    Ok(format!("\\chapter{{{}}}{}", title, entries))
}

async fn generate_collection_latex(
    transaction: &mut Transaction<'_>,
    lang: &str, // lang tag needed for escape_tex logic
    collection_id: i32,
    _source_langid: i32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut entries = String::new();

    let query = "
        SELECT
            ci.item_id, ci.definition_id, ci.notes as collection_note, ci.position,
            ci.free_content_front, ci.free_content_back,
            v.word, d.definition, d.notes as definition_notes, t.descriptor as word_type,
            c.rafsi, c.selmaho
        FROM collection_items ci
        LEFT JOIN definitions d ON ci.definition_id = d.definitionid
        LEFT JOIN valsi v ON d.valsiid = v.valsiid
        LEFT JOIN valsitypes t ON v.typeid = t.typeid
        LEFT JOIN convenientdefinitions c ON c.definitionid = d.definitionid
        WHERE ci.collection_id = $1
        ORDER BY ci.position";

    let rows = transaction.query(query, &[&collection_id]).await?;

    for row in rows {
        if row.get::<_, Option<i32>>("definition_id").is_some() {
            // Format as definition-based item
            let valsi_row = ValsiRow::from_collection_row(&row)?;
            entries.push_str(&format_lojban_entry(&valsi_row, lang));
        } else {
            // Format as free-content item
            entries.push_str(&format_free_content_entry(&row, lang));
        }
    }

    Ok(entries)
}

fn format_lojban_entry(valsi_row: &ValsiRow, lang: &str) -> String {
    let mut entry = format_lojban_heading(&valsi_row.word, &valsi_row.descriptor);
    entry.push_str(&format_rafsi(&valsi_row.rafsi));
    entry.push_str(&format_selmaho(&valsi_row.selmaho));
    entry.push_str(&format_definition(&valsi_row.definition, lang));
    entry.push_str(&format_notes(&valsi_row.notes));
    if let Some(note) = &valsi_row.collection_note {
        if !note.is_empty() {
            entry.push_str(&format_collection_note(note));
        }
    }
    entry
}

// Helper to create ValsiRow from collection item row
impl ValsiRow {
    fn from_collection_row(
        row: &tokio_postgres::Row,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ValsiRow {
            word: row.try_get("word")?,
            rafsi: row.try_get("rafsi")?,
            selmaho: row.try_get("selmaho")?,
            definition: row.try_get("definition")?,
            notes: row.try_get("definition_notes")?, // Use definition_notes alias
            collection_note: row.try_get("collection_note")?,
            descriptor: row.try_get("word_type")?, // Use word_type alias
        })
    }
}

fn format_free_content_entry(row: &tokio_postgres::Row, lang: &str) -> String {
    let front: String = row.get("free_content_front");
    let back: String = row.get("free_content_back");
    let note: Option<String> = row.get("collection_note");
    format_free_content_parts(&front, &back, note.as_deref(), lang)
}

fn format_free_content_parts(front: &str, back: &str, note: Option<&str>, lang: &str) -> String {
    format!(
        "\n\n{{\\sffamily\\bfseries {}}} \\enspace {} {}",
        escape_all(front),
        format_definition(back, lang),
        format_collection_note(note.unwrap_or_default())
    )
}

fn replace_newlines(s: &str) -> String {
    s.replace(['\n', '\r'], " ")
}

fn format_collection_note(note: &str) -> String {
    if !note.is_empty() {
        if sniff_tex(note) {
            format!(" \\textbf{{Collection note:}} {}", escape_tex(note, false))
        } else {
            format!(" \\textbf{{Collection note:}} {}", escape_all(note))
        }
    } else {
        String::new()
    }
}

async fn generate_lojban_and_natural_chapters(
    transaction: &mut Transaction<'_>,
    lang_id: i32,
    lang: &str,
    escaped_lang: &str,
    collection_id: Option<i32>,
    options: &ExportOptions,
    source_langid: i32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let (vlaste_from_jbo, vlaste_to_jbo) = if collection_id.is_some() {
        (
            "fanva fo la .lojban.".to_string(),
            "fanva fi la .lojban.".to_string(),
        )
    } else {
        (
            format!("fanva fi la'o zoi {} zoi", escaped_lang),
            format!("fanva fo la'o zoi {} zoi", escaped_lang),
        )
    };

    let lojban_chapter = generate_lojban_chapter(
        transaction,
        lang_id,
        lang,
        &vlaste_from_jbo,
        collection_id,
        options,
        source_langid,
    )
    .await?;

    // Check if there are any natural language entries before generating that chapter
    let has_natural_entries =
        check_natural_entries(transaction, lang_id, collection_id, options, source_langid).await?;

    if has_natural_entries {
        let natural_chapter =
            generate_natural_chapter(transaction, lang_id, collection_id, options, source_langid)
                .await?;
        Ok(format!(
            "{}\n\\chapter{{{}}}{}",
            lojban_chapter, vlaste_to_jbo, natural_chapter
        ))
    } else {
        Ok(lojban_chapter)
    }
}

async fn check_natural_entries(
    transaction: &mut Transaction<'_>,
    lang_id: i32,
    collection_id: Option<i32>,
    options: &ExportOptions,
    source_langid: i32,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let positive_scores_only = options.positive_scores_only.unwrap_or(true);

    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = bd.definitionid")
        .unwrap_or("");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let query = format!(
        "SELECT EXISTS (
            SELECT 1
            FROM export_best_definitions($1, $3) bd
            JOIN valsi v ON v.valsiid = bd.valsiid
            JOIN natlangwordbestplaces nlwbp ON nlwbp.definitionid = bd.definitionid
            JOIN natlangwords nlw ON nlw.wordid = nlwbp.wordid
            {}
            WHERE v.source_langid = $2 {}
              AND (nlwbp.score > 0 OR $3 = false)
              AND EXISTS (
                  SELECT 1
                  FROM keywordmapping km
                  WHERE km.natlangwordid = nlw.wordid AND km.definitionid = nlwbp.definitionid
              )
        )",
        collection_join, collection_condition
    );

    let row = transaction
        .query_one(&query, &[&lang_id, &source_langid, &positive_scores_only])
        .await?;
    Ok(row.get(0))
}

async fn generate_lojban_entries(
    transaction: &mut Transaction<'_>,
    lang_id: i32,
    lang: &str,
    collection_id: Option<i32>,
    options: &ExportOptions,
    source_langid: i32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut entries = String::new();
    let positive_scores_only = options.positive_scores_only.unwrap_or(true);

    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = bd.definitionid")
        .unwrap_or("");
    let collection_note_select = collection_id
        .map(|_| ", ci.notes as collection_note")
        .unwrap_or(", NULL as collection_note");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let query = format!(
        "SELECT v.word, c.rafsi, c.selmaho, c.definition,
                c.notes, t.descriptor{}
         FROM export_best_definitions($1, $3) bd
         JOIN valsi v ON v.valsiid = bd.valsiid
         JOIN convenientdefinitions c ON c.definitionid = bd.definitionid
         JOIN valsitypes t ON t.typeid = v.typeid
         {}
         WHERE v.source_langid = $2 {}
         ORDER BY lower(v.word)",
        collection_note_select, collection_join, collection_condition
    );

    let params: Vec<&(dyn postgres_types::ToSql + Sync)> =
        vec![&lang_id, &source_langid, &positive_scores_only];

    let rows = transaction.query(&query, &params[..]).await?;

    for row in rows {
        let valsi_row = ValsiRow {
            word: row.get("word"),
            rafsi: row.get("rafsi"),
            selmaho: row.get("selmaho"),
            definition: row.get("definition"),
            notes: row.get("notes"),
            collection_note: row.get("collection_note"),
            descriptor: row.get("descriptor"),
        };
        entries.push_str(&format_lojban_entry(&valsi_row, lang));
    }

    Ok(entries)
}

async fn generate_natural_chapter(
    transaction: &mut Transaction<'_>,
    lang_id: i32,
    collection_id: Option<i32>,
    options: &ExportOptions,
    source_langid: i32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let positive_scores_only = options.positive_scores_only.unwrap_or(true);

    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = bd.definitionid")
        .unwrap_or("");
    let collection_note_select = collection_id
        .map(|_| ", ci.notes as collection_note")
        .unwrap_or(", NULL as collection_note");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let query = format!(
        "SELECT nlw.word, nlw.meaning, v.word as valsi, nlwbp.place{}
         FROM export_best_definitions($1, $3) bd
         JOIN valsi v ON v.valsiid = bd.valsiid
         JOIN natlangwordbestplaces nlwbp ON nlwbp.definitionid = bd.definitionid
         JOIN natlangwords nlw ON nlw.wordid = nlwbp.wordid
         {}
         WHERE v.source_langid = $2 {}
           AND (nlwbp.score > 0 OR $3 = false)
           AND EXISTS (
             SELECT 1
             FROM keywordmapping km
             WHERE km.natlangwordid = nlw.wordid AND km.definitionid = nlwbp.definitionid
           )
         ORDER BY nlw.word",
        collection_note_select, collection_join, collection_condition
    );

    let rows = transaction
        .query(&query, &[&lang_id, &source_langid, &positive_scores_only])
        .await?;
    let mut entries = String::new();

    for row in rows {
        let entry = format_natural_entry(NaturalEntry {
            word: row.get("word"),
            meaning: row.get("meaning"),
            valsi: row.get("valsi"),
            place: row.get("place"),
            collection_note: row.get("collection_note"),
        });
        entries.push_str(&entry);
    }

    Ok(entries)
}

fn format_natural_entry(entry: NaturalEntry) -> String {
    let mut result = format_natural_heading(&entry.word);
    result.push_str(&format_meaning(&entry.meaning));
    result.push_str(&format_valsi(&entry.valsi));
    result.push_str(&format_place(entry.place));
    if let Some(note) = entry.collection_note {
        if !note.is_empty() {
            result.push_str(&format_collection_note(&note));
        }
    }
    result
}

async fn generate_tsv(
    transaction: &mut Transaction<'_>,
    lang: &str,
    options: &ExportOptions,
    collection_id: Option<i32>,
    source_langid: i32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(id) = collection_id {
        // Handle collection export
        let query = "
            SELECT
                ci.item_id, ci.definition_id, ci.notes as collection_note, ci.position,
                ci.free_content_front, ci.free_content_back, 
                ci.langid as language_id, ci.owner_user_id, ci.license,
                v.word, d.definition, d.notes as definition_notes, d.jargon, t.descriptor as word_type,
                c.rafsi, c.selmaho,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_mime,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_mime
            FROM collection_items ci
            LEFT JOIN definitions d ON ci.definition_id = d.definitionid
            LEFT JOIN valsi v ON d.valsiid = v.valsiid
            LEFT JOIN valsitypes t ON v.typeid = t.typeid
            LEFT JOIN convenientdefinitions c ON c.definitionid = d.definitionid
            WHERE ci.collection_id = $1
            ORDER BY ci.position";

        let rows = transaction.query(query, &[&id]).await?;

        let entries: Vec<CollectionExportItem> = rows
            .into_iter()
            .map(|row| {
                let front_image_url =
                    row.get::<_, Option<Vec<u8>>>("front_image_data")
                        .and_then(|data| {
                            row.get::<_, Option<String>>("front_image_mime")
                                .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(data)))
                        });
                let back_image_url =
                    row.get::<_, Option<Vec<u8>>>("back_image_data")
                        .and_then(|data| {
                            row.get::<_, Option<String>>("back_image_mime")
                                .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(data)))
                        });
                CollectionExportItem::from_row(row, front_image_url, back_image_url)
            })
            .collect();
        return Ok(serde_json::to_string_pretty(&entries)?);
    }

    if let Some(id) = collection_id {
        // Handle collection export
        let query = "
            SELECT
                ci.item_id, ci.definition_id, ci.notes as collection_note, ci.position,
                ci.free_content_front, ci.free_content_back,
                ci.langid as language_id, ci.owner_user_id, ci.license,
                v.word, d.definition, d.notes as definition_notes, d.jargon, t.descriptor as word_type,
                c.rafsi, c.selmaho
            FROM collection_items ci
            LEFT JOIN definitions d ON ci.definition_id = d.definitionid
            LEFT JOIN valsi v ON d.valsiid = v.valsiid
            LEFT JOIN valsitypes t ON v.typeid = t.typeid
            LEFT JOIN convenientdefinitions c ON c.definitionid = d.definitionid
            WHERE ci.collection_id = $1
            ORDER BY ci.position";

        let rows = transaction.query(query, &[&id]).await?;
        let tsv = generate_collection_tsv(rows)?;
        return Ok(tsv);
    }

    let positive_scores_only = options.positive_scores_only.unwrap_or(true);

    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = bd.definitionid")
        .unwrap_or("");
    let collection_note_select = collection_id
        .map(|_| ", ci.notes as collection_note")
        .unwrap_or(", NULL as collection_note");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let query = format!(
        "SELECT v.word, bd.definitionid, c.rafsi, c.selmaho, c.definition,
                c.notes, d.jargon, t.descriptor{}, bd.score
         FROM export_best_definitions($1, $3) bd
         JOIN valsi v ON v.valsiid = bd.valsiid
         JOIN convenientdefinitions c ON c.definitionid = bd.definitionid
         JOIN definitions d ON d.definitionid = bd.definitionid
         JOIN valsitypes t ON t.typeid = v.typeid
         {}
         WHERE v.source_langid = $2 {}
         ORDER BY lower(v.word)",
        collection_note_select, collection_join, collection_condition
    );

    let langid = transaction
        .query_one("SELECT langid FROM languages WHERE tag = $1", &[&lang])
        .await?
        .get::<_, i32>("langid");

    let rows = transaction
        .query(&query, &[&langid, &source_langid, &positive_scores_only])
        .await?;

    // Collect all definition IDs
    let def_ids: Vec<i32> = rows
        .iter()
        .map(|row| row.get::<_, i32>("definitionid"))
        .collect();

    // Fetch gloss keywords and place keywords for all definitions
    let (gloss_map, place_map) = fetch_keywords_for_export(transaction, &def_ids).await?;

    // Determine maximum number of gloss words and place keywords
    let max_gloss_count = gloss_map.values().map(|v| v.len()).max().unwrap_or(0);
    let max_place_count = place_map.values().map(|v| v.len()).max().unwrap_or(0);

    let mut tsv = String::new();
    // Write header
    tsv.push_str("word\ttype\trafsi\tselmaho\tdefinition\tnotes\tjargon\tcollection_note\tscore");

    // Add gloss word columns
    for i in 1..=max_gloss_count {
        tsv.push_str(&format!("\tglossword_{}\tglossword_{}_meaning", i, i));
    }

    // Add place keyword columns
    for i in 1..=max_place_count {
        tsv.push_str(&format!("\tplacekeyword_{}\tplacekeyword_{}_meaning", i, i));
    }

    tsv.push('\n');

    for row in rows.iter() {
        let definition_id: i32 = row.get("definitionid");
        let word: String = row.get("word");
        let descriptor: String = row.get("descriptor");
        let rafsi: Option<String> = row.get("rafsi");
        let selmaho: Option<String> = row.get("selmaho");
        let definition: String = row.get("definition");
        let notes: Option<String> = row.get("notes");
        let jargon: Option<String> = row.get("jargon");
        let collection_note: Option<String> = row.get("collection_note");
        let score: i64 = row.get("score");

        let gloss_keywords = gloss_map.get(&definition_id).cloned().unwrap_or_default();
        let place_keywords = place_map.get(&definition_id).cloned().unwrap_or_default();

        // Start row with basic fields
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            replace_newlines(&word),
            replace_newlines(&descriptor),
            replace_newlines(&rafsi.unwrap_or_default()),
            replace_newlines(&selmaho.unwrap_or_default()),
            replace_newlines(&definition),
            replace_newlines(&notes.unwrap_or_default()),
            replace_newlines(&jargon.unwrap_or_default()),
            replace_newlines(&collection_note.unwrap_or_default()),
            score
        ));

        // Add gloss word columns
        for i in 0..max_gloss_count {
            if let Some(keyword) = gloss_keywords.get(i) {
                let meaning_str = keyword
                    .meaning
                    .as_ref()
                    .map(|m| replace_newlines(m))
                    .unwrap_or_default();
                tsv.push_str(&format!(
                    "\t{}\t{}",
                    replace_newlines(&keyword.word),
                    meaning_str
                ));
            } else {
                tsv.push_str("\t\t");
            }
        }

        // Add place keyword columns
        for i in 0..max_place_count {
            if let Some(keyword) = place_keywords.get(i) {
                let meaning_str = keyword
                    .meaning
                    .as_ref()
                    .map(|m| replace_newlines(m))
                    .unwrap_or_default();
                tsv.push_str(&format!(
                    "\t{}\t{}",
                    replace_newlines(&keyword.word),
                    meaning_str
                ));
            } else {
                tsv.push_str("\t\t");
            }
        }

        tsv.push('\n');
    }

    Ok(tsv)
}

fn generate_collection_tsv(
    rows: Vec<tokio_postgres::Row>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut tsv = String::new();
    // Write header for collection items
    tsv.push_str("item_id\tposition\tdefinition_id\tword\tword_type\trafsi\tselmaho\tdefinition\tdefinition_notes\tjargon\tfree_content_front\tfree_content_back\tcollection_note\n");

    for row in rows {
        let item_id: i32 = row.get("item_id");
        let position: i32 = row.get("position");
        let definition_id: Option<i32> = row.get("definition_id");
        let word: Option<String> = row.get("word");
        let word_type: Option<String> = row.get("word_type");
        let rafsi: Option<String> = row.get("rafsi");
        let selmaho: Option<String> = row.get("selmaho");
        let definition: Option<String> = row.get("definition");
        let definition_notes: Option<String> = row.get("definition_notes");
        let jargon: Option<String> = row.get("jargon");
        let free_content_front: Option<String> = row.get("free_content_front");
        let free_content_back: Option<String> = row.get("free_content_back");
        let collection_note: Option<String> = row.get("collection_note");

        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            item_id,
            position,
            definition_id.map(|id| id.to_string()).unwrap_or_default(),
            replace_newlines(&word.unwrap_or_default()),
            replace_newlines(&word_type.unwrap_or_default()),
            replace_newlines(&rafsi.unwrap_or_default()),
            replace_newlines(&selmaho.unwrap_or_default()),
            replace_newlines(&definition.unwrap_or_default()),
            replace_newlines(&definition_notes.unwrap_or_default()),
            replace_newlines(&jargon.unwrap_or_default()),
            replace_newlines(&free_content_front.unwrap_or_default()),
            replace_newlines(&free_content_back.unwrap_or_default()),
            replace_newlines(&collection_note.unwrap_or_default())
        ));
    }
    Ok(tsv)
}

async fn generate_json(
    transaction: &mut Transaction<'_>,
    lang: &str,
    options: &ExportOptions,
    collection_id: Option<i32>,
    source_langid: i32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(id) = collection_id {
        // Handle collection export
        let query = "
            SELECT
                ci.item_id, ci.definition_id, ci.notes as collection_note, ci.position,
                ci.free_content_front, ci.free_content_back, 
                ci.langid as language_id, ci.owner_user_id, ci.license,
                v.word, d.definition, d.notes as definition_notes, d.jargon, t.descriptor as word_type,
                c.rafsi, c.selmaho,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_mime,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_mime
            FROM collection_items ci
            LEFT JOIN definitions d ON ci.definition_id = d.definitionid
            LEFT JOIN valsi v ON d.valsiid = v.valsiid
            LEFT JOIN valsitypes t ON v.typeid = t.typeid
            LEFT JOIN convenientdefinitions c ON c.definitionid = d.definitionid
            WHERE ci.collection_id = $1
            ORDER BY ci.position";

        let rows = transaction.query(query, &[&id]).await?;

        let entries: Vec<CollectionExportItem> = rows
            .into_iter()
            .map(|row| {
                let front_image_url =
                    row.get::<_, Option<Vec<u8>>>("front_image_data")
                        .and_then(|data| {
                            row.get::<_, Option<String>>("front_image_mime")
                                .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(data)))
                        });
                let back_image_url =
                    row.get::<_, Option<Vec<u8>>>("back_image_data")
                        .and_then(|data| {
                            row.get::<_, Option<String>>("back_image_mime")
                                .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(data)))
                        });
                CollectionExportItem::from_row(row, front_image_url, back_image_url)
            })
            .collect();
        return Ok(serde_json::to_string_pretty(&entries)?);
    }

    let positive_scores_only = options.positive_scores_only.unwrap_or(true);

    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = bd.definitionid")
        .unwrap_or("");
    let collection_note_select = collection_id
        .map(|_| ", ci.notes as collection_note")
        .unwrap_or(", NULL as collection_note");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let query = format!(
        "SELECT v.word, bd.definitionid, c.rafsi, c.selmaho, c.definition,
                c.notes, d.etymology, d.jargon, t.descriptor{}, u.username, u.realname, bd.score
         FROM export_best_definitions($1, $3) bd
         JOIN valsi v ON v.valsiid = bd.valsiid
         JOIN convenientdefinitions c ON c.definitionid = bd.definitionid
         JOIN definitions d ON d.definitionid = bd.definitionid
         JOIN valsitypes t ON t.typeid = v.typeid
         LEFT JOIN users u ON u.userid = d.userid
         {}
         WHERE v.source_langid = $2 {}
         ORDER BY lower(v.word)",
        collection_note_select, collection_join, collection_condition
    );

    let langid = transaction
        .query_one("SELECT langid FROM languages WHERE tag = $1", &[&lang])
        .await?
        .get::<_, i32>("langid");

    let rows = transaction
        .query(&query, &[&langid, &source_langid, &positive_scores_only])
        .await?;

    // Collect all definition IDs
    let def_ids: Vec<i32> = rows
        .iter()
        .map(|row| row.get::<_, i32>("definitionid"))
        .collect();

    // Fetch gloss keywords and place keywords for all definitions
    let (gloss_map, place_map) = fetch_keywords_for_export(transaction, &def_ids).await?;

    let entries: Vec<DictionaryEntry> = rows
        .into_iter()
        .map(|row| {
            let definition_id: i32 = row.get("definitionid");
            DictionaryEntry {
                definition_id: Some(definition_id),
                word: row.get("word"),
                word_type: row.get("descriptor"),
                rafsi: row.get("rafsi"),
                selmaho: row.get("selmaho"),
                definition: row.get("definition"),
                notes: row.get("notes"),
                etymology: row.get("etymology"),
                jargon: row.get("jargon"),
                collection_note: row.get("collection_note"),
                score: row.get::<_, i64>("score") as f32,
                gloss_keywords: gloss_map.get(&definition_id).cloned(),
                place_keywords: place_map.get(&definition_id).cloned(),
                user: row.get::<_, Option<String>>("username").map(|user| User {
                    username: user,
                    realname: row.get::<_, Option<String>>("realname"),
                }),
            }
        })
        .collect();

    Ok(serde_json::to_string_pretty(&entries)?)
}

pub async fn list_cached_exports(
    pool: &Pool,
) -> Result<Vec<CachedExport>, Box<dyn Error + Send + Sync>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Only list exports that are still within the cache TTL.
    let rows = transaction
        .query(
            "SELECT cde.language_tag, cde.source_language_tag,
                l.realname AS language_realname, cde.format,
                cde.positive_scores_only, cde.filename, cde.created_at
         FROM cached_dictionary_exports cde
         JOIN languages l ON cde.language_tag = l.tag
         WHERE cde.created_at > NOW() - INTERVAL '4 days'
         ORDER BY l.realname",
            &[],
        )
        .await?;

    let exports = rows
        .into_iter()
        .map(|row| CachedExport {
            language_tag: row.get("language_tag"),
            source_language_tag: row.get("source_language_tag"),
            language_realname: row.get("language_realname"),
            format: row.get("format"),
            positive_scores_only: row.get("positive_scores_only"),
            filename: row.get("filename"),
            created_at: row.get("created_at"),
        })
        .collect();

    transaction.commit().await?;
    Ok(exports)
}

pub async fn get_cached_export(
    pool: &Pool,
    language_tag: &str,
    source_language_tag: &str,
    format: &str,
    positive_scores_only: bool,
) -> Result<(Vec<u8>, String, String), Box<dyn Error + Send + Sync>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let row = transaction
        .query_opt(
            "SELECT content, content_type, filename
         FROM cached_dictionary_exports
         WHERE language_tag = $1
           AND source_language_tag = $2
           AND format = $3
           AND positive_scores_only = $4
           AND created_at > NOW() - INTERVAL '4 days'",
            &[
                &language_tag,
                &source_language_tag,
                &format,
                &positive_scores_only,
            ],
        )
        .await?;

    let result = match row {
        Some(row) => Ok((
            row.get("content"),
            row.get("content_type"),
            row.get("filename"),
        )),
        None => Err("Export not found".into()),
    };

    transaction.commit().await?;
    result
}

pub async fn export_all_dictionaries(pool: &Pool) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = pool.get().await?;

    let languages = client
        .query("SELECT tag FROM languages", &[])
        .await?
        .iter()
        .map(|row| row.get::<_, String>("tag"))
        .collect::<Vec<_>>();

    // Check existing cached exports (do not hold a transaction across long-running generates).
    // The background job only pre-warms the canonical Lojban-source, positive-scores-only
    // variant; other variants are generated on demand via the interactive endpoint.
    let cached_exports_rows = client
        .query(
            "SELECT language_tag, source_language_tag, format, positive_scores_only, MAX(created_at) as last_export
             FROM cached_dictionary_exports
             GROUP BY language_tag, source_language_tag, format, positive_scores_only",
            &[],
        )
        .await?;

    let mut cached_exports = std::collections::HashMap::new();
    for row in cached_exports_rows {
        let lang_tag: String = row.get("language_tag");
        let source_tag: String = row.get("source_language_tag");
        let format: String = row.get("format");
        let positive_only: bool = row.get("positive_scores_only");
        let last_export: DateTime<Utc> = row.get("last_export");
        cached_exports.insert((lang_tag, source_tag, format, positive_only), last_export);
    }

    let canonical_options = ExportOptions {
        format: None,
        positive_scores_only: Some(true),
        collection_id: None,
        source_lang: Some(DEFAULT_SOURCE_LANGUAGE_TAG.to_string()),
    };

    for lang in languages {
        for format in &[
            ExportFormat::Pdf,
            ExportFormat::LaTeX,
            ExportFormat::Xml,
            ExportFormat::Json,
            ExportFormat::Tsv,
        ] {
            let format_str = format.to_string();
            let cache_key = (
                lang.clone(),
                DEFAULT_SOURCE_LANGUAGE_TAG.to_string(),
                format_str.clone(),
                true,
            );
            if let Some(last_export_time) = cached_exports.get(&cache_key) {
                let duration_since_last =
                    chrono::Utc::now().signed_duration_since(*last_export_time);
                if duration_since_last < chrono::Duration::days(1) {
                    info!(
                        "Skipping {} {} export - last cached at {}",
                        lang, format, last_export_time
                    );
                    continue;
                }
            }

            info!(
                "Exporting dictionary for language {} in format {}",
                lang, format
            );

            // Must bypass DB cache read or we would keep re-storing stale blobs (< 4 days old).
            match export_dictionary(
                pool,
                &lang,
                *format,
                &canonical_options,
                None,
                DEFAULT_SOURCE_LANGID,
                DEFAULT_SOURCE_LANGUAGE_TAG,
                false,
            )
            .await
            {
                Ok((content, content_type, filename)) => {
                    let c = pool.get().await?;
                    if let Err(e) = c
                        .execute(
                            "INSERT INTO cached_dictionary_exports
                             (language_tag, source_language_tag, format, positive_scores_only, content, content_type, filename)
                             VALUES ($1, $2, $3, $4, $5, $6, $7)
                             ON CONFLICT (language_tag, source_language_tag, format, positive_scores_only)
                             DO UPDATE SET
                                content = EXCLUDED.content,
                                content_type = EXCLUDED.content_type,
                                filename = EXCLUDED.filename,
                                created_at = CURRENT_TIMESTAMP",
                            &[
                                &lang,
                                &DEFAULT_SOURCE_LANGUAGE_TAG,
                                &format.to_string(),
                                &true,
                                &content,
                                &content_type,
                                &filename,
                            ],
                        )
                        .await
                    {
                        error!("Failed to cache export for {}: {}", lang, e);
                    }
                }
                Err(e) => error!("Failed to export {} dictionary to {}: {}", lang, format, e),
            }
        }
    }

    Ok(())
}

fn parse_i32_csv(value: &Option<String>) -> Option<Vec<i32>> {
    let ids: Vec<i32> = value
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter_map(|s| s.trim().parse::<i32>().ok())
        .filter(|n| *n > 0)
        .collect();
    if ids.is_empty() {
        None
    } else {
        Some(ids)
    }
}

fn dictionary_entry_from_detail(d: crate::jbovlaste::DefinitionDetail) -> DictionaryEntry {
    DictionaryEntry {
        word: d.valsiword,
        word_type: d.type_name,
        rafsi: d.rafsi,
        selmaho: d.selmaho,
        definition: d.definition,
        definition_id: Some(d.definitionid),
        notes: d.notes,
        etymology: d.etymology,
        jargon: d.jargon,
        collection_note: None,
        score: d.score,
        gloss_keywords: d.gloss_keywords,
        place_keywords: d.place_keywords,
        user: Some(User {
            username: d.username,
            realname: None,
        }),
    }
}

fn valsi_row_from_dictionary_entry(entry: &DictionaryEntry) -> ValsiRow {
    ValsiRow {
        word: entry.word.clone(),
        rafsi: entry.rafsi.clone(),
        selmaho: entry.selmaho.clone(),
        definition: entry.definition.clone(),
        notes: entry.notes.clone(),
        collection_note: entry.collection_note.clone(),
        descriptor: entry.word_type.clone(),
    }
}

fn format_collection_export_item(item: &CollectionExportItem, lang: &str) -> String {
    if item.definition_id.is_some() && item.word.as_ref().is_some_and(|w| !w.is_empty()) {
        let row = ValsiRow {
            word: item.word.clone().unwrap_or_default(),
            rafsi: item.rafsi.clone(),
            selmaho: item.selmaho.clone(),
            definition: item.definition.clone().unwrap_or_default(),
            notes: item.definition_notes.clone(),
            collection_note: item.collection_note.clone(),
            descriptor: item.word_type.clone().unwrap_or_default(),
        };
        format_lojban_entry(&row, lang)
    } else {
        format_free_content_parts(
            item.free_content_front.as_deref().unwrap_or(""),
            item.free_content_back.as_deref().unwrap_or(""),
            item.collection_note.as_deref(),
            lang,
        )
    }
}

fn generate_search_export_latex(
    collection_items: &[CollectionExportItem],
    definitions: &[DictionaryEntry],
    lang: &str,
) -> String {
    let escaped_lang = escape_all(lang);
    let title = generate_title(&escaped_lang, Some(0));
    let mut body = String::new();
    if !collection_items.is_empty() {
        body.push_str("\\chapter{lo liste}");
        for item in collection_items {
            body.push_str(&format_collection_export_item(item, lang));
        }
    }
    if !definitions.is_empty() {
        body.push_str("\\chapter{lo vlaste}");
        for entry in definitions {
            body.push_str(&format_lojban_entry(
                &valsi_row_from_dictionary_entry(entry),
                lang,
            ));
        }
    }
    format!(
        "{}\n{}\n{}",
        latex_header(&title, lang),
        body,
        latex_footer()
    )
}

fn write_keyword_xml(
    writer: &mut EventWriter<Cursor<Vec<u8>>>,
    tag: &str,
    keywords: &[crate::jbovlaste::KeywordMapping],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if keywords.is_empty() {
        return Ok(());
    }
    writer.write(XmlEvent::start_element(tag))?;
    for keyword in keywords {
        writer.write(XmlEvent::start_element("keyword"))?;
        writer.write(XmlEvent::start_element("word"))?;
        writer.write(XmlEvent::Characters(&keyword.word))?;
        writer.write(XmlEvent::end_element())?;
        if let Some(meaning) = &keyword.meaning {
            writer.write(XmlEvent::start_element("meaning"))?;
            writer.write(XmlEvent::Characters(meaning))?;
            writer.write(XmlEvent::end_element())?;
        }
        writer.write(XmlEvent::end_element())?;
    }
    writer.write(XmlEvent::end_element())?;
    Ok(())
}

fn write_optional_xml(
    writer: &mut EventWriter<Cursor<Vec<u8>>>,
    tag: &str,
    value: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(v) = value.filter(|s| !s.is_empty()) {
        writer.write(XmlEvent::start_element(tag))?;
        writer.write(XmlEvent::Characters(v))?;
        writer.write(XmlEvent::end_element())?;
    }
    Ok(())
}

fn generate_search_export_xml(
    collection_items: &[CollectionExportItem],
    definitions: &[DictionaryEntry],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut writer = EventWriter::new(Cursor::new(Vec::new()));
    writer.write(XmlEvent::StartDocument {
        version: xml::common::XmlVersion::Version10,
        encoding: Some("UTF-8"),
        standalone: None,
    })?;
    writer.write(XmlEvent::start_element("export"))?;

    writer.write(XmlEvent::start_element("collection_items"))?;
    for item in collection_items {
        writer.write(XmlEvent::start_element("item"))?;
        writer.write(XmlEvent::start_element("item_id"))?;
        writer.write(XmlEvent::Characters(&item.item_id.to_string()))?;
        writer.write(XmlEvent::end_element())?;
        writer.write(XmlEvent::start_element("position"))?;
        writer.write(XmlEvent::Characters(&item.position.to_string()))?;
        writer.write(XmlEvent::end_element())?;
        write_optional_xml(&mut writer, "word", item.word.as_deref())?;
        write_optional_xml(&mut writer, "word_type", item.word_type.as_deref())?;
        write_optional_xml(&mut writer, "rafsi", item.rafsi.as_deref())?;
        write_optional_xml(&mut writer, "selmaho", item.selmaho.as_deref())?;
        write_optional_xml(&mut writer, "definition", item.definition.as_deref())?;
        write_optional_xml(
            &mut writer,
            "definition_notes",
            item.definition_notes.as_deref(),
        )?;
        write_optional_xml(&mut writer, "jargon", item.jargon.as_deref())?;
        write_optional_xml(
            &mut writer,
            "free_content_front",
            item.free_content_front.as_deref(),
        )?;
        write_optional_xml(
            &mut writer,
            "free_content_back",
            item.free_content_back.as_deref(),
        )?;
        write_optional_xml(
            &mut writer,
            "collection_note",
            item.collection_note.as_deref(),
        )?;
        writer.write(XmlEvent::end_element())?;
    }
    writer.write(XmlEvent::end_element())?;

    writer.write(XmlEvent::start_element("definitions"))?;
    for entry in definitions {
        writer.write(XmlEvent::start_element("entry"))?;
        writer.write(XmlEvent::start_element("word"))?;
        writer.write(XmlEvent::Characters(&entry.word))?;
        writer.write(XmlEvent::end_element())?;
        writer.write(XmlEvent::start_element("type"))?;
        writer.write(XmlEvent::Characters(&entry.word_type))?;
        writer.write(XmlEvent::end_element())?;
        write_optional_xml(&mut writer, "rafsi", entry.rafsi.as_deref())?;
        write_optional_xml(&mut writer, "selmaho", entry.selmaho.as_deref())?;
        writer.write(XmlEvent::start_element("definition"))?;
        writer.write(XmlEvent::Characters(&entry.definition))?;
        writer.write(XmlEvent::end_element())?;
        write_optional_xml(&mut writer, "notes", entry.notes.as_deref())?;
        write_optional_xml(&mut writer, "etymology", entry.etymology.as_deref())?;
        write_optional_xml(&mut writer, "jargon", entry.jargon.as_deref())?;
        writer.write(XmlEvent::start_element("score"))?;
        writer.write(XmlEvent::Characters(&entry.score.to_string()))?;
        writer.write(XmlEvent::end_element())?;
        if let Some(keywords) = &entry.gloss_keywords {
            write_keyword_xml(&mut writer, "gloss_keywords", keywords)?;
        }
        if let Some(keywords) = &entry.place_keywords {
            write_keyword_xml(&mut writer, "place_keywords", keywords)?;
        }
        writer.write(XmlEvent::end_element())?;
    }
    writer.write(XmlEvent::end_element())?;
    writer.write(XmlEvent::end_element())?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| e.into())
}

fn generate_definitions_tsv(entries: &[DictionaryEntry]) -> String {
    let max_gloss_count = entries
        .iter()
        .filter_map(|e| e.gloss_keywords.as_ref().map(|v| v.len()))
        .max()
        .unwrap_or(0);
    let max_place_count = entries
        .iter()
        .filter_map(|e| e.place_keywords.as_ref().map(|v| v.len()))
        .max()
        .unwrap_or(0);

    let mut tsv = String::from(
        "word\ttype\trafsi\tselmaho\tdefinition\tnotes\tjargon\tcollection_note\tscore",
    );
    for i in 1..=max_gloss_count {
        tsv.push_str(&format!("\tglossword_{}\tglossword_{}_meaning", i, i));
    }
    for i in 1..=max_place_count {
        tsv.push_str(&format!("\tplacekeyword_{}\tplacekeyword_{}_meaning", i, i));
    }
    tsv.push('\n');

    for entry in entries {
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            replace_newlines(&entry.word),
            replace_newlines(&entry.word_type),
            replace_newlines(entry.rafsi.as_deref().unwrap_or("")),
            replace_newlines(entry.selmaho.as_deref().unwrap_or("")),
            replace_newlines(&entry.definition),
            replace_newlines(entry.notes.as_deref().unwrap_or("")),
            replace_newlines(entry.jargon.as_deref().unwrap_or("")),
            replace_newlines(entry.collection_note.as_deref().unwrap_or("")),
            entry.score
        ));
        let gloss = entry.gloss_keywords.as_deref().unwrap_or(&[]);
        for i in 0..max_gloss_count {
            if let Some(keyword) = gloss.get(i) {
                tsv.push_str(&format!(
                    "\t{}\t{}",
                    replace_newlines(&keyword.word),
                    replace_newlines(keyword.meaning.as_deref().unwrap_or(""))
                ));
            } else {
                tsv.push_str("\t\t");
            }
        }
        let places = entry.place_keywords.as_deref().unwrap_or(&[]);
        for i in 0..max_place_count {
            if let Some(keyword) = places.get(i) {
                tsv.push_str(&format!(
                    "\t{}\t{}",
                    replace_newlines(&keyword.word),
                    replace_newlines(keyword.meaning.as_deref().unwrap_or(""))
                ));
            } else {
                tsv.push_str("\t\t");
            }
        }
        tsv.push('\n');
    }
    tsv
}

fn generate_collection_items_tsv(items: &[CollectionExportItem]) -> String {
    let mut tsv = String::from(
        "item_id\tposition\tdefinition_id\tword\tword_type\trafsi\tselmaho\tdefinition\tdefinition_notes\tjargon\tfree_content_front\tfree_content_back\tcollection_note\n",
    );
    for item in items {
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            item.item_id,
            item.position,
            item.definition_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            replace_newlines(item.word.as_deref().unwrap_or("")),
            replace_newlines(item.word_type.as_deref().unwrap_or("")),
            replace_newlines(item.rafsi.as_deref().unwrap_or("")),
            replace_newlines(item.selmaho.as_deref().unwrap_or("")),
            replace_newlines(item.definition.as_deref().unwrap_or("")),
            replace_newlines(item.definition_notes.as_deref().unwrap_or("")),
            replace_newlines(item.jargon.as_deref().unwrap_or("")),
            replace_newlines(item.free_content_front.as_deref().unwrap_or("")),
            replace_newlines(item.free_content_back.as_deref().unwrap_or("")),
            replace_newlines(item.collection_note.as_deref().unwrap_or(""))
        ));
    }
    tsv
}

async fn resolve_preamble_language_tag(
    pool: &Pool,
    language_ids: &Option<Vec<i32>>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let Some(id) = language_ids.as_ref().and_then(|ids| ids.first().copied()) else {
        return Ok("en".to_string());
    };
    let client = pool.get().await?;
    match client
        .query_opt("SELECT tag FROM languages WHERE langid = $1", &[&id])
        .await?
    {
        Some(row) => Ok(row.get::<_, String>(0)),
        None => Ok("en".to_string()),
    }
}

pub async fn export_search_results(
    pool: &Pool,
    query: &SearchExportQuery,
) -> Result<(Vec<u8>, String, String), Box<dyn std::error::Error + Send + Sync>> {
    if !super::models::has_search_export_constraint(query) {
        return Err(
            "Add a search query or at least one filter (collection, word type, author, selmaho, source language, or search-in-phrases off).".into(),
        );
    }

    let format = ExportFormat::from_query(query.format.as_deref()).map_err(|e| e.to_string())?;
    let search_term = query.search.as_deref().unwrap_or("").trim().to_string();
    let languages = parse_i32_csv(&query.languages);
    let usernames = crate::jbovlaste::dto::parse_username_list(&query.username);
    let exclude_usernames = crate::jbovlaste::dto::parse_username_list(&query.exclude_usernames);
    let selmaho = query
        .selmaho
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let collection_ids = query
        .collection_ids
        .as_deref()
        .map(|s| crate::collections::dto::parse_positive_id_list(s, 50))
        .unwrap_or_default();

    let use_semantic = query.semantic.unwrap_or(false) && !search_term.is_empty();
    if use_semantic {
        if crate::utils::embeddings::embeddings_disabled() {
            return Err("Semantic search is disabled. Use text search instead.".into());
        }
        let embedding = crate::utils::embeddings::get_embedding(&search_term).await?;
        let semantic_embedding = Some(pgvector::Vector::from(embedding.clone()));

        let params = crate::jbovlaste::SearchDefinitionsParams {
            page: 1,
            per_page: SEARCH_EXPORT_ROW_CAP,
            search_term: search_term.clone(),
            include_comments: false,
            sort_by: "word".to_string(),
            sort_order: "asc".to_string(),
            languages: languages.clone(),
            selmaho: selmaho.clone(),
            usernames: usernames.clone(),
            exclude_usernames: exclude_usernames.clone(),
            word_type: query.word_type,
            source_langid: query.source_langid,
            search_in_phrases: query.search_in_phrases,
            include_total_count: true,
            exclude_definition_id: None,
        };

        let collection_fut = async {
            if collection_ids.is_empty() {
                return Ok((Vec::new(), 0_i64));
            }
            let filters = crate::collections::dto::ListCollectionItemsFilters {
                languages: languages.clone(),
                selmaho: selmaho.clone(),
                word_type: query.word_type,
                usernames: usernames.clone(),
                exclude_usernames: exclude_usernames.clone(),
                source_langid: query.source_langid,
                search_in_phrases: query.search_in_phrases,
                semantic_embedding: semantic_embedding.clone(),
            };
            crate::collections::service::search_items_in_collections_for_export(
                pool,
                collection_ids.clone(),
                Some(search_term.clone()),
                filters,
                SEARCH_EXPORT_ROW_CAP,
            )
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })
        };

        let (collection_result, defs_result) = tokio::join!(
            collection_fut,
            crate::jbovlaste::service::semantic_search(pool, params, embedding, None)
        );
        let (collection_items, collection_total) = collection_result?;
        let defs = defs_result.map_err(|e| e.to_string())?;
        return finalize_search_export(
            pool,
            format,
            collection_items,
            collection_total,
            defs.definitions
                .into_iter()
                .map(dictionary_entry_from_detail)
                .collect(),
            defs.total,
            &languages,
        )
        .await;
    }

    let params = crate::jbovlaste::SearchDefinitionsParams {
        page: 1,
        per_page: SEARCH_EXPORT_ROW_CAP,
        search_term: search_term.clone(),
        include_comments: false,
        sort_by: "word".to_string(),
        sort_order: "asc".to_string(),
        languages: languages.clone(),
        selmaho: selmaho.clone(),
        usernames: usernames.clone(),
        exclude_usernames: exclude_usernames.clone(),
        word_type: query.word_type,
        source_langid: query.source_langid,
        search_in_phrases: query.search_in_phrases,
        include_total_count: true,
        exclude_definition_id: None,
    };

    let collection_fut = async {
        if collection_ids.is_empty() {
            return Ok((Vec::new(), 0_i64));
        }
        let filters = crate::collections::dto::ListCollectionItemsFilters {
            languages: languages.clone(),
            selmaho: selmaho.clone(),
            word_type: query.word_type,
            usernames: usernames.clone(),
            exclude_usernames: exclude_usernames.clone(),
            source_langid: query.source_langid,
            search_in_phrases: query.search_in_phrases,
            semantic_embedding: None,
        };
        crate::collections::service::search_items_in_collections_for_export(
            pool,
            collection_ids,
            if search_term.is_empty() {
                None
            } else {
                Some(search_term.clone())
            },
            filters,
            SEARCH_EXPORT_ROW_CAP,
        )
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })
    };

    let (collection_result, defs_result) = tokio::join!(
        collection_fut,
        crate::jbovlaste::service::search_definitions(pool, params, None)
    );
    let (collection_items, collection_total) = collection_result?;
    let defs = defs_result.map_err(|e| e.to_string())?;
    finalize_search_export(
        pool,
        format,
        collection_items,
        collection_total,
        defs.definitions
            .into_iter()
            .map(dictionary_entry_from_detail)
            .collect(),
        defs.total,
        &languages,
    )
    .await
}

async fn finalize_search_export(
    pool: &Pool,
    format: ExportFormat,
    collection_items: Vec<CollectionExportItem>,
    collection_total: i64,
    definitions: Vec<DictionaryEntry>,
    definition_total: i64,
    languages: &Option<Vec<i32>>,
) -> Result<(Vec<u8>, String, String), Box<dyn std::error::Error + Send + Sync>> {
    let combined = collection_total.saturating_add(definition_total);
    if combined > SEARCH_EXPORT_ROW_CAP {
        return Err(format!(
            "Too many matching rows ({}). Narrow the search (limit is {}).",
            combined, SEARCH_EXPORT_ROW_CAP
        )
        .into());
    }
    if collection_items.is_empty() && definitions.is_empty() {
        return Err("No matching definitions or collection items to export.".into());
    }

    let lang = resolve_preamble_language_tag(pool, languages).await?;
    let filename = format!("search-export.{}", format.file_extension());
    let content_type = format.content_type().to_string();

    let content = match format {
        ExportFormat::Pdf => {
            let latex = generate_search_export_latex(&collection_items, &definitions, &lang);
            generate_pdf(&latex).await?
        }
        ExportFormat::LaTeX => {
            generate_search_export_latex(&collection_items, &definitions, &lang).into_bytes()
        }
        ExportFormat::Xml => {
            generate_search_export_xml(&collection_items, &definitions)?.into_bytes()
        }
        ExportFormat::Json => serde_json::to_vec_pretty(&SearchExportJson {
            collection_items,
            definitions,
        })?,
        ExportFormat::Tsv => {
            let mut files: Vec<(String, String)> = Vec::new();
            if !collection_items.is_empty() {
                files.push((
                    "collection-items.tsv".to_string(),
                    generate_collection_items_tsv(&collection_items),
                ));
            }
            if !definitions.is_empty() {
                files.push((
                    "definitions.tsv".to_string(),
                    generate_definitions_tsv(&definitions),
                ));
            }
            let refs: Vec<(&str, &str)> = files
                .iter()
                .map(|(n, c)| (n.as_str(), c.as_str()))
                .collect();
            zip_tsv_files(&refs)?
        }
    };

    Ok((content, content_type, filename))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_filename_includes_source_language_tag() {
        assert_eq!(
            build_export_filename(None, "jbo", "hi", "json"),
            "dictionary-jbo-hi.json"
        );
    }

    #[test]
    fn export_filename_for_collection_uses_target_language_only() {
        assert_eq!(
            build_export_filename(Some(42), "jbo", "hi", "zip"),
            "collection-42-hi.zip"
        );
    }

    #[test]
    fn export_filename_defaults_to_lojban_source_tag() {
        // This is the canonical cache key the background builder pre-warms.
        assert_eq!(
            build_export_filename(None, DEFAULT_SOURCE_LANGUAGE_TAG, "hi", "json"),
            "dictionary-jbo-hi.json"
        );
    }

    #[test]
    fn search_export_constraint_requires_query_or_filter() {
        use super::super::models::{has_search_export_constraint, SearchExportQuery};
        assert!(!has_search_export_constraint(&SearchExportQuery::default()));
        assert!(has_search_export_constraint(&SearchExportQuery {
            search: Some("broda".into()),
            ..Default::default()
        }));
        assert!(has_search_export_constraint(&SearchExportQuery {
            word_type: Some(1),
            ..Default::default()
        }));
        assert!(has_search_export_constraint(&SearchExportQuery {
            collection_ids: Some("12,15".into()),
            ..Default::default()
        }));
        assert!(!has_search_export_constraint(&SearchExportQuery {
            languages: Some("2".into()),
            ..Default::default()
        }));
        assert!(has_search_export_constraint(&SearchExportQuery {
            search_in_phrases: Some(false),
            ..Default::default()
        }));
    }

    #[test]
    fn export_format_from_query_accepts_aliases() {
        assert_eq!(
            ExportFormat::from_query(Some("tex")).unwrap(),
            ExportFormat::LaTeX
        );
        assert!(ExportFormat::from_query(Some("docx")).is_err());
        assert_eq!(
            format!("search-export.{}", ExportFormat::Json.file_extension()),
            "search-export.json"
        );
        assert_eq!(
            format!("search-export.{}", ExportFormat::Tsv.file_extension()),
            "search-export.zip"
        );
    }
}

#[cfg(test)]
mod positive_scores_integration {
    use super::*;
    use deadpool_postgres::{Config, Runtime};
    use std::collections::HashMap;
    use std::env;

    fn test_pool() -> Pool {
        dotenvy::dotenv().ok();
        let mut cfg = Config::new();
        cfg.host = Some(env::var("DB_HOST").unwrap_or_else(|_| "localhost".into()));
        cfg.port = Some(
            env::var("DB_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
        );
        cfg.user = Some(env::var("DB_USER").expect("DB_USER"));
        cfg.password = Some(env::var("DB_PASSWORD").expect("DB_PASSWORD"));
        cfg.dbname = Some(env::var("DB_NAME").expect("DB_NAME"));
        cfg.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
            .unwrap()
    }

    fn parse_entries(bytes: &[u8]) -> Vec<serde_json::Value> {
        serde_json::from_slice(bytes).unwrap()
    }

    #[tokio::test]
    #[ignore = "requires local Postgres with dictionary data"]
    async fn false_exports_nonpositive_definitions_even_when_word_has_positive() {
        let pool = test_pool();
        {
            let c = pool.get().await.unwrap();
            c.execute(
                "DELETE FROM cached_dictionary_exports
                 WHERE language_tag = 'de' AND source_language_tag = 'jbo' AND format = 'json'",
                &[],
            )
            .await
            .unwrap();
        }

        let true_opts = ExportOptions {
            format: Some("json".into()),
            positive_scores_only: Some(true),
            source_lang: Some("jbo".into()),
            collection_id: None,
        };
        let false_opts = ExportOptions {
            format: Some("json".into()),
            positive_scores_only: Some(false),
            source_lang: Some("jbo".into()),
            collection_id: None,
        };

        let (true_bytes, _, _) = export_dictionary(
            &pool,
            "de",
            ExportFormat::Json,
            &true_opts,
            None,
            1,
            "jbo",
            true,
        )
        .await
        .expect("true export");
        let true_entries = parse_entries(&true_bytes);
        assert!(
            true_entries
                .iter()
                .all(|e| e["score"].as_f64().unwrap_or(0.0) > 0.0),
            "positive-only export must not include nonpositive scores"
        );

        let (false_bytes, _, _) = export_dictionary(
            &pool,
            "de",
            ExportFormat::Json,
            &false_opts,
            None,
            1,
            "jbo",
            true,
        )
        .await
        .expect("false export");
        let false_entries = parse_entries(&false_bytes);
        let false_nonpos = false_entries
            .iter()
            .filter(|e| e["score"].as_f64().unwrap_or(1.0) <= 0.0)
            .count();
        assert!(
            false_nonpos > 0,
            "positive_scores_only=false must include nonpositive definitions"
        );
        assert!(
            false_entries.len() > true_entries.len(),
            "unfiltered export should be larger than positive-only"
        );

        // Root-cause regression: a word with both positive and nonpositive definitions
        // must contribute the nonpositive row(s) when the filter is disabled.
        let mut by_word: HashMap<String, Vec<f64>> = HashMap::new();
        for e in &false_entries {
            let word = e["word"].as_str().unwrap_or("").to_string();
            let score = e["score"].as_f64().unwrap_or(0.0);
            by_word.entry(word).or_default().push(score);
        }
        let has_mixed_word = by_word
            .values()
            .any(|scores| scores.iter().any(|s| *s > 0.0) && scores.iter().any(|s| *s <= 0.0));
        assert!(
            has_mixed_word,
            "expected at least one word to export both positive and nonpositive definitions when filter is off"
        );
    }
}
