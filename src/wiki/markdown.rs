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
    let joined = if args.is_empty() {
        format!("{{{{{}}}}}", name)
    } else {
        format!("{{{{{}|{}}}}}", name, args.join("|"))
    };
    md.push_str(&format!("_{}_", joined));
    plain.push_str(&joined);
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
    fn template_renders_as_italic() {
        let (md, _) = wikitext_to_markdown("{{stub|reason=test}}");
        assert!(md.contains("_{{stub|reason=test}}_"), "md={md}");
    }

    #[test]
    fn redirect_recognised() {
        let (md, plain) = wikitext_to_markdown("#REDIRECT [[Other]]");
        assert!(md.to_lowercase().contains("redirect"), "md={md}");
        assert!(plain.to_lowercase().contains("redirect"));
    }
}
