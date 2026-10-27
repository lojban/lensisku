//! Dialogue script parser (pure). Syntax (one block per line, blank lines ignored):
//!
//! ```text
//! # Title                       optional title
//! > narrator line               narrator
//! M: text / F: text             anonymous male / female speaker
//! Marian(F): text               named speaker with gender (gender remembered for later `Marian:`)
//! Marian: text                  named speaker (gender from an earlier line, else unknown)
//! [Eva] text / [Eva] Marian: …  explicit Kokoro voice for this line
//! ~ gloss                       translation of the previous line
//! ---                           questions follow
//! ? fill: Marian(F): lo mi [ckiku] cu zvati ma     typed blank ([a|b] = alternatives)
//! ? bank: lo do [ckiku] cu cpana lo jubme  (chips: karce, ckafi)   word-bank blank
//! ? choice: Where are the keys?  + on the table  - in the car      (or + / - on next lines)
//! ```
use serde::Serialize;
use std::collections::HashMap;
use utoipa::ToSchema;

pub const VOICES: [&str; 4] = ["Martin", "Victoria", "Bernd", "Eva"];
pub const MALE_VOICES: [&str; 2] = ["Martin", "Bernd"];
pub const FEMALE_VOICES: [&str; 2] = ["Victoria", "Eva"];
pub const NARRATOR: &str = "narrator";

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq)]
pub struct DialogueLine {
    pub index: usize,
    /// Speaker key: `narrator`, `M`, `F` or a character name.
    pub speaker: String,
    /// `m`, `f` or `None` when unknown (narrator, unnamed).
    pub gender: Option<String>,
    /// `[Voice]` override for this line.
    pub voice_override: Option<String>,
    pub text: String,
    pub gloss: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq)]
pub struct ChoiceOption {
    pub text: String,
    pub correct: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq)]
pub struct DialogueQuestion {
    pub index: usize,
    /// `fill`, `bank` or `choice`.
    pub kind: String,
    /// Choice: the question. Fill/bank: the speaker of the line (may be empty).
    pub prompt: String,
    /// Fill/bank: text before / after the blank.
    pub before: String,
    pub after: String,
    /// Accepted answers (fill/bank), first is canonical.
    pub answers: Vec<String>,
    /// Choice options; bank: extra distractor chips (`correct` false).
    pub options: Vec<ChoiceOption>,
}

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq)]
pub struct ParseIssue {
    /// 1-based source line.
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Default)]
pub struct Dialogue {
    pub title: Option<String>,
    pub lines: Vec<DialogueLine>,
    pub questions: Vec<DialogueQuestion>,
    pub errors: Vec<ParseIssue>,
}

pub fn canonical_voice(name: &str) -> Option<&'static str> {
    VOICES
        .iter()
        .copied()
        .find(|v| v.eq_ignore_ascii_case(name.trim()))
}

fn parse_gender(s: &str) -> Option<&'static str> {
    match s.trim() {
        "M" | "m" => Some("m"),
        "F" | "f" => Some("f"),
        _ => None,
    }
}

struct Speech {
    speaker: String,
    gender: Option<String>,
    voice: Option<String>,
    text: String,
}

/// `[Voice] Name(G): text`, `M: text`, `Name: text`, `> text`. `None` = not a speech line.
fn parse_speech(
    raw: &str,
    genders: &mut HashMap<String, String>,
) -> Result<Option<Speech>, String> {
    let mut s = raw.trim();
    let mut voice = None;
    if let Some(rest) = s.strip_prefix('[') {
        let Some(end) = rest.find(']') else {
            return Err("unclosed [Voice]".into());
        };
        let v = &rest[..end];
        voice = Some(
            canonical_voice(v)
                .ok_or_else(|| format!("unknown voice [{v}] (use {})", VOICES.join(", ")))?
                .to_string(),
        );
        s = rest[end + 1..].trim_start();
    }
    if let Some(text) = s.strip_prefix('>') {
        return Ok(Some(Speech {
            speaker: NARRATOR.into(),
            gender: None,
            voice,
            text: text.trim().into(),
        }));
    }
    // Speaker prefix: up to the first ':' if the head looks like a name (no spaces beyond one word
    // group, no brackets).
    if let Some(colon) = s.find(':') {
        let head = s[..colon].trim();
        let text = s[colon + 1..].trim();
        let valid_head = !head.is_empty()
            && head.len() <= 40
            && !head.contains('[')
            && !head.chars().any(|c| c == '.' || c == '\'');
        if valid_head {
            let (name, gender) = if let Some(open) = head.find('(') {
                let close = head.rfind(')').ok_or("unclosed (gender)")?;
                let g = parse_gender(&head[open + 1..close])
                    .ok_or_else(|| format!("gender must be M or F in {head}"))?;
                (head[..open].trim().to_string(), Some(g.to_string()))
            } else if let Some(g) = parse_gender(head) {
                (g.to_ascii_uppercase(), Some(g.to_string()))
            } else {
                (head.to_string(), None)
            };
            if name.is_empty() {
                return Err("empty speaker name".into());
            }
            let gender = match gender {
                Some(g) => {
                    genders.insert(name.clone(), g.clone());
                    Some(g)
                }
                None => genders.get(&name).cloned(),
            };
            return Ok(Some(Speech {
                speaker: name,
                gender,
                voice,
                text: text.into(),
            }));
        }
    }
    if let Some(v) = voice {
        // `[Eva] text`: the voice itself is the speaker.
        let gender = if FEMALE_VOICES.contains(&v.as_str()) {
            "f"
        } else {
            "m"
        };
        return Ok(Some(Speech {
            speaker: v.clone(),
            gender: Some(gender.into()),
            voice: Some(v),
            text: s.into(),
        }));
    }
    Ok(None)
}

/// `before [a|b] after` → (before, answers, after).
fn split_blank(s: &str) -> Result<(String, Vec<String>, String), String> {
    let open = s.find('[').ok_or("missing [blank]")?;
    let close = s[open..]
        .find(']')
        .map(|i| i + open)
        .ok_or("unclosed [blank]")?;
    let answers: Vec<String> = s[open + 1..close]
        .split('|')
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .collect();
    if answers.is_empty() {
        return Err("empty [blank]".into());
    }
    if s[close + 1..].contains('[') {
        return Err("only one [blank] per question".into());
    }
    Ok((
        s[..open].trim_end().to_string(),
        answers,
        s[close + 1..].trim_start().to_string(),
    ))
}

/// Inline `+ a  - b` options (separated by two+ spaces before a sign) or none.
fn split_inline_options(s: &str) -> (String, Vec<ChoiceOption>) {
    let mut cut = None;
    let b = s.as_bytes();
    for i in 0..b.len() {
        if (b[i] == b'+' || b[i] == b'-')
            && i >= 2
            && b[i - 1] == b' '
            && b[i - 2] == b' '
            && b.get(i + 1) == Some(&b' ')
        {
            cut = Some(i);
            break;
        }
    }
    let Some(cut) = cut else {
        return (s.trim().into(), vec![]);
    };
    let mut opts = vec![];
    let rest = &s[cut..];
    let mut current: Option<(bool, String)> = None;
    let rb = rest.as_bytes();
    let mut i = 0;
    while i < rb.len() {
        let at_sign = (rb[i] == b'+' || rb[i] == b'-')
            && (i == 0 || (i >= 2 && rb[i - 1] == b' ' && rb[i - 2] == b' '))
            && rb.get(i + 1) == Some(&b' ');
        if at_sign {
            if let Some((c, t)) = current.take() {
                opts.push(ChoiceOption {
                    text: t.trim().into(),
                    correct: c,
                });
            }
            current = Some((rb[i] == b'+', String::new()));
            i += 1;
            continue;
        }
        let ch = rest[i..].chars().next().unwrap_or(' ');
        if let Some((_, t)) = current.as_mut() {
            t.push(ch);
        }
        i += ch.len_utf8();
    }
    if let Some((c, t)) = current {
        opts.push(ChoiceOption {
            text: t.trim().into(),
            correct: c,
        });
    }
    (s[..cut].trim().into(), opts)
}

pub fn parse(script: &str) -> Dialogue {
    // Saved item text is HTML-sanitized (`>` → `&gt;`); the script markup needs the raw chars.
    let script = crate::utils::decode_html_entities(script);
    let script = script.as_str();
    let mut d = Dialogue::default();
    let mut genders: HashMap<String, String> = HashMap::new();
    let mut in_questions = false;
    for (n, raw) in script.lines().enumerate() {
        let line_no = n + 1;
        let t = raw.trim();
        if t.is_empty() {
            continue;
        }
        macro_rules! err {
            ($m:expr) => {
                d.errors.push(ParseIssue {
                    line: line_no,
                    message: $m,
                })
            };
        }
        if t == "---" {
            in_questions = true;
            continue;
        }
        if !in_questions {
            if let Some(title) = t.strip_prefix('#') {
                d.title = Some(title.trim().into());
                continue;
            }
            if let Some(g) = t.strip_prefix('~') {
                match d.lines.last_mut() {
                    Some(l) => l.gloss = Some(g.trim().into()),
                    None => err!("gloss (~) before any line".into()),
                }
                continue;
            }
            match parse_speech(t, &mut genders) {
                Ok(Some(sp)) if sp.text.is_empty() => err!("empty line text".into()),
                Ok(Some(sp)) => d.lines.push(DialogueLine {
                    index: d.lines.len(),
                    speaker: sp.speaker,
                    gender: sp.gender,
                    voice_override: sp.voice,
                    text: sp.text,
                    gloss: None,
                }),
                Ok(None) => err!("expected `M:`, `F:`, `Name(F):`, `[Voice]` or `>` prefix".into()),
                Err(e) => err!(e),
            }
            continue;
        }
        // Question section.
        if let Some(opt) = t.strip_prefix('+').or_else(|| t.strip_prefix('-')) {
            let correct = t.starts_with('+');
            match d.questions.last_mut() {
                Some(q) if q.kind != "fill" => q.options.push(ChoiceOption {
                    text: opt.trim().into(),
                    correct: correct && q.kind == "choice",
                }),
                _ => err!("option without a `? choice:` / `? bank:` question".into()),
            }
            continue;
        }
        let Some(body) = t.strip_prefix('?') else {
            err!("expected `? fill:`, `? bank:`, `? choice:` or an option line".into());
            continue;
        };
        let body = body.trim();
        let Some(colon) = body.find(':') else {
            err!("expected `? kind: …`".into());
            continue;
        };
        let kind = body[..colon].trim().to_ascii_lowercase();
        let rest = body[colon + 1..].trim();
        let index = d.questions.len();
        match kind.as_str() {
            "fill" | "bank" => {
                let (rest, chips) = match rest.rfind("(chips:") {
                    Some(i) if rest.ends_with(')') => (
                        rest[..i].trim(),
                        rest[i + 7..rest.len() - 1]
                            .split(',')
                            .map(|c| c.trim())
                            .filter(|c| !c.is_empty())
                            .map(|c| ChoiceOption {
                                text: c.into(),
                                correct: false,
                            })
                            .collect(),
                    ),
                    _ => (rest, vec![]),
                };
                let mut g = genders.clone();
                let (speaker, text) = match parse_speech(rest, &mut g) {
                    Ok(Some(sp)) if sp.speaker != NARRATOR || rest.starts_with('>') => (
                        if sp.speaker == NARRATOR {
                            String::new()
                        } else {
                            sp.speaker
                        },
                        sp.text,
                    ),
                    _ => (String::new(), rest.to_string()),
                };
                match split_blank(&text) {
                    Ok((before, answers, after)) => d.questions.push(DialogueQuestion {
                        index,
                        kind,
                        prompt: speaker,
                        before,
                        after,
                        answers,
                        options: chips,
                    }),
                    Err(e) => err!(e),
                }
            }
            "choice" => {
                let (prompt, options) = split_inline_options(rest);
                d.questions.push(DialogueQuestion {
                    index,
                    kind,
                    prompt,
                    before: String::new(),
                    after: String::new(),
                    answers: vec![],
                    options,
                });
            }
            other => err!(format!(
                "unknown question kind `{other}` (fill, bank, choice)"
            )),
        }
    }
    for q in &d.questions {
        if q.kind == "choice" {
            let right = q.options.iter().filter(|o| o.correct).count();
            if q.options.len() < 2 || right != 1 {
                d.errors.push(ParseIssue {
                    line: 0,
                    message: format!(
                        "question {}: a choice needs 2+ options and exactly one `+`",
                        q.index + 1
                    ),
                });
            }
        }
    }
    if d.lines.is_empty() {
        d.errors.push(ParseIssue {
            line: 0,
            message: "no dialogue lines".into(),
        });
    }
    d
}

/// Stable speaker → voice map. Saved entries win; new speakers take the next free voice of
/// their gender (cycling when all are used); unknown gender / narrator take any free voice.
pub fn assign_voices(d: &Dialogue, saved: &HashMap<String, String>) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for l in &d.lines {
        if let Some(v) = saved.get(&l.speaker).and_then(|v| canonical_voice(v)) {
            map.insert(l.speaker.clone(), v.to_string());
        }
    }
    for l in &d.lines {
        if map.contains_key(&l.speaker) {
            continue;
        }
        let pool: &[&str] = match l.gender.as_deref() {
            Some("m") => &MALE_VOICES,
            Some("f") => &FEMALE_VOICES,
            _ => &VOICES,
        };
        let used: Vec<&String> = map.values().collect();
        let voice = l
            .voice_override
            .clone()
            .or_else(|| {
                pool.iter()
                    .find(|v| !used.iter().any(|u| u == *v))
                    .map(|v| v.to_string())
            })
            .unwrap_or_else(|| {
                let n = map.values().filter(|u| pool.contains(&u.as_str())).count();
                pool[n % pool.len()].to_string()
            });
        map.insert(l.speaker.clone(), voice);
    }
    map
}

/// Voice for one line (override, else speaker's voice).
pub fn line_voice(l: &DialogueLine, voices: &HashMap<String, String>) -> String {
    l.voice_override
        .clone()
        .or_else(|| voices.get(&l.speaker).cloned())
        .unwrap_or_else(|| "Martin".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "# coi\n> .i la .djan. cu zvati lo zdani\nDjan(M): coi .marian.\n~ hi Marian\nMarian(F): lo mi ckiku cu zvati ma\nDjan: lo do ckiku cu cpana lo jubme\n[Eva] .ie ki'e\n---\n? fill: Marian(F): lo mi [ckiku|ckiku be mi] cu zvati ma\n? choice: Where are the keys?  + on the table  - in the car\n? bank: lo do [ckiku] cu cpana lo jubme (chips: karce, ckafi)\n? choice: Who is tired?\n+ Marian\n- Djan\n";

    #[test]
    fn decodes_sanitized_markup() {
        let d = parse("&gt; .i coi\nM: ko&#39;a\n");
        assert!(d.errors.is_empty(), "{:?}", d.errors);
        assert_eq!(d.lines[0].speaker, NARRATOR);
        assert_eq!(d.lines[1].text, "ko'a");
    }

    #[test]
    fn parses_lines_glosses_and_questions() {
        let d = parse(SAMPLE);
        assert!(d.errors.is_empty(), "{:?}", d.errors);
        assert_eq!(d.title.as_deref(), Some("coi"));
        assert_eq!(d.lines.len(), 5);
        assert_eq!(d.lines[0].speaker, NARRATOR);
        assert_eq!(d.lines[1].gloss.as_deref(), Some("hi Marian"));
        assert_eq!(d.lines[3].gender.as_deref(), Some("m"), "gender remembered");
        assert_eq!(d.lines[4].voice_override.as_deref(), Some("Eva"));
        assert_eq!(d.questions.len(), 4);
        let f = &d.questions[0];
        assert_eq!((f.kind.as_str(), f.prompt.as_str()), ("fill", "Marian"));
        assert_eq!(f.before, "lo mi");
        assert_eq!(f.after, "cu zvati ma");
        assert_eq!(f.answers, vec!["ckiku", "ckiku be mi"]);
        let c = &d.questions[1];
        assert_eq!(c.prompt, "Where are the keys?");
        assert_eq!(c.options.len(), 2);
        assert!(c.options[0].correct && !c.options[1].correct);
        assert_eq!(d.questions[2].options.len(), 2);
        assert_eq!(d.questions[3].options.len(), 2);
    }

    #[test]
    fn reports_errors_with_line_numbers() {
        let d = parse(
            "Djan(M): coi\nno prefix here\n[Zork] hi\n---\n? fill: no blank\n? choice: x  + a\n",
        );
        let lines: Vec<usize> = d.errors.iter().map(|e| e.line).collect();
        assert!(
            lines.contains(&2) && lines.contains(&3) && lines.contains(&5),
            "{:?}",
            d.errors
        );
        assert!(d.errors.iter().any(|e| e.message.contains("exactly one")));
    }

    #[test]
    fn voices_are_stable_and_gendered() {
        let d = parse("A(M): coi\nB(F): coi\nC(M): coi\nD(F): coi\nE(M): coi\n> narrator\n");
        let v = assign_voices(&d, &HashMap::new());
        assert_eq!(v["A"], "Martin");
        assert_eq!(v["B"], "Victoria");
        assert_eq!(v["C"], "Bernd");
        assert_eq!(v["D"], "Eva");
        assert!(MALE_VOICES.contains(&v["E"].as_str()));
        assert!(VOICES.contains(&v[NARRATOR].as_str()));
        // Saved map wins even when the order of first appearance changes.
        let mut saved = HashMap::new();
        saved.insert("B".to_string(), "Eva".to_string());
        let d2 = parse("B(F): coi\nA(M): coi\n");
        let v2 = assign_voices(&d2, &saved);
        assert_eq!(v2["B"], "Eva");
        assert_eq!(v2["A"], "Martin");
        assert_eq!(assign_voices(&d2, &v2), v2, "idempotent");
    }

    #[test]
    fn anonymous_m_f_prefixes() {
        let d = parse("M: coi\nF: coi\nM: co'o\n");
        assert!(d.errors.is_empty());
        let v = assign_voices(&d, &HashMap::new());
        assert_eq!((v["M"].as_str(), v["F"].as_str()), ("Martin", "Victoria"));
        assert_eq!(line_voice(&d.lines[2], &v), "Martin");
    }
}
