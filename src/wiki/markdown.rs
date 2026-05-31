//! Best-effort wikitext -> Markdown renderer using `parse-wiki-text-2`.
//!
//! Two outputs are produced from a single AST walk:
//! - `markdown`: GitHub-flavored Markdown intended for in-browser rendering.
//! - `plain`:    formatting-stripped text used for `ILIKE` search and previews.
//!
//! Templates and parser functions are intentionally rendered as italicised
//! `{{name|args}}` text rather than expanded — full template expansion would
//! require running MediaWiki itself.

use parse_wiki_text_2::{
    Configuration, DefinitionListItem, DefinitionListItemType, ListItem, Node, Parameter,
    TableCaption, TableCell, TableCellType, TableRow,
};

/// Convert MediaWiki source to (markdown, plain_text).
pub fn wikitext_to_markdown(input: &str) -> (String, String) {
    let cfg = Configuration::default();
    let output = match cfg.parse(input) {
        Ok(o) => o,
        Err(_) => {
            // Parse failure: degrade gracefully — emit raw text.
            return (input.to_string(), input.to_string());
        }
    };
    let mut md = String::new();
    let mut plain = String::new();
    render_nodes(&output.nodes, &mut md, &mut plain, 0);
    // Collapse 3+ newlines and trailing whitespace.
    let md = collapse_blank_lines(md.trim_end());
    let plain = collapse_whitespace(plain.trim());
    (md, plain)
}

/// Build a relative MediaWiki page target for a `[[Target]]` link target.
pub fn wiki_target_url(target: &str) -> String {
    let t = target.trim().replace(' ', "_");
    format!("/papri/{}", urlencoding::encode(&t))
}

/// Rewrite imported MediaWiki-relative page links for the Lensisku article route.
pub fn rewrite_wiki_links_for_lensisku(markdown: &str) -> String {
    markdown.replace("](/papri/", "](/wiki/")
}

fn render_nodes(nodes: &[Node<'_>], md: &mut String, plain: &mut String, depth: usize) {
    for node in nodes {
        render_node(node, md, plain, depth);
    }
}

fn render_node(node: &Node<'_>, md: &mut String, plain: &mut String, depth: usize) {
    match node {
        Node::Text { value, .. } => {
            md.push_str(value);
            plain.push_str(value);
        }
        Node::CharacterEntity { character, .. } => {
            md.push(*character);
            plain.push(*character);
        }
        Node::Bold { .. } => md.push_str("**"),
        Node::Italic { .. } => md.push('*'),
        Node::BoldItalic { .. } => md.push_str("***"),
        Node::ParagraphBreak { .. } => {
            md.push_str("\n\n");
            plain.push('\n');
        }
        Node::HorizontalDivider { .. } => {
            md.push_str("\n\n---\n\n");
            plain.push('\n');
        }
        Node::Heading { level, nodes, .. } => {
            md.push('\n');
            for _ in 0..(*level).min(6) {
                md.push('#');
            }
            md.push(' ');
            let mut inner_plain = String::new();
            render_nodes(nodes, md, &mut inner_plain, depth);
            md.push_str("\n\n");
            plain.push_str(&inner_plain);
            plain.push('\n');
        }
        Node::Link { target, text, .. } => {
            let label = if text.is_empty() {
                target.to_string()
            } else {
                let mut buf = String::new();
                let mut sink = String::new();
                render_nodes(text, &mut buf, &mut sink, depth);
                if buf.trim().is_empty() {
                    target.to_string()
                } else {
                    buf
                }
            };
            md.push_str(&format!("[{}]({})", label.replace(']', "\\]"), wiki_target_url(target)));
            plain.push_str(&label);
        }
        Node::ExternalLink { nodes, start, end } => {
            // ExternalLink content includes the URL + optional label after first space.
            let mut buf = String::new();
            let mut sink = String::new();
            render_nodes(nodes, &mut buf, &mut sink, depth);
            let trimmed = buf.trim();
            let (url, label) = match trimmed.split_once(char::is_whitespace) {
                Some((u, l)) => (u.trim().to_string(), l.trim().to_string()),
                None => (trimmed.to_string(), String::new()),
            };
            let _ = (start, end);
            if url.is_empty() {
                return;
            }
            let display = if label.is_empty() { url.clone() } else { label };
            md.push_str(&format!("[{}]({})", display, url));
            plain.push_str(&display);
        }
        Node::Image { target, text, .. } => {
            let mut alt = String::new();
            let mut sink = String::new();
            render_nodes(text, &mut alt, &mut sink, depth);
            md.push_str(&format!(
                "![{}]({})",
                alt.trim(),
                wiki_target_url(target)
            ));
        }
        Node::Category { target, .. } => {
            md.push_str(&format!(
                "\n_Category: [{}]({})_\n",
                target,
                wiki_target_url(&format!("Category:{target}"))
            ));
        }
        Node::Redirect { target, .. } => {
            md.push_str(&format!(
                "_Redirect to [{}]({})._\n",
                target,
                wiki_target_url(target)
            ));
            plain.push_str(&format!("Redirect to {target}."));
        }
        Node::UnorderedList { items, .. } => {
            md.push('\n');
            render_list_items(items, md, plain, depth, false);
            md.push('\n');
        }
        Node::OrderedList { items, .. } => {
            md.push('\n');
            render_list_items(items, md, plain, depth, true);
            md.push('\n');
        }
        Node::DefinitionList { items, .. } => {
            md.push('\n');
            render_def_list_items(items, md, plain, depth);
            md.push('\n');
        }
        Node::Preformatted { nodes, .. } => {
            md.push_str("\n```\n");
            let mut inner_plain = String::new();
            render_nodes(nodes, md, &mut inner_plain, depth);
            md.push_str("\n```\n");
            plain.push_str(&inner_plain);
            plain.push('\n');
        }
        Node::Tag { name, nodes, .. } => {
            let n = name.as_ref();
            match n {
                "code" | "tt" | "pre" => {
                    md.push_str("`");
                    let mut inner_plain = String::new();
                    render_nodes(nodes, md, &mut inner_plain, depth);
                    md.push_str("`");
                    plain.push_str(&inner_plain);
                }
                "nowiki" => {
                    let mut inner_plain = String::new();
                    render_nodes(nodes, md, &mut inner_plain, depth);
                    plain.push_str(&inner_plain);
                }
                "br" => md.push_str("  \n"),
                "ref" => {
                    // Footnotes: ignore content, just drop a marker.
                    md.push_str("[^ref]");
                }
                _ => {
                    let mut inner_plain = String::new();
                    render_nodes(nodes, md, &mut inner_plain, depth);
                    plain.push_str(&inner_plain);
                }
            }
        }
        Node::StartTag { name, .. } => {
            if name.as_ref() == "br" {
                md.push_str("  \n");
            }
        }
        Node::EndTag { .. } | Node::Comment { .. } | Node::MagicWord { .. } => {}
        Node::Table {
            captions,
            rows,
            ..
        } => {
            render_table(captions, rows, md, plain, depth);
        }
        Node::Template {
            name, parameters, ..
        } => {
            render_template(name, parameters, md, plain, depth);
        }
        Node::Parameter { name, default, .. } => {
            let mut buf = String::new();
            let mut sink = String::new();
            render_nodes(name, &mut buf, &mut sink, depth);
            md.push_str(&format!("{{{{{{{}}}}}}}", buf));
            if let Some(def) = default {
                let mut dbuf = String::new();
                render_nodes(def, &mut dbuf, &mut sink, depth);
                let _ = dbuf;
            }
        }
    }
}

fn render_list_items(
    items: &[ListItem<'_>],
    md: &mut String,
    plain: &mut String,
    depth: usize,
    ordered: bool,
) {
    for (i, item) in items.iter().enumerate() {
        for _ in 0..depth {
            md.push_str("  ");
        }
        if ordered {
            md.push_str(&format!("{}. ", i + 1));
        } else {
            md.push_str("- ");
        }
        let mut item_md = String::new();
        let mut item_plain = String::new();
        render_nodes(&item.nodes, &mut item_md, &mut item_plain, depth + 1);
        // Indent multi-line item bodies for nested content.
        let trimmed = item_md.trim_end_matches('\n');
        md.push_str(trimmed);
        md.push('\n');
        plain.push_str(item_plain.trim());
        plain.push('\n');
    }
}

fn render_def_list_items(
    items: &[DefinitionListItem<'_>],
    md: &mut String,
    plain: &mut String,
    depth: usize,
) {
    for item in items {
        let prefix = match item.type_ {
            DefinitionListItemType::Term => "**",
            DefinitionListItemType::Details => ": ",
        };
        let mut buf = String::new();
        let mut sink = String::new();
        render_nodes(&item.nodes, &mut buf, &mut sink, depth);
        match item.type_ {
            DefinitionListItemType::Term => {
                md.push_str(&format!("{prefix}{}{prefix}\n", buf.trim()));
            }
            DefinitionListItemType::Details => {
                md.push_str(&format!("{prefix}{}\n", buf.trim()));
            }
        }
        plain.push_str(buf.trim());
        plain.push('\n');
    }
}

fn render_template(
    name_nodes: &[Node<'_>],
    params: &[Parameter<'_>],
    md: &mut String,
    plain: &mut String,
    _depth: usize,
) {
    let mut name = String::new();
    let mut sink = String::new();
    render_nodes(name_nodes, &mut name, &mut sink, 0);
    let name = name.trim();
    if name.is_empty() {
        return;
    }

    // Extract template arguments
    let mut args: Vec<String> = Vec::new();
    for p in params {
        let mut k = String::new();
        let mut v = String::new();
        let mut sink2 = String::new();
        if let Some(n) = &p.name {
            render_nodes(n, &mut k, &mut sink2, 0);
        }
        render_nodes(&p.value, &mut v, &mut sink2, 0);
        let k = k.trim();
        let v = v.trim();
        if k.is_empty() {
            args.push(v.to_string());
        } else {
            args.push(format!("{k}={v}"));
        }
    }

    // Process template based on type
    let name_lower = name.trim().to_lowercase();
    let name_lower = name_lower.trim();
    match name_lower {
        // ── Lojban word templates ────────────────────────────────────────────
        // Render as bold (the word itself)
        "vla" | "jbo" | "c" | "vlapoi" | "cmevla" | "selmaho" | "selma'o" => {
            let text = args.first().map(|s| s.as_str()).unwrap_or(name);
            md.push_str(&format!("**{}**", text));
            plain.push_str(text);
        }

        // Cmavo / grammar word classes — bold
        "cmavo" | "grammar" | "nunjikca" | "jikca" | "gln"
        | "cc" | "ir" | "mv" | "vj" | "ep" | "lg" | "leng" => {
            let text = args.first().map(|s| s.as_str()).unwrap_or(name);
            md.push_str(&format!("**{}**", text));
            plain.push_str(text);
        }

        // Selma'o (word class) — italic uppercase
        "s" => {
            let text = args.first().map(|s| s.as_str()).unwrap_or("");
            if !text.is_empty() {
                md.push_str(&format!("*{}*", text.to_uppercase()));
                plain.push_str(&text.to_uppercase());
            }
        }

        // English gloss — quoted
        "gl" => {
            let text = args.first().map(|s| s.as_str()).unwrap_or("");
            if !text.is_empty() {
                md.push_str(&format!("\"{}\"", text));
                plain.push_str(text);
            }
        }

        // Variables / placeholders — italic
        "ma" | "lerfu" | "mu" | "mo" | "l" => {
            let text = args.first().map(|s| s.as_str()).unwrap_or(name);
            md.push_str(&format!("_{}_", text));
            plain.push_str(text);
        }

        // ── Link shortcuts ───────────────────────────────────────────────────
        "ls" | "lch" | "raf" | "ms" | "llg" => {
            if let Some(target) = args.first() {
                md.push_str(&format!("[{}]({})", target, wiki_target_url(target)));
                plain.push_str(target);
            }
        }

        // Jbovlaste link
        "jvs" => {
            let word = args.first().map(|s| s.as_str()).unwrap_or("");
            if !word.is_empty() {
                md.push_str(&format!(
                    "[{}](https://jbovlaste.lojban.org/dict/{})",
                    word,
                    urlencoding::encode(word)
                ));
                plain.push_str(word);
            }
        }

        // Wikipedia link
        "wikipedia" => {
            if let Some(article) = args.first() {
                md.push_str(&format!(
                    "[Wikipedia: {}](https://en.wikipedia.org/wiki/{})",
                    article,
                    urlencoding::encode(article)
                ));
                plain.push_str(article);
            }
        }

        // Lojban StackExchange shortcut
        "lojban stackexchange" => {
            md.push_str("[Lojban StackExchange](https://linguistics.stackexchange.com/questions/tagged/lojban)");
            plain.push_str("Lojban StackExchange");
        }

        // ── Inline formatting pass-throughs ─────────────────────────────────
        // Math expressions — code span
        "math" => {
            let text = args.first().map(|s| s.as_str()).unwrap_or("");
            if !text.is_empty() {
                md.push_str(&format!("`{}`", text));
                plain.push_str(text);
            }
        }

        // IPA / phonetics — code span
        "ipa" | "x-sampa" => {
            let text = args.first().map(|s| s.as_str()).unwrap_or("");
            if !text.is_empty() {
                md.push_str(&format!("`{}`", text));
                plain.push_str(text);
            }
        }

        // Inline text-formatting helpers — pass content through
        "color" | "colour" | "small" | "sc" | "overline" | "nobr" | "uc" | "lc"
        | "lang-en" | "lang-de" | "lang" => {
            // For lang: first arg is language code, second is text.
            // For others: first arg is the text (or second if first is a colour spec).
            let text = if name_lower == "color" || name_lower == "colour" {
                // {{color|#hex|text}}
                args.get(1).or_else(|| args.first()).map(|s| s.as_str()).unwrap_or("")
            } else if name_lower == "lang" {
                args.get(1).map(|s| s.as_str()).unwrap_or("")
            } else {
                args.first().map(|s| s.as_str()).unwrap_or("")
            };
            if !text.is_empty() {
                md.push_str(text);
                plain.push_str(text);
            }
        }

        // Superscript / keyboard key — backtick span
        "keypress" | "key" => {
            let text = args.first().map(|s| s.as_str()).unwrap_or("");
            if !text.is_empty() {
                md.push_str(&format!("`{}`", text));
                plain.push_str(text);
            }
        }

        // Anchor — strip (invisible marker)
        "anchor" => {}

        // Nihongo / Japanese text — render first arg (the text)
        "nihongo2" => {
            let text = args.first().map(|s| s.as_str()).unwrap_or("");
            if !text.is_empty() {
                md.push_str(text);
                plain.push_str(text);
            }
        }

        // ── Citations ────────────────────────────────────────────────────────
        // Inline citation markers — drop silently (no useful text)
        "irci" | "reltonga" | "ref" | "efn" => {}

        // Cite templates — render as bracketed reference text
        "cite" | "cite book" | "cite web" | "cite journal" | "cite news" => {
            // Extract title= and url= named params if present
            let mut title = String::new();
            let mut url = String::new();
            let mut author = String::new();
            for arg in &args {
                if let Some(v) = arg.strip_prefix("title=") {
                    title = v.to_string();
                } else if let Some(v) = arg.strip_prefix("url=") {
                    url = v.to_string();
                } else if let Some(v) = arg.strip_prefix("author=") {
                    author = v.to_string();
                } else if let Some(v) = arg.strip_prefix("last=") {
                    if author.is_empty() { author = v.to_string(); }
                }
            }
            if title.is_empty() && !args.is_empty() {
                title = args[0].clone();
            }
            if !title.is_empty() {
                let cite_text = if !author.is_empty() {
                    format!("{}: {}", author, title)
                } else {
                    title.clone()
                };
                if !url.is_empty() {
                    md.push_str(&format!("[{}]({})", cite_text, url));
                } else {
                    md.push_str(&format!("_{}_", cite_text));
                }
                plain.push_str(&cite_text);
            }
        }

        // Reflist / footnote containers — strip
        "reflist" | "refbegin" | "refend" | "refs" => {}

        // ── Block / section templates ────────────────────────────────────────
        // See also
        "see also" | "see_also" => {
            // Positional args are page titles; named args (e.g. label1=) are ignored.
            let links: Vec<&str> = args.iter()
                .filter(|a| !a.contains('='))
                .map(|s| s.as_str())
                .collect();
            if !links.is_empty() {
                md.push_str("\n\n**See also:** ");
                for (i, link) in links.iter().enumerate() {
                    if i > 0 { md.push_str(", "); }
                    md.push_str(&format!("[{}]({})", link, wiki_target_url(link)));
                    plain.push_str(link);
                    if i + 1 < links.len() { plain.push_str(", "); }
                }
                md.push_str("\n\n");
            }
        }

        // Hatnote / disambiguation / notice boxes — strip (navigation cruft)
        "hatnote" | "main" | "ombox" | "ambox" | "note" | "side box"
        | "about" | "for" | "redirect" | "distinguish" => {
            // Could render as blockquote, but they're mostly navigation noise.
        }

        // Quotation blocks — blockquote
        "quotation" | "quote box" | "rquote" => {
            let text = args.iter().find(|a| !a.contains('=')).map(|s| s.as_str()).unwrap_or("");
            if !text.is_empty() {
                md.push_str("\n> ");
                md.push_str(text);
                md.push('\n');
                plain.push_str(text);
            }
        }

        // Example blocks — blockquote
        "example" => {
            if !args.is_empty() {
                md.push_str("\n> ");
                md.push_str(&args.join(" | "));
                md.push('\n');
                plain.push_str(&args.join(" "));
            }
        }

        // mupli (Lojban example) — blockquote
        "mupli" => {
            if !args.is_empty() {
                md.push_str("\n> Example: ");
                md.push_str(&args.join(" | "));
                md.push('\n');
                plain.push_str(&args.join(" "));
            }
        }

        // Ordered list
        "ordered list" => {
            if !args.is_empty() {
                md.push('\n');
                for (i, item) in args.iter().enumerate() {
                    md.push_str(&format!("{}. {}\n", i + 1, item));
                    plain.push_str(item);
                    plain.push(' ');
                }
            }
        }

        // lm (Lojban-Math-gloss triple) — GFM table row
        "lm" => {
            if args.len() >= 3 {
                md.push_str("\n| ");
                md.push_str(&args[0]);
                md.push_str(" | ");
                md.push_str(&args[1]);
                md.push_str(" | ");
                md.push_str(&args[2]);
                md.push_str(" |\n");
                plain.push_str(&format!("{} {} {}", args[0], args[1], args[2]));
            } else if !args.is_empty() {
                md.push_str(&args.join(" "));
                plain.push_str(&args.join(" "));
            }
        }

        // ── Audio / media ────────────────────────────────────────────────────
        // Strip audio embeds (can't play in markdown)
        "wave" | "audio" | "listen" => {
            if let Some(filename) = args.first() {
                // Render as a label so readers know something is there
                let label = filename.trim_end_matches(|c: char| c == ',' || c.is_whitespace());
                if !label.is_empty() {
                    md.push_str(&format!("🔊 _{}_", label));
                    plain.push_str(label);
                }
            }
        }

        // ── Metadata / index / structure — strip silently ────────────────────
        "ind" | "dsp" | "judri" | "lex" | "ssp" | "startchapter" | "bookcat"
        | "bpfk section box open" | "bpfk section box close" | "bpfk section from tiki"
        | "bpfk section poll" | "bpfk"
        | "se inspekte/en" | "se_inspekte/en" | "se_inspekte/jbo" | "se_inspekte/ru"
        | "jbocre/en" | "jbocre/ja" | "jbocre/fr" | "jbocre/jbo" | "jbocre"
        | "nalylojbo/en" | "comment" | "navigation" | "newpage"
        | "notci" | "secmavo" | "personal" | "csp" | "lfk"
        | "•" | "=" | "clear" | "false" | "robox" | "robox/close"
        | "colwidth" | "div col" | "div col end" | "extraclasses"
        | "int:tadni-url" | "webchat url qwebirc" | "webchat url kiwi" | "webchat url"
        | "fullpagename" | "fullpagenamee" | "pagename" | "basepagename"
        | "basepagenamee" | "subpagename" | "namespace" | "ns"
        | "pageid" | "pagesize" | "numberofarticles" | "currentyear"
        | "currentday" | "currentmonth" | "currentversion" | "server"
        | "fullurl" | "localurl" | "canonicalurl" | "filepath" | "urlencode"
        | "formatnum" | "lcfirst" | "ucfirst" | "subst" | "defaultsort"
        | "displaytitle" | "int" | "subjectspace" | "subjectpagename"
        | "doc" | "template" | "template page" | "subject page"
        | "refimprove section" | "citation needed" | "by whom?" | "clarify" | "sic"
        | "unsolved" | "status" | "featured article" | "did you know"
        | "in the news" | "print version" | "print version notice"
        | "latest news" | "categorylist" | "languages" | "gfdl"
        | "associated wikimedia" | "wiktionarycat" | "commons"
        | "sbcnewline" | "sbcnewline2"
        | "волны ложбана" | "onde" | "wave chunks"
        | "guglgirzu lojban-soudan nuzyfle"
        | "guglgirzu ponjo_lojbo_citno_girzu nuzyfle"
        | "remoisocial" | "ennuzba" | "nuzba/jbo" | "nalcatni"
        | "infobox person" | "grammatical moods" | "lexical categories"
        | "val" | "vajni" | "mun"
        | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
        | "-" | "!" | "a" | "t" | "g" | "p/s" => {}

        // All sbc* (Spanish/Portuguese audio-course) templates — strip
        t if t.starts_with("sbc") => {}

        // Transclusion of sub-pages (e.g. {{:ELG. Introduction}}) — strip
        t if t.starts_with(':') => {}

        // MediaWiki i18n magic words — strip
        t if t.starts_with("int:") => {}

        // Unknown templates — drop silently (no raw {{...}} noise in output)
        _ => {}
    }
}

fn render_table(
    captions: &[TableCaption<'_>],
    rows: &[TableRow<'_>],
    md: &mut String,
    plain: &mut String,
    _depth: usize,
) {
    md.push('\n');
    for cap in captions {
        let mut buf = String::new();
        let mut sink = String::new();
        render_nodes(&cap.content, &mut buf, &mut sink, 0);
        md.push_str(&format!("**{}**\n\n", buf.trim()));
        plain.push_str(buf.trim());
        plain.push('\n');
    }
    if rows.is_empty() {
        return;
    }
    // Determine column count from widest row.
    let cols = rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
    if cols == 0 {
        return;
    }
    let header_row = pick_header_row(rows);
    if let Some(idx) = header_row {
        write_table_row(&rows[idx].cells, md, plain);
        md.push('|');
        for _ in 0..cols {
            md.push_str(" --- |");
        }
        md.push('\n');
        for (i, row) in rows.iter().enumerate() {
            if i == idx {
                continue;
            }
            write_table_row(&row.cells, md, plain);
        }
    } else {
        // No explicit header: synthesize a blank one for valid GFM.
        md.push('|');
        for _ in 0..cols {
            md.push_str("   |");
        }
        md.push('\n');
        md.push('|');
        for _ in 0..cols {
            md.push_str(" --- |");
        }
        md.push('\n');
        for row in rows {
            write_table_row(&row.cells, md, plain);
        }
    }
    md.push('\n');
}

fn pick_header_row(rows: &[TableRow<'_>]) -> Option<usize> {
    rows.iter()
        .position(|r| r.cells.iter().all(|c| matches!(c.type_, TableCellType::Heading)))
}

fn write_table_row(cells: &[TableCell<'_>], md: &mut String, plain: &mut String) {
    md.push('|');
    for cell in cells {
        let mut buf = String::new();
        let mut sink = String::new();
        render_nodes(&cell.content, &mut buf, &mut sink, 0);
        let s = buf.trim().replace('\n', " ").replace('|', "\\|");
        md.push_str(&format!(" {} |", s));
        plain.push_str(s.as_str());
        plain.push(' ');
    }
    md.push('\n');
    plain.push('\n');
}

fn collapse_blank_lines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut blank = 0;
    for line in s.split('\n') {
        if line.trim().is_empty() {
            blank += 1;
            if blank <= 2 {
                out.push('\n');
            }
        } else {
            blank = 0;
            out.push_str(line);
            out.push('\n');
        }
    }
    out.trim_end().to_string()
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_ws = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_ws {
                out.push(' ');
            }
            prev_ws = true;
        } else {
            out.push(c);
            prev_ws = false;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_and_bold() {
        let (md, plain) = wikitext_to_markdown("== Title ==\n\nHello '''world'''.");
        assert!(md.contains("## Title"), "md={md}");
        assert!(md.contains("**world**"), "md={md}");
        assert!(plain.contains("Title"));
        assert!(plain.contains("world"));
    }

    #[test]
    fn internal_link_with_label() {
        let (md, _) = wikitext_to_markdown("See [[Lojban|the language]] for details.");
        assert!(md.contains("[the language](/papri/Lojban)"), "md={md}");
    }

    #[test]
    fn rewrites_internal_links_for_lensisku_route() {
        let markdown = "See [the language](/papri/Lojban) and [external](https://example.com).";
        let rewritten = rewrite_wiki_links_for_lensisku(markdown);
        assert!(rewritten.contains("[the language](/wiki/Lojban)"), "rewritten={rewritten}");
        assert!(rewritten.contains("[external](https://example.com)"), "rewritten={rewritten}");
    }

    #[test]
    fn unordered_list() {
        let (md, _) = wikitext_to_markdown("* one\n* two\n* three");
        assert!(md.contains("- one"), "md={md}");
        assert!(md.contains("- two"));
    }

    #[test]
    fn unknown_template_dropped_silently() {
        let (md, _) = wikitext_to_markdown("before {{stub|reason=test}} after");
        assert!(!md.contains("{{"), "md should not contain raw template syntax: {md}");
        assert!(md.contains("before") && md.contains("after"), "surrounding text preserved: {md}");
    }

    #[test]
    fn lojban_word_template_renders_bold() {
        let (md, plain) = wikitext_to_markdown("The word {{vla|broda}} is used here.");
        assert!(md.contains("**broda**"), "md={md}");
        assert!(plain.contains("broda"), "plain={plain}");
    }

    #[test]
    fn gloss_template_renders_quoted() {
        let (md, plain) = wikitext_to_markdown("{{gl|I love you}}");
        assert!(md.contains("\"I love you\""), "md={md}");
        assert!(plain.contains("I love you"), "plain={plain}");
    }

    #[test]
    fn metadata_templates_stripped() {
        let (md, _plain) = wikitext_to_markdown("Text {{ind|index|entry}} more {{dsp|id}} text.");
        assert!(!md.contains("ind"), "md={md}");
        assert!(!md.contains("dsp"), "md={md}");
        assert!(md.contains("Text") && md.contains("more") && md.contains("text"), "md={md}");
    }

    #[test]
    fn math_template_renders_code() {
        let (md, _) = wikitext_to_markdown("{{math|3x + 2y}}");
        assert!(md.contains("`3x + 2y`"), "md={md}");
    }

    #[test]
    fn jbovlaste_link_template() {
        let (md, plain) = wikitext_to_markdown("See {{jvs|broda}} for details.");
        assert!(md.contains("[broda](https://jbovlaste.lojban.org/dict/broda)"), "md={md}");
        assert!(plain.contains("broda"), "plain={plain}");
    }

    #[test]
    fn variable_template_renders_italic() {
        let (md, _) = wikitext_to_markdown("{{ma|x1}} is a variable");
        assert!(md.contains("_x1_"), "md={md}");
    }

    #[test]
    fn selmaho_template_renders_italic_caps() {
        let (md, plain) = wikitext_to_markdown("{{s|PA}} is a number");
        assert!(md.contains("*PA*"), "md={md}");
        assert!(plain.contains("PA"), "plain={plain}");
    }

    #[test]
    fn see_also_template_renders_links() {
        let (md, _) = wikitext_to_markdown("{{See also|Lojban orthographies}}");
        assert!(md.contains("**See also:**"), "md={md}");
        assert!(md.contains("[Lojban orthographies](/papri/Lojban_orthographies)"), "md={md}");
    }

    #[test]
    fn quotation_template_renders_blockquote() {
        let (md, _) = wikitext_to_markdown("{{Quotation|This is a quote}}");
        assert!(md.contains("> This is a quote"), "md={md}");
    }

    #[test]
    fn lm_template_renders_table_row() {
        let (md, _) = wikitext_to_markdown("{{lm|mi prami do|I love you|gloss}}");
        assert!(md.contains("| mi prami do | I love you | gloss |"), "md={md}");
    }

    #[test]
    fn navigation_templates_stripped() {
        let (md, _) = wikitext_to_markdown("Text {{Navigation}} more {{Newpage}} text");
        assert!(!md.contains("Navigation"), "md={md}");
        assert!(!md.contains("Newpage"), "md={md}");
        assert!(md.contains("Text") && md.contains("more") && md.contains("text"), "md={md}");
    }

    #[test]
    fn link_shortcut_templates() {
        let (md, plain) = wikitext_to_markdown("See {{ls|Page Name}} for details");
        assert!(md.contains("[Page Name](/papri/Page_Name)"), "md={md}");
        assert!(plain.contains("Page Name"), "plain={plain}");
    }

    #[test]
    fn redirect_recognised() {
        let (md, plain) = wikitext_to_markdown("#REDIRECT [[Other]]");
        assert!(md.to_lowercase().contains("redirect"), "md={md}");
        assert!(plain.to_lowercase().contains("redirect"));
    }
}
