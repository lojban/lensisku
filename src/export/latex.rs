use chrono::Datelike;

// Constants
pub(crate) const JAPANESE: &str = "ja";
pub(crate) const GUASPI: &str = "art-guaspi";

pub(crate) fn escape_all(term: &str) -> String {
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

pub(crate) fn escape_tex(term: &str, escape_carets: bool) -> String {
    let mut result = String::with_capacity(term.len());
    for c in term.chars() {
        match c {
            '\\' => result.push_str("\\textbackslash{}"),
            '>' => result.push_str("\\textgreater{}"),
            '<' => result.push_str("\\textless{}"),
            '–' => result.push_str("\\textendash{}"),
            '—' => result.push_str("\\textemdash{}"),
            '~' => result.push_str("\\textasciitilde{}"),
            '^' if escape_carets => result.push_str("\\textasciicircum{}"),
            '/' => result.push_str("\\slash{}"),
            // Valid math is protected before this runs. Remaining math syntax
            // and braces are literal text, including malformed delimiters.
            '#' | '%' | '&' | '$' | '_' | '{' | '}' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result
}

/// Escape so long headwords/tokens can wrap inside twocolumn layout.
pub(crate) fn breakable_escape_all(term: &str) -> String {
    breakable_escape_segment(term, false, true)
}

pub(crate) fn breakable_escape_segment(
    term: &str,
    escape_carets: bool,
    full_escape: bool,
) -> String {
    let mut out = String::with_capacity(term.len().saturating_mul(2));
    for c in term.chars() {
        out.push_str(&escape_segment(&c.to_string(), escape_carets, full_escape));
        if !c.is_whitespace() {
            // Terminate the control sequence: bare `\allowbreak` + letter is
            // parsed as `\allowbreakb` etc. and fails with undefined control sequence.
            out.push_str("\\allowbreak{}");
        }
    }
    out
}

pub(crate) fn escape_segment(term: &str, escape_carets: bool, full_escape: bool) -> String {
    if full_escape {
        escape_all(term)
    } else {
        escape_tex(term, escape_carets)
    }
}

/// Insert break opportunities into long non-space runs so body text wraps.
/// Math placeholders must stay intact so they can be restored later.
pub(crate) fn escape_segment_with_breaks(
    term: &str,
    escape_carets: bool,
    full_escape: bool,
) -> String {
    let mut out = String::with_capacity(term.len().saturating_mul(2));
    let mut rest = term;
    while let Some(start) = rest.find(MATH_PLACEHOLDER_PREFIX) {
        let after_prefix = &rest[start + MATH_PLACEHOLDER_PREFIX.len()..];
        if let Some(end_rel) = after_prefix.find(MATH_PLACEHOLDER_SUFFIX) {
            let placeholder_end =
                start + MATH_PLACEHOLDER_PREFIX.len() + end_rel + MATH_PLACEHOLDER_SUFFIX.len();
            // Only treat as placeholder if the middle is digits.
            let middle = &after_prefix[..end_rel];
            if !middle.is_empty() && middle.chars().all(|c| c.is_ascii_digit()) {
                out.push_str(&escape_runs_with_breaks(
                    &rest[..start],
                    escape_carets,
                    full_escape,
                ));
                out.push_str(&rest[start..placeholder_end]);
                rest = &rest[placeholder_end..];
                continue;
            }
        }
        // Not a real placeholder — escape through the prefix char-by-char path.
        out.push_str(&escape_runs_with_breaks(
            &rest[..start + MATH_PLACEHOLDER_PREFIX.len()],
            escape_carets,
            full_escape,
        ));
        rest = &rest[start + MATH_PLACEHOLDER_PREFIX.len()..];
    }
    out.push_str(&escape_runs_with_breaks(rest, escape_carets, full_escape));
    out
}

pub(crate) fn escape_runs_with_breaks(
    term: &str,
    escape_carets: bool,
    full_escape: bool,
) -> String {
    const LONG_TOKEN_CHARS: usize = 12;
    let mut out = String::with_capacity(term.len().saturating_mul(2));
    let mut run = String::new();

    let flush_run = |run: &mut String, out: &mut String| {
        if run.is_empty() {
            return;
        }
        if run.chars().count() >= LONG_TOKEN_CHARS {
            out.push_str(&breakable_escape_segment(run, escape_carets, full_escape));
        } else {
            out.push_str(&escape_segment(run, escape_carets, full_escape));
        }
        run.clear();
    };

    for c in term.chars() {
        if c.is_whitespace() {
            flush_run(&mut run, &mut out);
            out.push(c);
        } else {
            run.push(c);
        }
    }
    flush_run(&mut run, &mut out);
    out
}

pub(crate) const MATH_PLACEHOLDER_PREFIX: &str = "MJXPH";
pub(crate) const MATH_PLACEHOLDER_SUFFIX: &str = "MJXEND";

pub(crate) fn math_placeholder(index: usize) -> String {
    format!("{MATH_PLACEHOLDER_PREFIX}{index}{MATH_PLACEHOLDER_SUFFIX}")
}

/// Protect `$...$` / `$$...$$` so curly-link parsing does not touch math braces.
pub(crate) fn protect_tex_math(text: &str) -> (String, Vec<String>) {
    let mut parts = Vec::new();
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < text.len() {
        if text[i..].starts_with("$$") {
            if let Some(rel) = text[i + 2..].find("$$") {
                let end = i + 2 + rel + 2;
                parts.push(text[i..end].to_string());
                out.push_str(&math_placeholder(parts.len() - 1));
                i = end;
                continue;
            }
        }
        if bytes[i] == b'$' && text.get(i + 1..).is_none_or(|s| !s.starts_with('$')) {
            if let Some(rel) = text[i + 1..].find('$') {
                let end = i + 1 + rel + 1;
                parts.push(text[i..end].to_string());
                out.push_str(&math_placeholder(parts.len() - 1));
                i = end;
                continue;
            }
        }
        let ch = text[i..].chars().next().unwrap_or('\0');
        out.push(ch);
        i += ch.len_utf8();
    }
    (out, parts)
}

pub(crate) fn restore_tex_math(text: &str, parts: &[String]) -> String {
    if parts.is_empty() {
        return text.to_string();
    }
    let mut out = text.to_string();
    for (i, part) in parts.iter().enumerate() {
        out = out.replace(&math_placeholder(i), part);
    }
    out
}

pub(crate) fn is_numeric_curly_inner(inner: &str) -> bool {
    !inner.is_empty() && inner.chars().all(|c| c.is_ascii_digit())
}

/// Format dictionary text for LaTeX: `{klama}` → italic `klama`, with escaping and wraps.
pub(crate) fn format_export_text(text: &str, escape_carets: bool, full_escape: bool) -> String {
    let (protected, math_parts) = protect_tex_math(text);
    let mut out = String::with_capacity(protected.len().saturating_mul(2));
    let mut rest = protected.as_str();

    while let Some(start) = rest.find('{') {
        out.push_str(&escape_segment_with_breaks(
            &rest[..start],
            escape_carets,
            full_escape,
        ));
        let after_open = &rest[start + 1..];
        if let Some(end) = after_open.find('}') {
            let inner = &after_open[..end];
            let after = &after_open[end + 1..];
            if is_numeric_curly_inner(inner) {
                // Keep subscript-like `{1}` out of link conversion (matches UI).
                out.push_str(&escape_segment(
                    &format!("{{{inner}}}"),
                    escape_carets,
                    full_escape,
                ));
            } else {
                let trimmed = inner.trim();
                if trimmed.is_empty() {
                    out.push_str(&escape_segment("{}", escape_carets, full_escape));
                } else {
                    out.push_str(&format!(
                        "\\textit{{{}}}",
                        breakable_escape_segment(trimmed, escape_carets, true)
                    ));
                }
            }
            rest = after;
        } else {
            out.push_str(&escape_segment_with_breaks(
                rest,
                escape_carets,
                full_escape,
            ));
            rest = "";
            break;
        }
    }
    out.push_str(&escape_segment_with_breaks(
        rest,
        escape_carets,
        full_escape,
    ));
    restore_tex_math(&out, &math_parts)
}

pub(crate) fn generate_title(escaped_lang: &str, collection_id: Option<i32>) -> String {
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

pub(crate) fn format_lojban_heading(word: &str, valsi_type: &str) -> String {
    let escaped_word = escape_all(word);
    let breakable_word = breakable_escape_all(word);
    let heading = if valsi_type.starts_with("experimental") || valsi_type.starts_with("obsolete") {
        format_lojban_experimental_heading(&breakable_word)
    } else {
        format_normal_heading(&breakable_word)
    };
    format!("{}{}", heading, markboth(&escaped_word))
}

pub(crate) fn format_normal_heading(escaped_word: &str) -> String {
    format!("\n\n{{\\sffamily\\bfseries {}}}", escaped_word)
}

pub(crate) fn format_lojban_experimental_heading(escaped_word: &str) -> String {
    format!("\n\n{{\\sffamily\\bfseries $\\triangle$ {}}}", escaped_word)
}

pub(crate) fn markboth(escaped_word: &str) -> String {
    format!("\\markboth{{{}}}{{{}}}", escaped_word, escaped_word)
}

pub(crate) fn format_rafsi(rafsi: &Option<String>) -> String {
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

pub(crate) fn format_selmaho(selmaho: &Option<String>) -> String {
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

pub(crate) fn format_definition(definition: &str, lang: &str) -> String {
    let carets_are_literal = lang == GUASPI;
    format!(
        " {}",
        format_export_text(definition, carets_are_literal, false)
    )
}

pub(crate) fn format_notes(notes: &Option<String>) -> String {
    match notes {
        Some(n) if !n.is_empty() => {
            let full_escape = !sniff_tex(n);
            format!(
                " \\textemdash{{}} {}",
                format_export_text(n, false, full_escape)
            )
        }
        _ => String::new(),
    }
}

pub(crate) fn sniff_tex(text: &str) -> bool {
    text.contains('$')
}

pub(crate) fn format_natural_heading(word: &str) -> String {
    let escaped_word = escape_all(word);
    let heading = format_normal_heading(&breakable_escape_all(word));
    format!("{}{}", heading, markboth(&escaped_word))
}

pub(crate) fn format_meaning(meaning: &Option<String>) -> String {
    meaning
        .as_ref()
        .map(|m| format!("\\textit{{({})}} ", escape_all(m)))
        .unwrap_or_default()
}

pub(crate) fn format_valsi(valsi: &str) -> String {
    format!(" {}", breakable_escape_all(valsi))
}

pub(crate) fn format_place(place: i32) -> String {
    if place > 0 {
        format!("$_{{{}}}$", place)
    } else {
        String::new()
    }
}
pub(crate) fn latex_header(title: &str, lang: &str) -> String {
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

pub(crate) fn latex_preamble(lang: &str) -> String {
    format!(
        "{}{}{}",
        latex_preamble_intro(),
        latex_preamble_fonts(lang),
        latex_preamble_outro()
    )
}

pub(crate) fn latex_preamble_intro() -> String {
    r#"%!TEX encoding = UTF-8 Unicode
%!TEX TS-program = xelatex
\documentclass[notitlepage,twocolumn,a4paper,10pt]{book}
\renewcommand\chaptername{ni'o ni'o}

\usepackage{underscore}
\usepackage{amsmath,amssymb}
\usepackage{graphicx}
% PDF generation supplies local image files; text-only LaTeX exports omit them.
\newcommand{\definitionimages}[1]{}
\newcommand{\collectionimages}[1]{}
% Keep illustrated entries together when they fit; long entries remain breakable.
\newsavebox{\exportentrybox}
\newcommand{\exportentry}[2]{%
  \if\relax\detokenize{#1}\relax
    #2%
  \else
    \sbox{\exportentrybox}{\begin{minipage}{\linewidth}#2#1\end{minipage}}%
    \ifdim\dimexpr\ht\exportentrybox+\dp\exportentrybox\relax>\textheight
      #2#1%
    \else
      \par\noindent\usebox{\exportentrybox}\par
    \fi
  \fi
}

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

pub(crate) fn latex_preamble_fonts(lang: &str) -> String {
    format!(
        "{}{}",
        latex_preamble_roman_fonts(),
        latex_preamble_cjk_fonts(lang)
    )
}

pub(crate) fn latex_preamble_roman_fonts() -> String {
    r#"
\setromanfont[Mapping=tex-text]{Linux Libertine O}
\setsansfont[Mapping=tex-text]{Linux Biolinum O}"#
        .to_string()
}

pub(crate) fn latex_preamble_cjk_fonts(lang: &str) -> String {
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
\setCJKmainfont[Mapping=tex-text]{Noto Sans CJK SC}
\setCJKsansfont[Mapping=tex-text]{Noto Sans CJK SC}
\setCJKmonofont[Mapping=tex-text]{Noto Sans CJK SC}"#
            .to_string(),
    }
}

pub(crate) fn latex_preamble_outro() -> String {
    r#"
\fancyhead{}          % empty out the header
\fancyfoot{}          % empty out the footer
\fancyhead[LE,LO]{\rightmark} % left side, odd and even pages
\fancyhead[RE,RO]{\leftmark}  % right side, odd and even pages
\fancyfoot[LE,RO]{\thepage}   % left side even, right side odd

\setlength{\parindent}{1 em}

% Allow long valsi / unbroken tokens to wrap instead of overflowing columns.
\sloppy
\setlength{\emergencystretch}{3em}"#
        .to_string()
}

pub(crate) fn latex_footer() -> String {
    "\n\\end{document}".to_string()
}
