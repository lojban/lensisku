//! Best-effort wikitext -> Markdown renderer using `parse-wiki-text-2`.
//!
//! Two outputs are produced from a single AST walk:
//! - `markdown`: GitHub-flavored Markdown intended for in-browser rendering.
//!   MediaWiki tables are emitted as HTML (`<table>`) so captions, cell
//!   attributes, colspan/rowspan, and uneven header rows survive — GFM pipe
//!   tables cannot represent those.
//! - `plain`:    formatting-stripped text used for `ILIKE` search and previews.
//!
//! Templates that the AST does not expand are rewritten after parse: `{{jvs|word}}`
//! becomes a dictionary link, and other leftover `{{templates}}` are dropped.

use parse_wiki_text_2::{
    Configuration, DefinitionListItem, DefinitionListItemType, ListItem, Node, Parameter,
    TableCaption, TableCellType, TableRow,
};

/// Convert MediaWiki source to (markdown, plain_text).
pub fn wikitext_to_markdown(input: &str) -> (String, String) {
    let cfg = Configuration::default();
    let (md, plain) = match cfg.parse(input) {
        Ok(o) => {
            let mut md = String::new();
            let mut plain = String::new();
            render_nodes(&o.nodes, &mut md, &mut plain, 0);
            (md, plain)
        }
        Err(_) => {
            // Parse failure: degrade gracefully — emit raw text, then still
            // rewrite leftover `{{templates}}` so they are not shown as source.
            (input.to_string(), input.to_string())
        }
    };
    let md = collapse_blank_lines(
        normalize_markdown_code_spans(&rewrite_leftover_templates(&md, true)).trim_end(),
    );
    let plain = collapse_whitespace(rewrite_leftover_templates(&plain, false).trim());
    (md, plain)
}

/// Lensisku dictionary link for a jbovlaste / jvs word.
fn jvs_valsi_link(word: &str) -> String {
    let word = word.trim();
    if word.is_empty() {
        return String::new();
    }
    format!(
        "[{}](/valsi/{})",
        word.replace(']', "\\]"),
        urlencoding::encode(&word.replace(' ', "_"))
    )
}

/// Rewrite `{{jvs|...}}` (and drop other leftover templates) that the AST missed.
/// `as_markdown` true → jvs becomes a `/valsi/` link; false → just the word.
fn rewrite_leftover_templates(input: &str, as_markdown: bool) -> String {
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let chars: Vec<char> = input.chars().collect();
    while i < chars.len() {
        if chars[i] == '{' && i + 2 < chars.len() && chars[i + 1] == '{' && chars[i + 2] == '{' {
            // `{{{param}}}` — keep the first brace and continue, so we don't
            // treat parser-function parameters as templates.
            out.push('{');
            i += 1;
            continue;
        }
        if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            if let Some((consumed, inner)) = extract_balanced_template(&chars[i..]) {
                out.push_str(&render_leftover_template(&inner, as_markdown));
                i += consumed;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// `chars` starts at `{{`. Returns (chars consumed, inner without braces).
fn extract_balanced_template(chars: &[char]) -> Option<(usize, String)> {
    if chars.len() < 4 || chars[0] != '{' || chars[1] != '{' {
        return None;
    }
    let mut depth = 0i32;
    let mut i = 0;
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
            depth += 1;
            i += 2;
            continue;
        }
        if i + 1 < chars.len() && chars[i] == '}' && chars[i + 1] == '}' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                let inner: String = chars[2..i - 2].iter().collect();
                return Some((i, inner));
            }
            continue;
        }
        i += 1;
    }
    None
}

fn render_leftover_template(inner: &str, as_markdown: bool) -> String {
    let inner = inner.trim();
    if inner.is_empty() {
        return String::new();
    }
    let (name_raw, rest) = match inner.split_once('|') {
        Some((n, r)) => (n, Some(r)),
        None => (inner, None),
    };
    let name = normalize_template_name(name_raw);
    let first_arg = rest
        .and_then(|r| r.split('|').next())
        .map(strip_numbered_arg)
        .unwrap_or("");
    if name == "jvs" {
        if as_markdown {
            return jvs_valsi_link(first_arg);
        }
        return first_arg.to_string();
    }
    // Unknown leftover templates are dropped (no raw `{{...}}` in the article).
    String::new()
}

fn strip_numbered_arg(arg: &str) -> &str {
    let arg = arg.trim();
    arg.strip_prefix("1=")
        .or_else(|| arg.strip_prefix("1 ="))
        .unwrap_or(arg)
        .trim()
}

fn normalize_template_name(name: &str) -> String {
    name.trim()
        .trim_start_matches("Template:")
        .trim_start_matches("template:")
        .trim()
        .to_lowercase()
}

/// Build a relative MediaWiki page target for a `[[Target]]` link target.
pub fn wiki_target_url(target: &str) -> String {
    let t = target.trim().replace(' ', "_");
    format!("/papri/{}", urlencoding::encode(&t))
}

/// Rewrite imported MediaWiki-relative page links for the Lensisku article route.
pub fn rewrite_wiki_links_for_lensisku(markdown: &str) -> String {
    markdown
        .replace("](/papri/", "](/wiki/")
        // HTML tables emit <a href="/papri/...">; rewrite those too.
        .replace("href=\"/papri/", "href=\"/wiki/")
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
            md.push_str(&format!(
                "[{}]({})",
                label.replace(']', "\\]"),
                wiki_target_url(target)
            ));
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
            md.push_str(&format!("![{}]({})", alt.trim(), wiki_target_url(target)));
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
                    md.push('`');
                    let mut inner_plain = String::new();
                    let mut code = String::new();
                    render_nodes(nodes, &mut code, &mut inner_plain, depth);
                    // MediaWiki commonly serializes escaped punctuation (for
                    // example `x\_1`). Inside a Markdown code span those
                    // escapes are already literal text, so retaining the
                    // backslash makes it visible in the rendered article.
                    // Remove only Markdown punctuation escapes; preserve
                    // backslashes that are meaningful in code.
                    md.push_str(&unescape_code_span(&code));
                    md.push('`');
                    plain.push_str(&inner_plain);
                }
                "nowiki" => {
                    let mut inner_plain = String::new();
                    render_nodes(nodes, md, &mut inner_plain, depth);
                    plain.push_str(&inner_plain);
                }
                "br" => md.push_str("  \n"),
                "s" | "strike" | "del" => {
                    md.push_str("~~");
                    let mut inner_plain = String::new();
                    render_nodes(nodes, md, &mut inner_plain, depth);
                    md.push_str("~~");
                    plain.push_str(&inner_plain);
                }
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
        Node::StartTag { name, .. } => match name.as_ref() {
            "br" => md.push_str("  \n"),
            "s" | "strike" | "del" => md.push_str("~~"),
            _ => {}
        },
        Node::EndTag { name, .. } => match name.as_ref() {
            "s" | "strike" | "del" => md.push_str("~~"),
            _ => {}
        },
        Node::Comment { .. } | Node::MagicWord { .. } => {},
        Node::Table {
            attributes,
            captions,
            rows,
            ..
        } => {
            render_table(attributes, captions, rows, md, plain, depth);
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

fn unescape_code_span(code: &str) -> String {
    code.replace("\\`", "`")
        .replace("\\*", "*")
        .replace("\\_", "_")
        .replace("\\[", "[")
        .replace("\\]", "]")
}

fn normalize_markdown_code_spans(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut in_code = false;
    let mut chars = markdown.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '`' {
            in_code = !in_code;
            out.push(ch);
        } else if in_code && ch == '\\' {
            // Markdown punctuation escapes are unnecessary inside code spans.
            // Keep other backslashes (for example in `\\n`) intact.
            let next_is_punctuation = chars.peek().is_some_and(|next| "\\`*_[]".contains(*next));
            if !next_is_punctuation {
                out.push(ch);
            }
        } else {
            out.push(ch);
        }
    }
    out
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
    let name = normalize_template_name(&name);
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
            let text = args.first().map(|s| s.as_str()).unwrap_or(&name);
            md.push_str(&format!("**{}**", text));
            plain.push_str(text);
        }

        // Cmavo / grammar word classes — bold
        "cmavo" | "grammar" | "nunjikca" | "jikca" | "gln" | "cc" | "ir" | "mv" | "vj" | "ep"
        | "lg" | "leng" => {
            let text = args.first().map(|s| s.as_str()).unwrap_or(&name);
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
            let text = args.first().map(|s| s.as_str()).unwrap_or(&name);
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
            let word = args.first().map(|s| strip_numbered_arg(s)).unwrap_or("");
            if !word.is_empty() {
                md.push_str(&jvs_valsi_link(word));
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
        "color" | "colour" | "small" | "sc" | "overline" | "nobr" | "uc" | "lc" | "lang-en"
        | "lang-de" | "lang" => {
            // For lang: first arg is language code, second is text.
            // For others: first arg is the text (or second if first is a colour spec).
            let text = if name_lower == "color" || name_lower == "colour" {
                // {{color|#hex|text}}
                args.get(1)
                    .or_else(|| args.first())
                    .map(|s| s.as_str())
                    .unwrap_or("")
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
                    if author.is_empty() {
                        author = v.to_string();
                    }
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
            let links: Vec<&str> = args
                .iter()
                .filter(|a| !a.contains('='))
                .map(|s| s.as_str())
                .collect();
            if !links.is_empty() {
                md.push_str("\n\n**See also:** ");
                for (i, link) in links.iter().enumerate() {
                    if i > 0 {
                        md.push_str(", ");
                    }
                    md.push_str(&format!("[{}]({})", link, wiki_target_url(link)));
                    plain.push_str(link);
                    if i + 1 < links.len() {
                        plain.push_str(", ");
                    }
                }
                md.push_str("\n\n");
            }
        }

        // Hatnote / disambiguation / notice boxes — strip (navigation cruft)
        "hatnote" | "main" | "ombox" | "ambox" | "note" | "side box" | "about" | "for"
        | "redirect" | "distinguish" => {
            // Could render as blockquote, but they're mostly navigation noise.
        }

        // Quotation blocks — blockquote
        "quotation" | "quote box" | "rquote" => {
            let text = args
                .iter()
                .find(|a| !a.contains('='))
                .map(|s| s.as_str())
                .unwrap_or("");
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
        "ind"
        | "dsp"
        | "judri"
        | "lex"
        | "ssp"
        | "startchapter"
        | "bookcat"
        | "bpfk section box open"
        | "bpfk section box close"
        | "bpfk section from tiki"
        | "bpfk section poll"
        | "bpfk"
        | "se inspekte/en"
        | "se_inspekte/en"
        | "se_inspekte/jbo"
        | "se_inspekte/ru"
        | "jbocre/en"
        | "jbocre/ja"
        | "jbocre/fr"
        | "jbocre/jbo"
        | "jbocre"
        | "nalylojbo/en"
        | "comment"
        | "navigation"
        | "newpage"
        | "notci"
        | "secmavo"
        | "personal"
        | "csp"
        | "lfk"
        | "•"
        | "="
        | "clear"
        | "false"
        | "robox"
        | "robox/close"
        | "colwidth"
        | "div col"
        | "div col end"
        | "extraclasses"
        | "int:tadni-url"
        | "webchat url qwebirc"
        | "webchat url kiwi"
        | "webchat url"
        | "fullpagename"
        | "fullpagenamee"
        | "pagename"
        | "basepagename"
        | "basepagenamee"
        | "subpagename"
        | "namespace"
        | "ns"
        | "pageid"
        | "pagesize"
        | "numberofarticles"
        | "currentyear"
        | "currentday"
        | "currentmonth"
        | "currentversion"
        | "server"
        | "fullurl"
        | "localurl"
        | "canonicalurl"
        | "filepath"
        | "urlencode"
        | "formatnum"
        | "lcfirst"
        | "ucfirst"
        | "subst"
        | "defaultsort"
        | "displaytitle"
        | "int"
        | "subjectspace"
        | "subjectpagename"
        | "doc"
        | "template"
        | "template page"
        | "subject page"
        | "refimprove section"
        | "citation needed"
        | "by whom?"
        | "clarify"
        | "sic"
        | "unsolved"
        | "status"
        | "featured article"
        | "did you know"
        | "in the news"
        | "print version"
        | "print version notice"
        | "latest news"
        | "categorylist"
        | "languages"
        | "gfdl"
        | "associated wikimedia"
        | "wiktionarycat"
        | "commons"
        | "sbcnewline"
        | "sbcnewline2"
        | "волны ложбана"
        | "onde"
        | "wave chunks"
        | "guglgirzu lojban-soudan nuzyfle"
        | "guglgirzu ponjo_lojbo_citno_girzu nuzyfle"
        | "remoisocial"
        | "ennuzba"
        | "nuzba/jbo"
        | "nalcatni"
        | "infobox person"
        | "grammatical moods"
        | "lexical categories"
        | "val"
        | "vajni"
        | "mun"
        | "1"
        | "2"
        | "3"
        | "4"
        | "5"
        | "6"
        | "7"
        | "8"
        | "9"
        | "-"
        | "!"
        | "a"
        | "t"
        | "g"
        | "p/s" => {}

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
    attributes: &[Node<'_>],
    captions: &[TableCaption<'_>],
    rows: &[TableRow<'_>],
    md: &mut String,
    plain: &mut String,
    depth: usize,
) {
    // Emit real HTML tables. GFM pipe tables cannot express captions, cell
    // attributes (style/colspan/rowspan), or multi-row headers with uneven
    // cell counts — which left complex JACU-style matrices as raw `|` text.
    md.push_str("\n\n");
    md.push_str("<table");
    push_sanitized_attrs(md, &nodes_plain_text(attributes));
    md.push_str(">\n");

    for cap in captions {
        md.push_str("<caption");
        if let Some(attrs) = &cap.attributes {
            push_sanitized_attrs(md, &nodes_plain_text(attrs));
        }
        md.push('>');
        render_nodes_html(&cap.content, md, plain, depth);
        md.push_str("</caption>\n");
        plain.push('\n');
    }

    for row in rows {
        md.push_str("<tr");
        push_sanitized_attrs(md, &nodes_plain_text(&row.attributes));
        md.push_str(">\n");
        for cell in &row.cells {
            let tag = match cell.type_ {
                TableCellType::Heading => "th",
                TableCellType::Ordinary => "td",
            };
            md.push('<');
            md.push_str(tag);
            if let Some(attrs) = &cell.attributes {
                push_sanitized_attrs(md, &nodes_plain_text(attrs));
            }
            md.push('>');
            render_nodes_html(&cell.content, md, plain, depth);
            md.push_str("</");
            md.push_str(tag);
            md.push_str(">\n");
            plain.push(' ');
        }
        md.push_str("</tr>\n");
        plain.push('\n');
    }

    md.push_str("</table>\n\n");
}

fn nodes_plain_text(nodes: &[Node<'_>]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            Node::Text { value, .. } => out.push_str(value),
            Node::CharacterEntity { character, .. } => out.push(*character),
            _ => {}
        }
    }
    out
}

/// Append whitespace-prefixed sanitized HTML attributes, or nothing if empty/unsafe.
fn push_sanitized_attrs(out: &mut String, raw: &str) {
    let cleaned = sanitize_html_attribute_string(raw);
    if cleaned.is_empty() {
        return;
    }
    out.push(' ');
    out.push_str(&cleaned);
}

fn sanitize_html_attribute_string(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return String::new();
    }
    // Reject obvious script / event-handler payloads early.
    let lower = raw.to_ascii_lowercase();
    if lower.contains("javascript:") || lower.contains("data:text/html") {
        return String::new();
    }
    let mut out = String::new();
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let key_start = i;
        while i < bytes.len()
            && (bytes[i].is_ascii_alphanumeric()
                || bytes[i] == b'-'
                || bytes[i] == b'_'
                || bytes[i] == b':')
        {
            i += 1;
        }
        if i == key_start {
            i += 1;
            continue;
        }
        let key = raw[key_start..i].to_ascii_lowercase();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let mut value = String::new();
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i];
                i += 1;
                let vstart = i;
                while i < bytes.len() && bytes[i] != quote {
                    i += 1;
                }
                value.push_str(&raw[vstart..i]);
                if i < bytes.len() {
                    i += 1;
                }
            } else {
                let vstart = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                value.push_str(&raw[vstart..i]);
            }
        }
        if !is_allowed_table_attr(&key, &value) {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&key);
        out.push_str("=\"");
        out.push_str(&html_escape_attr(&value));
        out.push('"');
    }
    out
}

fn is_allowed_table_attr(key: &str, value: &str) -> bool {
    match key {
        "class" | "id" | "title" | "lang" | "dir" | "align" | "scope" | "headers"
        | "rowspan" | "colspan" | "width" | "height" | "bgcolor" => {
            !value.chars().any(|c| c == '<' || c == '>' || c == '"')
                && !value.to_ascii_lowercase().contains("javascript:")
        }
        "style" => is_safe_css_style(value),
        _ if key.starts_with("on") => false,
        _ => false,
    }
}

fn is_safe_css_style(style: &str) -> bool {
    let lower = style.to_ascii_lowercase();
    if lower.contains("expression")
        || lower.contains("javascript:")
        || lower.contains("behavior:")
        || lower.contains("-moz-binding")
        || lower.contains("url(")
        || lower.contains('@')
    {
        return false;
    }
    for decl in style.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        let prop = decl
            .split(':')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        let allowed = matches!(
            prop.as_str(),
            "background"
                | "background-color"
                | "color"
                | "text-align"
                | "vertical-align"
                | "width"
                | "height"
                | "min-width"
                | "max-width"
                | "padding"
                | "padding-left"
                | "padding-right"
                | "padding-top"
                | "padding-bottom"
                | "border"
                | "border-collapse"
                | "border-color"
                | "border-width"
                | "border-style"
                | "font-weight"
                | "font-style"
                | "font-size"
                | "white-space"
        );
        if !allowed {
            return false;
        }
    }
    true
}

fn html_escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn html_escape_attr(s: &str) -> String {
    html_escape_text(s).replace('\'', "&#39;")
}

/// Render wiki AST nodes as an HTML fragment (for table cells/captions).
/// Bold/Italic markers are toggles in parse-wiki-text (MediaWiki quotes).
fn render_nodes_html(nodes: &[Node<'_>], html: &mut String, plain: &mut String, depth: usize) {
    let mut bold = false;
    let mut italic = false;
    for node in nodes {
        match node {
            Node::Text { value, .. } => {
                html.push_str(&html_escape_text(value));
                plain.push_str(value);
            }
            Node::CharacterEntity { character, .. } => {
                html.push_str(&html_escape_text(&character.to_string()));
                plain.push(*character);
            }
            Node::Bold { .. } => {
                if bold {
                    html.push_str("</strong>");
                } else {
                    html.push_str("<strong>");
                }
                bold = !bold;
            }
            Node::Italic { .. } => {
                if italic {
                    html.push_str("</em>");
                } else {
                    html.push_str("<em>");
                }
                italic = !italic;
            }
            Node::BoldItalic { .. } => {
                if bold && italic {
                    html.push_str("</strong></em>");
                    bold = false;
                    italic = false;
                } else {
                    if !italic {
                        html.push_str("<em>");
                        italic = true;
                    }
                    if !bold {
                        html.push_str("<strong>");
                        bold = true;
                    }
                }
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
                        buf.replace("**", "").replace('*', "").replace("~~", "")
                    }
                };
                html.push_str("<a href=\"");
                html.push_str(&html_escape_attr(&wiki_target_url(target)));
                html.push_str("\">");
                html.push_str(&html_escape_text(label.trim()));
                html.push_str("</a>");
                plain.push_str(label.trim());
            }
            Node::ExternalLink { nodes, .. } => {
                let mut buf = String::new();
                let mut sink = String::new();
                render_nodes(nodes, &mut buf, &mut sink, depth);
                let trimmed = buf.trim();
                let (url, label) = match trimmed.split_once(char::is_whitespace) {
                    Some((u, l)) => (u.trim().to_string(), l.trim().to_string()),
                    None => (trimmed.to_string(), String::new()),
                };
                if url.is_empty() {
                    continue;
                }
                let display = if label.is_empty() {
                    url.clone()
                } else {
                    label
                };
                html.push_str("<a href=\"");
                html.push_str(&html_escape_attr(&url));
                html.push_str("\">");
                html.push_str(&html_escape_text(&display));
                html.push_str("</a>");
                plain.push_str(&display);
            }
            Node::Tag { name, nodes, .. } => {
                let n = name.as_ref();
                match n {
                    "code" | "tt" => {
                        html.push_str("<code>");
                        let mut inner_plain = String::new();
                        let mut code = String::new();
                        render_nodes(nodes, &mut code, &mut inner_plain, depth);
                        html.push_str(&html_escape_text(&unescape_code_span(&code)));
                        html.push_str("</code>");
                        plain.push_str(&inner_plain);
                    }
                    "s" | "strike" | "del" => {
                        html.push_str("<s>");
                        render_nodes_html(nodes, html, plain, depth);
                        html.push_str("</s>");
                    }
                    "br" => html.push_str("<br>"),
                    "nowiki" => {
                        let mut sink = String::new();
                        let mut raw = String::new();
                        render_nodes(nodes, &mut raw, &mut sink, depth);
                        html.push_str(&html_escape_text(&raw));
                        plain.push_str(&sink);
                    }
                    _ => {
                        render_nodes_html(nodes, html, plain, depth);
                    }
                }
            }
            Node::StartTag { name, .. } => match name.as_ref() {
                "br" => html.push_str("<br>"),
                "s" | "strike" | "del" => html.push_str("<s>"),
                _ => {}
            },
            Node::EndTag { name, .. } => match name.as_ref() {
                "s" | "strike" | "del" => html.push_str("</s>"),
                _ => {}
            },
            Node::Template {
                name, parameters, ..
            } => {
                let mut buf = String::new();
                let mut sink = String::new();
                render_template(name, parameters, &mut buf, &mut sink, depth);
                html.push_str(&inline_markdown_fragment_to_html(&buf));
                plain.push_str(&sink);
            }
            Node::ParagraphBreak { .. } => {
                html.push(' ');
                plain.push('\n');
            }
            Node::Comment { .. } | Node::MagicWord { .. } => {}
            Node::Table {
                attributes,
                captions,
                rows,
                ..
            } => {
                render_table(attributes, captions, rows, html, plain, depth);
            }
            other => {
                let mut buf = String::new();
                let mut sink = String::new();
                render_node(other, &mut buf, &mut sink, depth);
                html.push_str(&inline_markdown_fragment_to_html(&buf));
                plain.push_str(&sink);
            }
        }
    }
    if bold {
        html.push_str("</strong>");
    }
    if italic {
        html.push_str("</em>");
    }
}

/// Convert a small markdown fragment (links, emphasis, code, strike) to HTML.
fn inline_markdown_fragment_to_html(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    let chars: Vec<char> = md.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '[' {
            if let Some((label, url, consumed)) = parse_md_link(&chars[i..]) {
                out.push_str("<a href=\"");
                out.push_str(&html_escape_attr(&url));
                out.push_str("\">");
                out.push_str(&html_escape_text(&label));
                out.push_str("</a>");
                i += consumed;
                continue;
            }
        }
        if chars[i] == '`' {
            if let Some(end) = chars[i + 1..].iter().position(|&c| c == '`') {
                let inner: String = chars[i + 1..i + 1 + end].iter().collect();
                out.push_str("<code>");
                out.push_str(&html_escape_text(&inner));
                out.push_str("</code>");
                i += end + 2;
                continue;
            }
        }
        if matches_run(&chars, i, "***") || matches_run(&chars, i, "___") {
            let delim = if chars[i] == '*' { "***" } else { "___" };
            if let Some(end) = find_closing_delim(&chars, i + 3, delim) {
                let inner: String = chars[i + 3..end].iter().collect();
                out.push_str("<em><strong>");
                out.push_str(&inline_markdown_fragment_to_html(&inner));
                out.push_str("</strong></em>");
                i = end + 3;
                continue;
            }
        }
        if matches_run(&chars, i, "**") || matches_run(&chars, i, "__") {
            let delim = if chars[i] == '*' { "**" } else { "__" };
            if let Some(end) = find_closing_delim(&chars, i + 2, delim) {
                let inner: String = chars[i + 2..end].iter().collect();
                out.push_str("<strong>");
                out.push_str(&inline_markdown_fragment_to_html(&inner));
                out.push_str("</strong>");
                i = end + 2;
                continue;
            }
        }
        if matches_run(&chars, i, "~~") {
            if let Some(end) = find_closing_delim(&chars, i + 2, "~~") {
                let inner: String = chars[i + 2..end].iter().collect();
                out.push_str("<s>");
                out.push_str(&inline_markdown_fragment_to_html(&inner));
                out.push_str("</s>");
                i = end + 2;
                continue;
            }
        }
        if chars[i] == '*' || chars[i] == '_' {
            let delim = chars[i].to_string();
            if let Some(end) = find_closing_delim(&chars, i + 1, &delim) {
                let inner: String = chars[i + 1..end].iter().collect();
                if !inner.is_empty() {
                    out.push_str("<em>");
                    out.push_str(&inline_markdown_fragment_to_html(&inner));
                    out.push_str("</em>");
                    i = end + 1;
                    continue;
                }
            }
        }
        out.push_str(&html_escape_text(&chars[i].to_string()));
        i += 1;
    }
    out
}

fn matches_run(chars: &[char], i: usize, lit: &str) -> bool {
    let lit: Vec<char> = lit.chars().collect();
    i + lit.len() <= chars.len() && chars[i..i + lit.len()] == lit[..]
}

fn find_closing_delim(chars: &[char], start: usize, delim: &str) -> Option<usize> {
    let d: Vec<char> = delim.chars().collect();
    let mut i = start;
    while i + d.len() <= chars.len() {
        if chars[i..i + d.len()] == d[..] {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn parse_md_link(chars: &[char]) -> Option<(String, String, usize)> {
    if chars.first() != Some(&'[') {
        return None;
    }
    let mut i = 1;
    let mut label = String::new();
    while i < chars.len() {
        if chars[i] == ']' {
            break;
        }
        if chars[i] == '\\' && i + 1 < chars.len() {
            label.push(chars[i + 1]);
            i += 2;
            continue;
        }
        label.push(chars[i]);
        i += 1;
    }
    if i >= chars.len() || chars[i] != ']' {
        return None;
    }
    i += 1;
    if i >= chars.len() || chars[i] != '(' {
        return None;
    }
    i += 1;
    let mut url = String::new();
    while i < chars.len() && chars[i] != ')' {
        url.push(chars[i]);
        i += 1;
    }
    if i >= chars.len() || chars[i] != ')' {
        return None;
    }
    Some((label, url, i + 1))
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
        assert!(
            rewritten.contains("[the language](/wiki/Lojban)"),
            "rewritten={rewritten}"
        );
        assert!(
            rewritten.contains("[external](https://example.com)"),
            "rewritten={rewritten}"
        );
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
        assert!(
            !md.contains("{{"),
            "md should not contain raw template syntax: {md}"
        );
        assert!(
            md.contains("before") && md.contains("after"),
            "surrounding text preserved: {md}"
        );
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
        assert!(
            md.contains("Text") && md.contains("more") && md.contains("text"),
            "md={md}"
        );
    }

    #[test]
    fn math_template_renders_code() {
        let (md, _) = wikitext_to_markdown("{{math|3x + 2y}}");
        assert!(md.contains("`3x + 2y`"), "md={md}");
    }

    #[test]
    fn escaped_underscore_in_inline_code_is_not_visible() {
        let (md, plain) = wikitext_to_markdown("`x\\_1`");
        assert_eq!(md.trim(), "`x_1`");
        assert_eq!(plain.trim(), "`x\\_1`");
    }

    #[test]
    fn jbovlaste_link_template() {
        let (md, plain) = wikitext_to_markdown("See {{jvs|broda}} for details.");
        assert!(md.contains("[broda](/valsi/broda)"), "md={md}");
        assert!(!md.contains("{{"), "raw template leaked: {md}");
        assert!(plain.contains("broda"), "plain={plain}");
    }

    #[test]
    fn jvs_template_with_apostrophe_in_list_is_rewritten() {
        let (md, _) = wikitext_to_markdown("* {{jvs|se du'o}}\n* {{jvs|te du'o}}");
        assert!(
            md.contains("[se du'o](/valsi/se_du%27o)") || md.contains("se du'o"),
            "md={md}"
        );
        assert!(!md.contains("{{jvs"), "raw jvs leaked: {md}");
        assert!(!md.contains("{{"), "raw template leaked: {md}");
    }

    #[test]
    fn jvs_template_prefix_and_unparsed_source_filtered() {
        let (md, _) = wikitext_to_markdown("{{Template:jvs|cusku}} and leftover {{jvs|zanru}}");
        assert!(!md.contains("{{"), "raw template leaked: {md}");
        assert!(md.contains("cusku") || md.contains("zanru"), "md={md}");
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
        assert!(
            md.contains("[Lojban orthographies](/papri/Lojban_orthographies)"),
            "md={md}"
        );
    }

    #[test]
    fn quotation_template_renders_blockquote() {
        let (md, _) = wikitext_to_markdown("{{Quotation|This is a quote}}");
        assert!(md.contains("> This is a quote"), "md={md}");
    }

    #[test]
    fn lm_template_renders_table_row() {
        let (md, _) = wikitext_to_markdown("{{lm|mi prami do|I love you|gloss}}");
        assert!(
            md.contains("| mi prami do | I love you | gloss |"),
            "md={md}"
        );
    }

    #[test]
    fn navigation_templates_stripped() {
        let (md, _) = wikitext_to_markdown("Text {{Navigation}} more {{Newpage}} text");
        assert!(!md.contains("Navigation"), "md={md}");
        assert!(!md.contains("Newpage"), "md={md}");
        assert!(
            md.contains("Text") && md.contains("more") && md.contains("text"),
            "md={md}"
        );
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

    #[test]
    fn mediawiki_simple_wikitable_emits_html() {
        let wt = r#"{| class="wikitable"
|+ Uses of JA*
! connective type
! example
|-
| logical ''sumti'' connective
|style="background-color:#ddddff"| ''ko'a ja ko'e''
|}
"#;
        let (md, plain) = wikitext_to_markdown(wt);
        assert!(md.contains("<table class=\"wikitable\">"), "md={md}");
        assert!(md.contains("<caption>"), "md={md}");
        assert!(md.contains("Uses of JA*"), "md={md}");
        assert!(md.contains("</caption>"), "md={md}");
        assert!(md.contains("<th>"), "md={md}");
        assert!(
            md.contains("style=\"background-color:#ddddff\""),
            "md={md}"
        );
        assert!(md.contains("<em>sumti</em>"), "md={md}");
        assert!(md.contains("<em>ko'a ja ko'e</em>"), "md={md}");
        assert!(!md.contains("| --- |"), "should not emit GFM pipes: {md}");
        assert!(plain.contains("sumti"), "plain={plain}");
        assert!(plain.contains("Uses of JA*"), "plain={plain}");
    }

    #[test]
    fn mediawiki_colspan_rowspan_matrix_emits_html() {
        let wt = r#"{| class="wikitable"
|+ Afterthought connectives
!rowspan="2"| argument type
!colspan="2"| logical
!colspan="2"| non-logical
|-
!CLL
!"JACU"
!CLL
!"JACU"
|-
| ''sumti''
| ''ko'a .a ko'e''
|style="background-color:#ddddff"| ''ko'a ja ko'e''
| ''ko'a joi ko'e''
| ''ko'a joi ko'e''
|-
| relative clause
|style="background-color:#dddddd"| <s>undefined</s>
|style="background-color:#ddffdd"| ''poi broda ja poi brode''
| ''poi broda zi'e poi brode''
|style="background-color:#ddddff"| ''poi broda joi poi brode''
|}
"#;
        let (md, _) = wikitext_to_markdown(wt);
        assert!(md.contains("<table class=\"wikitable\">"), "md={md}");
        assert!(md.contains("<caption>Afterthought connectives</caption>"), "md={md}");
        assert!(md.contains("rowspan=\"2\""), "md={md}");
        assert!(md.contains("colspan=\"2\""), "md={md}");
        assert!(md.contains("<s>undefined</s>"), "md={md}");
        assert!(
            md.contains("style=\"background-color:#ddffdd\""),
            "md={md}"
        );
        // Must not fall back to broken GFM with literal separators.
        assert!(!md.contains("| --- |"), "md={md}");
        assert!(!md.contains("| argument type |"), "md={md}");
    }

    #[test]
    fn mediawiki_complexity_table_keeps_count_column() {
        let wt = r#"{| class="wikitable"
|+ Complexity
! connective system
! logical connective ''cmavo''
! count
|-
| CLL
| ''.a'', ''.e'', ''ja''
| 27
|-
| "JACU"
| ''ja'', ''je'', ''gi''
| 9
|}
"#;
        let (md, plain) = wikitext_to_markdown(wt);
        assert!(md.contains("<caption>Complexity</caption>"), "md={md}");
        assert!(md.contains("<th>count</th>") || md.contains(">count</th>"), "md={md}");
        assert!(md.contains(">27</td>") || md.contains(">27<"), "md={md}");
        assert!(md.contains(">9</td>") || md.contains(">9<"), "md={md}");
        assert!(plain.contains("27"), "plain={plain}");
        assert!(plain.contains("9"), "plain={plain}");
    }

    #[test]
    fn strikethrough_start_end_tags_in_prose() {
        let (md, _) = wikitext_to_markdown("before <s>undefined</s> after");
        assert!(md.contains("~~undefined~~"), "md={md}");
    }

    #[test]
    fn rewrite_wiki_links_rewrites_html_hrefs() {
        let html = r#"See <a href="/papri/Lojban">Lojban</a>."#;
        let rewritten = rewrite_wiki_links_for_lensisku(html);
        assert!(
            rewritten.contains("href=\"/wiki/Lojban\""),
            "rewritten={rewritten}"
        );
    }

    #[test]
    fn unsafe_table_attrs_are_stripped() {
        let wt = r#"{| class="wikitable" onclick="alert(1)"
| style="background-color:#ddddff; expression(alert(1))"| x
|}
"#;
        let (md, _) = wikitext_to_markdown(wt);
        assert!(md.contains("<table"), "md={md}");
        assert!(!md.to_ascii_lowercase().contains("onclick"), "md={md}");
        assert!(!md.to_ascii_lowercase().contains("expression"), "md={md}");
    }

}
