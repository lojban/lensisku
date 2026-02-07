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
use std::process::Command;
use tempfile::tempdir;
use xml::writer::{EventWriter, XmlEvent};
use zip::write::{FileOptions, ZipWriter};

use super::models::CachedExport;
use super::models::CollectionExportItem;
use super::models::DictionaryEntry;
use super::models::NaturalEntry;
use super::models::User;
use super::models::ValsiRow;
use super::models::{ExportFormat, ExportOptions};
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
    let mut command = std::process::Command::new("xelatex");
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

    // Run xelatex and capture output
    debug!("Executing command: {:?}", command);
    let output = match command.output() {
        Ok(out) => out,
        Err(e) => {
            error!("Failed to execute xelatex: {}", e);
            return Err(Box::new(e));
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

    transaction.commit().await?;
    export_dictionary(pool, lang, format, options, options.collection_id).await
}

pub async fn export_dictionary(
    pool: &Pool,
    lang: &str,
    format: ExportFormat,
    options: &ExportOptions,
    collection_id: Option<i32>,
) -> Result<(Vec<u8>, String, String), Box<dyn std::error::Error + Send + Sync>> {
    // For collection exports, bypass cache
    if collection_id.is_some() {
        return generate_export(pool, lang, format, options, collection_id).await;
    }

    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Try to get from cache first using transaction
    if let Some(row) = transaction
        .query_opt(
            "SELECT content, content_type, filename
             FROM cached_dictionary_exports
             WHERE language_tag = $1 AND format = $2
             AND created_at > NOW() - INTERVAL '4 days'",
            &[&lang, &format.to_string()],
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

    // If not in cache, generate new export
    generate_export(pool, lang, format, options, collection_id).await
}

fn zip_tsv_content(
    tsv_content: &str,
    filename: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut zip_buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut zip_buffer));
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file(filename, options)?;
        zip.write_all(tsv_content.as_bytes())?;
        zip.finish()?;
    }
    Ok(zip_buffer)
}

async fn generate_export(
    pool: &Pool,
    lang: &str,
    format: ExportFormat,
    options: &ExportOptions,
    collection_id: Option<i32>,
) -> Result<(Vec<u8>, String, String), Box<dyn std::error::Error + Send + Sync>> {
    let filename = match collection_id {
        Some(id) => format!("collection-{}-{}.{}", id, lang, format.file_extension()),
        None => format!("dictionary-{}.{}", lang, format.file_extension()),
    };
    let content_type = format.content_type().to_string();

    let mut client = pool.get().await?;
    let mut transaction = client.transaction().await?;

    let content = match format {
        ExportFormat::Pdf => {
            let latex = generate_latex(&mut transaction, lang, collection_id).await?;
            transaction.commit().await?;
            generate_pdf(&latex).await?
        }
        ExportFormat::LaTeX => {
            let latex = generate_latex(&mut transaction, lang, collection_id).await?;
            transaction.commit().await?;
            latex.into_bytes()
        }
        ExportFormat::Xml => {
            let xml = generate_xml(&mut transaction, lang, options, collection_id).await?;
            transaction.commit().await?;
            xml.into_bytes()
        }
        ExportFormat::Json => {
            let json = generate_json(&mut transaction, lang, options, collection_id).await?;
            transaction.commit().await?;
            json.into_bytes()
        }
        ExportFormat::Tsv => {
            let tsv = generate_tsv(&mut transaction, lang, options, collection_id).await?;
            transaction.commit().await?;
            // Determine the TSV filename (without .zip extension)
            let tsv_filename = match collection_id {
                Some(id) => format!("collection-{}-{}.tsv", id, lang),
                None => format!("dictionary-{}.tsv", lang),
            };
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
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_mime,
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_mime
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
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_mime,
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_mime
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

    let score_condition = if options.positive_scores_only.unwrap_or(false) {
        "AND (SELECT COALESCE(SUM(value), 0) FROM definitionvotes WHERE definitionid = vbg.definitionid) > 0"
    } else {
        ""
    };

    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = vbg.definitionid")
        .unwrap_or("");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let query = format!(
        "SELECT v.word, vbg.definitionid, c.rafsi, c.selmaho, c.definition,
                c.notes, d.jargon, t.descriptor,
                (SELECT COALESCE(SUM(value), 0) FROM definitionvotes WHERE definitionid = vbg.definitionid) as score
         FROM valsibestguesses vbg
         JOIN valsi v ON v.valsiid = vbg.valsiid
         JOIN convenientdefinitions c ON c.definitionid = vbg.definitionid
         JOIN definitions d ON d.definitionid = vbg.definitionid
         JOIN valsitypes t ON t.typeid = v.typeid
         {}
         WHERE vbg.langid = $1 {} {}
         AND v.source_langid = 1
         ORDER BY lower(v.word)",
        collection_join, score_condition, collection_condition
    );

    let langid = lang_info.get::<_, i32>("langid");
    let rows = transaction.query(&query, &[&langid]).await?;

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
            &row.get::<_, f32>("score").to_string(),
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
        }
    }
}

async fn generate_latex(
    transaction: &mut Transaction<'_>,
    lang: &str,
    collection_id: Option<i32>,
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
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_mime,
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_mime
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

    let content = if collection_id.is_some() {
        // Generate LaTeX specifically for a collection
        generate_collection_latex(transaction, lang, collection_id.unwrap()).await?
    } else {
        // Generate standard dictionary chapters
        generate_chapters(transaction, lang, &escaped_lang, None).await?
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
        )
        .await
    } else {
        generate_lojban_and_natural_chapters(
            transaction,
            lang_id,
            lang,
            escaped_lang,
            collection_id,
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
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let entries = generate_lojban_entries(transaction, lang_id, lang, collection_id).await?;
    Ok(format!("\\chapter{{{}}}{}", title, entries))
}

async fn generate_collection_latex(
    transaction: &mut Transaction<'_>,
    lang: &str, // lang tag needed for escape_tex logic
    collection_id: i32,
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

    format!(
        "\n\n{{\\sffamily\\bfseries {}}} \\enspace {} {}",
        escape_all(&front),
        format_definition(&back, lang),
        format_collection_note(&note.unwrap_or_default())
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

    let lojban_chapter =
        generate_lojban_chapter(transaction, lang_id, lang, &vlaste_from_jbo, collection_id)
            .await?;

    // Check if there are any natural language entries before generating that chapter
    let has_natural_entries = check_natural_entries(transaction, lang_id, collection_id).await?;

    if has_natural_entries {
        let natural_chapter = generate_natural_chapter(transaction, lang_id, collection_id).await?;
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
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = vbg.definitionid")
        .unwrap_or("");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let query = format!(
        "SELECT EXISTS (
            SELECT 1
            FROM valsibestguesses vbg
            JOIN valsi v ON v.valsiid = vbg.valsiid
            JOIN natlangwordbestguesses nlwbg ON nlwbg.definitionid = vbg.definitionid
            JOIN natlangwords nlw ON nlw.wordid = nlwbg.natlangwordid
            {}
            WHERE vbg.langid = $1 {}
            AND v.source_langid = 1
        )",
        collection_join, collection_condition
    );

    let row = transaction.query_one(&query, &[&lang_id]).await?;
    Ok(row.get(0))
}

async fn generate_lojban_entries(
    transaction: &mut Transaction<'_>,
    lang_id: i32,
    lang: &str,
    collection_id: Option<i32>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut entries = String::new();
    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = vbg.definitionid")
        .unwrap_or("");
    let collection_note_select = collection_id
        .map(|_| ", ci.notes as collection_note")
        .unwrap_or(", NULL as collection_note");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let where_clause = format!("WHERE vbg.langid = $1{}", collection_condition);

    let query = format!(
        "SELECT v.word, c.rafsi, c.selmaho, c.definition,
                c.notes, t.descriptor{}
         FROM valsibestguesses vbg
         JOIN valsi v ON v.valsiid = vbg.valsiid
         JOIN convenientdefinitions c ON c.definitionid = vbg.definitionid
         JOIN valsitypes t ON t.typeid = v.typeid
         {}
         {}
         AND v.source_langid = 1
         ORDER BY lower(v.word)",
        collection_note_select, collection_join, where_clause
    );

    let params: Vec<&(dyn postgres_types::ToSql + Sync)> = vec![&lang_id];

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
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = vbg.definitionid")
        .unwrap_or("");
    let collection_note_select = collection_id
        .map(|_| ", ci.notes as collection_note")
        .unwrap_or(", NULL as collection_note");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let query = format!(
        "SELECT nlw.word, nlw.meaning, v.word as valsi, nlwbg.place{}
         FROM valsibestguesses vbg
         JOIN valsi v ON v.valsiid = vbg.valsiid
         JOIN natlangwordbestguesses nlwbg ON nlwbg.definitionid = vbg.definitionid
         JOIN natlangwords nlw ON nlw.wordid = nlwbg.natlangwordid
         {}
         WHERE vbg.langid = $1 {}
         AND v.source_langid = 1
         AND EXISTS (
          SELECT 1
          FROM keywordmapping km
          WHERE km.natlangwordid = nlw.wordid and km.definitionid=nlwbg.definitionid
         )
         ORDER BY nlw.word",
        collection_note_select, collection_join, collection_condition
    );

    let rows = transaction.query(&query, &[&lang_id]).await?;
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
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_mime,
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_mime
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

    let score_condition = if options.positive_scores_only.unwrap_or(false) {
        "AND (SELECT COALESCE(SUM(value), 0) FROM definitionvotes WHERE definitionid = vbg.definitionid) > 0"
    } else {
        ""
    };

    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = vbg.definitionid")
        .unwrap_or("");
    let collection_note_select = collection_id
        .map(|_| ", ci.notes as collection_note")
        .unwrap_or(", NULL as collection_note");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let query = format!(
        "SELECT v.word, vbg.definitionid, c.rafsi, c.selmaho, c.definition,
                c.notes, d.jargon, t.descriptor{},
                (SELECT COALESCE(SUM(value), 0) FROM definitionvotes WHERE definitionid = vbg.definitionid) as score
         FROM valsibestguesses vbg
         JOIN valsi v ON v.valsiid = vbg.valsiid
         JOIN convenientdefinitions c ON c.definitionid = vbg.definitionid
         JOIN definitions d ON d.definitionid = vbg.definitionid
         JOIN valsitypes t ON t.typeid = v.typeid
         {}
         WHERE vbg.langid = $1 {} {}
         AND v.source_langid = 1
         ORDER BY lower(v.word)",
        collection_note_select, collection_join, score_condition, collection_condition
    );

    let langid = transaction
        .query_one("SELECT langid FROM languages WHERE tag = $1", &[&lang])
        .await?
        .get::<_, i32>("langid");

    let rows = transaction.query(&query, &[&langid]).await?;

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

    tsv.push_str("\n");

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
        let score: f32 = row.get("score");

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

        tsv.push_str("\n");
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
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_mime,
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_mime
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

    let score_condition = if options.positive_scores_only.unwrap_or(false) {
        "AND (SELECT COALESCE(SUM(value), 0) FROM definitionvotes WHERE definitionid = vbg.definitionid) > 0"
    } else {
        ""
    };

    let collection_join = collection_id
        .map(|_| "JOIN collection_items ci ON ci.definition_id = vbg.definitionid")
        .unwrap_or("");
    let collection_note_select = collection_id
        .map(|_| ", ci.notes as collection_note")
        .unwrap_or(", NULL as collection_note");
    let collection_condition = collection_id
        .map(|id| format!("AND ci.collection_id = {}", id))
        .unwrap_or_default();

    let query = format!(
        "SELECT v.word, vbg.definitionid, c.rafsi, c.selmaho, c.definition,
                c.notes, d.jargon, t.descriptor{},
                (SELECT COALESCE(SUM(value), 0) FROM definitionvotes WHERE definitionid = vbg.definitionid) as score
         FROM valsibestguesses vbg
         JOIN valsi v ON v.valsiid = vbg.valsiid
         JOIN convenientdefinitions c ON c.definitionid = vbg.definitionid
         JOIN definitions d ON d.definitionid = vbg.definitionid
         JOIN valsitypes t ON t.typeid = v.typeid
         {}
         WHERE vbg.langid = $1 {} {}
         AND v.source_langid = 1
         ORDER BY lower(v.word)",
        collection_note_select, collection_join, score_condition, collection_condition
    );

    let langid = transaction
        .query_one("SELECT langid FROM languages WHERE tag = $1", &[&lang])
        .await?
        .get::<_, i32>("langid");

    let rows = transaction.query(&query, &[&langid]).await?;

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
                jargon: row.get("jargon"),
                collection_note: row.get("collection_note"),
                score: row.get("score"),
                gloss_keywords: gloss_map.get(&definition_id).cloned(),
                place_keywords: place_map.get(&definition_id).cloned(),
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

    let rows = transaction
        .query(
            "SELECT cde.language_tag, l.realname AS language_realname, cde.format, cde.filename, cde.created_at
         FROM cached_dictionary_exports cde
         JOIN languages l ON cde.language_tag = l.tag
         ORDER BY l.realname",
            &[],
        )
        .await?;

    let exports = rows
        .into_iter()
        .map(|row| CachedExport {
            language_tag: row.get("language_tag"),
            language_realname: row.get("language_realname"),
            format: row.get("format"),
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
    format: &str,
) -> Result<(Vec<u8>, String, String), Box<dyn Error + Send + Sync>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let row = transaction
        .query_opt(
            "SELECT content, content_type, filename
         FROM cached_dictionary_exports
         WHERE language_tag = $1 AND format = $2",
            &[&language_tag, &format],
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
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let languages = transaction
        .query("SELECT tag FROM languages", &[])
        .await?
        .iter()
        .map(|row| row.get::<_, String>("tag"))
        .collect::<Vec<_>>();

    // Check existing cached exports
    let cached_exports_rows = transaction
        .query(
            "SELECT language_tag, format, MAX(created_at) as last_export FROM cached_dictionary_exports GROUP BY language_tag, format",
            &[],
        )
        .await?;

    let mut cached_exports = std::collections::HashMap::new();
    for row in cached_exports_rows {
        let lang_tag: String = row.get("language_tag");
        let format: String = row.get("format");
        let last_export: DateTime<Utc> = row.get("last_export");
        cached_exports.insert((lang_tag, format), last_export);
    }

    for lang in languages {
        for format in &[
            ExportFormat::Pdf,
            ExportFormat::LaTeX,
            ExportFormat::Xml,
            ExportFormat::Json,
            ExportFormat::Tsv,
        ] {
            let format_str = format.to_string();
            if let Some(last_export_time) = cached_exports.get(&(lang.clone(), format_str.clone()))
            {
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

            match export_dictionary(pool, &lang, *format, &Default::default(), None).await {
                Ok((content, content_type, filename)) => {
                    if let Err(e) = transaction
                        .execute(
                            "INSERT INTO cached_dictionary_exports
                             (language_tag, format, content, content_type, filename)
                             VALUES ($1, $2, $3, $4, $5)
                             ON CONFLICT (language_tag, format)
                             DO UPDATE SET
                                content = EXCLUDED.content,
                                content_type = EXCLUDED.content_type,
                                filename = EXCLUDED.filename,
                                created_at = CURRENT_TIMESTAMP",
                            &[
                                &lang,
                                &format.to_string(),
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

    transaction.commit().await?;
    Ok(())
}
