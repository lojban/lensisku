#!/usr/bin/env python3
"""
Build src/assistant/core_reference_dictionary.txt from archive/dict TSVs.

- Top 150 gismu; cmavo = PA1 digits no…so (0–9) in order plus 50 more by `score`.
- English ↔ Lojban pairs from muplis-database.tsv (same ` ↔ ` separator as word lines):
  grammar-diverse greedy cover with **per-family caps** (attitudinals, vocatives, structural openers),
  tie-breaking by **low sum of per-token corpus frequencies** in the pool (whole phrase, not just the start),
  and a reserved math- / logic-leaning batch.
- Optional **learn-lojban** block (lojban.pw book): core notions + example pairs scraped from Markdown.

Environment:
  LEARN_LOJBAN_BOOK_DIR — directory with `1.md` … `13.md` (default: try sibling lojban.pw path).

Run from repo root:
  python3 scripts/build_assistant_core_dictionary.py
"""

from __future__ import annotations

import csv
import os
import re
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
ARCHIVE = REPO / "archive" / "dict"
OUT = REPO / "src" / "assistant" / "core_reference_dictionary.txt"

GISMU_PATH = ARCHIVE / "gismu.tsv"
CMAVO_PATH = ARCHIVE / "cmavo.tsv"
MUPLIS_PATH = ARCHIVE / "muplis-database.tsv"

# learn-lojban (lojban.pw) — example pairs cap (diversity-selected).
TUTORIAL_EXAMPLE_QUOTA = 40

# Single column separator for every data line (valsi↔gloss, English↔Lojban). U+2194, spaced.
COL_SEP = " ↔ "

# One-line notions aligned with the same course (English ↔ English); always included.
TUTORIAL_NOTION_LINES: list[str] = [
    f"Alphabet{COL_SEP}Latin letters; ' = /h/ between vowels; . = pause before vowel-initial words; stress on the second-to-last vowel.",
    f"selbrivla (brivla){COL_SEP}Relation/content word: consonant cluster within first five sounds, ends in a vowel (e.g. gleki, klama).",
    f"cmavo{COL_SEP}Particle: consonant + vowel (+ optional 'V sequences); may be run together (e.g. lenu, naku).",
    f"cmevla{COL_SEP}Name word: ends in a consonant; often written .name. with pauses matching dots.",
    f"Bridi{COL_SEP}A clause: sumti fill numbered places x₁ x₂ … of one selbri (relation).",
    f"cu{COL_SEP}Separates sumti cluster from selbri when needed; not used before nu/du'u/ka abstractions as selbri.",
    f"fa fe fi fo fu (place tags){COL_SEP}Tag sumti to selbri places x₁–x₅; reorder sumti freely; unlike SE conversion they do not reshuffle the place structure; after be they mark places inside a tanru sumti (e.g. le klama be fi le tcadu).",
    f"Modal terms (tense, location, cause, etc.){COL_SEP}Apply to the whole bridi; they do not fill numbered selbri slots—omitted slots are still zo'e unless a place tag assigns them (learn-lojban: ca, fa'a, pu, bu'u, …).",
    f"Modal term without a following sumti{COL_SEP}Locates the event relative to the speaker's here and now (e.g. ba alone = future from now; bu'u alone = at this place).",
    f"Modal term with a following sumti{COL_SEP}Locates the event relative to the event or state in that argument (e.g. ba le nu mi cadzu = after I walk).",
    f"Time and space (same principle){COL_SEP}Temporal distance uses pu/ba with zi/za/zu; spatial distance uses vi/va/vu (short / medium / far); vowel order i–a–u repeats for subjective near/medium/long.",
    f"la'u{COL_SEP}Modal with a sumti stating how far in time or space (e.g. in three days).",
    f"fau, ca, bu'u{COL_SEP}fau: same time, place, or situation as…; ca: at the same time as… (present-like); bu'u: at a place (spatial analogue of ca).",
    f"nau{COL_SEP}At the speaker's time or place—useful in nested bridi so an inner tense can be anchored to the utterance (English-style sequence of tenses).",
    f"Modal terms and place tags together{COL_SEP}Modal terms do not remove fa–fu; both apply (e.g. mi klama se ka'a le rirxe le dinju still fills klama's places).",
    f"Several modal particles (imaginary journey){COL_SEP}Read left to right from implied now/here; order scopes modals (e.g. mo'u co'a vs co'a mo'u); pu ba vs ba pu differs.",
    f"ce'e between modals{COL_SEP}Joins two modal particles at the same level so they are not nested, overriding the default left-to-right journey reading.",
    f"nu / du'u / ka{COL_SEP}Abstractors: event, proposition, property (-ness); closed with kei where applicable.",
    f"lu … li'u{COL_SEP}Quotation: Lojban text as a sumti; nestable.",
    f"zo{COL_SEP}Quotes the single following Lojban word.",
    f"sei{COL_SEP}Incidental bridi commenting on the host sentence.",
    f"ma / mo / xu{COL_SEP}Questions: sumti, selbri, yes/no respectively.",
    f"Attitudinals (interjections), placement{COL_SEP}Modify the construct immediately before them; at the start of a bridi they scope the whole bridi; moving them changes which phrase is in scope (learn-lojban lesson 1).",
    f"Attitudinal structure{COL_SEP}Root cmavo (e.g. ui, ie) plus optional scalar particles on the root; then optional suffixes pei, dai, zo'o; suffixes may take scalars too (e.g. ie zo'o nai = agree, not kidding).",
    f"Attitudinal scalars: cu'i, nai, sai, cai, ru'e{COL_SEP}cu'i = middle/neutral on the scale; nai = opposite pole (ui happy vs ui nai alas); sai = strong intensity; cai = extreme intensity; ru'e = weak intensity (standard Lojban scale; learn-lojban stresses cu'i, nai, sai).",
    f"pei (attitudinal suffix){COL_SEP}Makes the attitudinal a question about the listener's feeling; pei alone asks for an appropriate attitudinal reply.",
    f"dai (attitudinal suffix){COL_SEP}Attributes the feeling to someone else (empathy, e.g. ui nai dai = you must be sad); without dai the speaker expresses their own attitude toward the bridi.",
    f"zo'o / zo'o nai (attitudinal suffix){COL_SEP}zo'o marks humor or non-serious tone; zo'o nai marks seriousness (I'm not joking).",
    f"Vocatives and attitudinal scalars{COL_SEP}Vocatives use the same scalar modifiers as interjections (e.g. ki'e sai = thank you very much).",
    f"Interjections before vocatives (sentence-wide scope){COL_SEP}Put interjections before vocatives when both should modify the whole utterance; an interjection immediately after a vocative modifies that vocative or its argument (learn-lojban lesson 1).",
    f"ku / kei / vau{COL_SEP}Terminators: end LE description, NU abstraction, bridi (when required).",
]

# Phrase selection: total lines from muplis, split between grammar diversity and math emphasis.
MUPLIS_PHRASE_TOTAL = 168
# Reserved for high math_phrase_score() lines (numbers, measure, logic, comparison) before greedy fill.
MUPLIS_MATH_PHRASE_QUOTA = 48

# English hints (regex, weight) for math-like content.
MATH_SCORE_EN: list[tuple[str, int]] = [
    (r"logically equivalent", 10),
    (r"\bprime\b", 7),
    (r"greatest positive integer", 12),
    (r"positive integer", 6),
    (r"square meters?", 7),
    (r"\binteger\b", 4),
    (r"\btheorem\b|\bproof\b|\blemma\b", 6),
    (r"\bgeometry\b|\balgebra\b|\bcalculus\b|\bderivative\b|\bintegral\b", 6),
    (r"\bmatrix\b|\bvector\b|\bcoordinate", 5),
    (r"\bmultiply\b|\bdivid(e|ing)\b|\bsubtract\b", 4),
    (r"\bpercent(age)?\b|\bfraction\b|\bratio\b", 4),
    (r"\binfinity\b|\bnegative\b|\bodd\b|\beven\b", 3),
    (r"\bkm\b|kilomet", 4),
    (r"o'clock|years old", 3),
    (r"days in a week|seven days|five days|three days|ten days|twice a day", 3),
    (r"€|euros?\b", 2),
    (r"distance between", 4),
    (r"\btwo is\b", 6),
    (r"more fun than", 1),
    (r"liter of|dozen eggs", 2),
    (r"\bcost(s|ing)?\b|\bprice\b", 2),
    (r"\b(o'clock|minutes?|hours?|clock)\b", 2),
]

# Lojban hints (regex, weight).
MATH_SCORE_LO: list[tuple[str, int]] = [
    (r"\bli\s+", 6),
    (r"\ble\s+ni\s+|\blo\s+ni\s+", 5),
    (r"ganai\s+.+\sgi.+seni'i|seni'i\s+bo\s+ga", 9),
    (r"\bbroda\b.*\bbrode\b|\bbrode\b.*\bbroda\b", 8),
    (r"namcr(primu|nanba)|mulna'u|nonmau|dubmau|dubme", 8),
    (r"mitre\s+be\s+li|te'ai", 7),
    (r"rupne|rupnu\s+be\s+li", 5),
    (r"se\s+la'u\s+li\s+xo\s+kau", 5),
    (r"djedi\s+be\s+li\s", 3),
    (r"nanca\s+be\s+li\s", 4),
    (r"ti'u\s+li\s", 4),
    (r"cacra\s+be\s+li|mentu\s+be\s+li", 4),
    (r"\bfe'i\b|\btenfa\b|\bsumji\b|\bvamji\b|\bdilcu\b|\bpi'i\b|\bfatne\b", 7),
    (r"ju'u|ju'au", 5),
    (r"le\s+ni\s+.+\s+cu\s+dubmau", 5),
    (r"ronru'u|ki'o", 3),
]

# (tag_id, regex on Lojban text) — broad heuristics for instructional diversity.
GRAMMAR_TAG_PATTERNS: list[tuple[str, re.Pattern[str]]] = [
    ("bridi_sep_i", re.compile(r"(?:^|\s)\.i(?:\s|$)")),
    ("imperative_ko", re.compile(r"(?:^|\s)ko(?:\s|$)")),
    ("question_xu", re.compile(r"(?:^|\s)xu(?:\s|$)")),
    ("question_ma", re.compile(r"(?:^|\s)ma(?:\s|$)")),
    ("sumti_da_de_di", re.compile(r"(?:^|\s)d[aei](?:\s|$)")),
    ("abstraction_nu", re.compile(r"(?:^|\s)nu(?:\s|$)")),
    ("abstraction_duu", re.compile(r"du['’]u")),
    ("abstraction_ka", re.compile(r"(?:^|\s)ka(?:\s|$)")),
    ("relative_noi_poi", re.compile(r"\bnoi\b|\bpoi\b")),
    ("quotes_lu", re.compile(r"\blu\b|\bli['’]u\b")),
    ("names_la", re.compile(r"\bla\s*\.[a-z]")),
    ("articles_le_lo", re.compile(r"(?:^|\s)le(?:\s|$)|(?:^|\s)lo(?:\s|$)")),
    ("cu", re.compile(r"(?:^|\s)cu(?:\s|$)")),
    ("tense_pu", re.compile(r"(?:^|\s)pu(?:\s|$)")),
    ("tense_ca", re.compile(r"(?:^|\s)ca(?:\s|$)")),
    ("tense_ba", re.compile(r"(?:^|\s)ba(?:\s|$)")),
    ("connective_je_ja", re.compile(r"(?:^|\s|\.)je(?:\s|$)|(?:^|\s|\.)ja(?:\s|$)")),
    ("logical_ij", re.compile(r"\.i\s+j|\.i\s+na\.i")),
    ("negation_na", re.compile(r"(?:^|\s)na(?:\s|$)|na['’]e")),
    ("modal_bai_gau", re.compile(r"\bbai\b|\bgau\b|mu['’]i|ki['’]u|ni['’]i")),
    ("be_bei", re.compile(r"\bbe\b|\bbei\b|\bbe['’]o\b")),
    ("fa_family", re.compile(r"(?:^|\s)fa(?:\s|$)|(?:^|\s)fe(?:\s|$)|(?:^|\s)fi(?:\s|$)")),
    ("se_te_ve", re.compile(r"(?:^|\s)se\s+|(?:^|\s)te\s+|(?:^|\s)ve\s+")),
    ("attitudinal", re.compile(r"\.[a-z][a-z'’]*(?:\s|$)")),
    ("vocative_co", re.compile(r"\bco[iou]['’]?\b|\bcoi\b|\bre['’]i\b")),
    ("subscript_xi", re.compile(r"\bxi\s|vo['’]e|ce['’]u")),
    ("zoi_quote", re.compile(r"\bzoi\b")),
    ("numerals_pa", re.compile(r"\bpa\b|\bre\b|\bci\b|\bvo\b|\bmu\b|\bxa\b")),
    ("ke_group", re.compile(r"\bke\b|\bke['’]e\b")),
    ("gi_connective", re.compile(r"\bgi\b")),
    ("joi_je", re.compile(r"\bjoi\b|\bjo['’]u\b|\bce['’]o\b")),
    ("fi_fi", re.compile(r"\bfi['’]o\b|\bfi'o\b")),
    ("nahe", re.compile(r"na['’]e|to['’]e|je['’]a")),
    ("ui_discursive", re.compile(r"\.ui\b|\.ua\b|\.uu\b|\.oi\b")),
    ("zo_quote", re.compile(r"\bzo\b")),
    ("ri_ra", re.compile(r"\bri\b|\bra\b|\bru\b")),
    ("fi_o_modal", re.compile(r"\bfi['’]o\b|\bfi'o\b")),
    ("tu_a_raising", re.compile(r"\btu['’]a\b|\btu'a\b")),
    ("jai_conversion", re.compile(r"\bjai\b")),
    ("co_tanru", re.compile(r"\bco\b")),
    ("pe_ne_po", re.compile(r"\bpe\b|\bne\b|\bpo\b|\bpo'e\b|\bge'u\b")),
    ("goi_assign", re.compile(r"\bgoi\b|\bnei\b|\bno'u\b")),
    ("ce_u_mass", re.compile(r"\bce['’]u\b|\bce'u\b")),
    ("tu_o_output", re.compile(r"\btu['’]o\b|\btu'o\b")),
    ("si_erase", re.compile(r"\bsi\b|\bsa\b|\bsu\b")),
    ("za_hi_tense", re.compile(r"\bza['’]u\b|\bze['’]a\b|\bzu['’]a\b")),
    ("vu_hi_space", re.compile(r"\bvu\b|\bvi\b|\bva\b|\bvu'o\b")),
    ("mo_interjection", re.compile(r"(?:^|\s)mo(?:\s|$)")),
    ("xu_kau_indir", re.compile(r"xu\s+kau|kau\s+xu")),
    ("du_metaling", re.compile(r"\bdu\s+da\b|\bme\s+li\b")),
    ("toi_sub_bridi", re.compile(r"\btoi\b|\bto\s+da\b")),
    ("sei_discourse", re.compile(r"\bsei\b")),
    ("zoq_quote", re.compile(r"zo['’]i\b|lo'u\b|le'u\b")),
    ("termsets", re.compile(r"\bce'e\b|\bnu'i\b")),
]


def one_line(s: str) -> str:
    return " ".join(s.replace("\n", " ").split())


def _is_numeric_scale_gismu(definition: str) -> bool:
    """High-score metrology gismu (powers of ten); deprioritize for learner-focused core list."""
    d = definition
    return "$10^" in d or "$10^{" in d or "10^{-" in d


def read_scored_rows(path: Path, want: int) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as f:
        r = csv.DictReader(f, delimiter="\t")
        rows = list(r)
    for row in rows:
        try:
            row["_score"] = int((row.get("score") or "0").strip() or "0")
        except ValueError:
            row["_score"] = 0
    rows.sort(key=lambda x: (-x["_score"], x.get("word", "")))
    return rows[:want]


# Decimal digit cmavo (selma'o PA1): fixed order no=0 … so=9.
CMOVO_DIGIT_ORDER: dict[str, int] = {
    "no": 0,
    "pa": 1,
    "re": 2,
    "ci": 3,
    "vo": 4,
    "mu": 5,
    "xa": 6,
    "ze": 7,
    "bi": 8,
    "so": 9,
}

# After the PA1 block, how many additional cmavo to take by corpus score.
CMAVO_TOP_AFTER_DIGITS = 50


def read_cmavo_pa1_digits(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as f:
        rows = list(csv.DictReader(f, delimiter="\t"))
    digit_rows = [r for r in rows if (r.get("selmaho") or "").strip() == "PA1"]
    digit_rows.sort(
        key=lambda r: CMOVO_DIGIT_ORDER.get((r.get("word") or "").strip().lower(), 100),
    )
    return digit_rows


def read_cmavo_core(path: Path) -> list[dict[str, str]]:
    """PA1 digits 0–9 (archive order), then top `CMAVO_TOP_AFTER_DIGITS` other cmavo by score."""
    digit_rows = read_cmavo_pa1_digits(path)
    digit_words = {(r.get("word") or "").strip().lower() for r in digit_rows}
    with path.open(newline="", encoding="utf-8") as f:
        rows = list(csv.DictReader(f, delimiter="\t"))
    for row in rows:
        try:
            row["_score"] = int((row.get("score") or "0").strip() or "0")
        except ValueError:
            row["_score"] = 0
    rows.sort(key=lambda x: (-x["_score"], x.get("word", "")))
    more: list[dict[str, str]] = []
    for r in rows:
        w = (r.get("word") or "").strip().lower()
        if w in digit_words:
            continue
        more.append(r)
        if len(more) >= CMAVO_TOP_AFTER_DIGITS:
            break
    return digit_rows + more


def read_gismu_rows_prefer_non_numeric(path: Path, want: int) -> list[dict[str, str]]:
    """Same as score order, but fill with non-metrology gismu first (still all from same TSV)."""
    with path.open(newline="", encoding="utf-8") as f:
        r = csv.DictReader(f, delimiter="\t")
        rows = list(r)
    for row in rows:
        try:
            row["_score"] = int((row.get("score") or "0").strip() or "0")
        except ValueError:
            row["_score"] = 0
    rows.sort(key=lambda x: (-x["_score"], x.get("word", "")))
    primary = [x for x in rows if not _is_numeric_scale_gismu(x.get("definition") or "")]
    secondary = [x for x in rows if _is_numeric_scale_gismu(x.get("definition") or "")]
    out = primary[:want]
    if len(out) < want:
        out.extend(secondary[: want - len(out)])
    return out[:want]


def format_word_definition_row(row: dict[str, str]) -> str:
    """Head word ↔ English definition (section implies gismu vs cmavo)."""
    w = (row.get("word") or "").strip()
    d = one_line(row.get("definition") or "")
    return f"{w}{COL_SEP}{d}"


def tags_for_lojban(lo: str) -> set[str]:
    out: set[str] = set()
    for tid, pat in GRAMMAR_TAG_PATTERNS:
        if pat.search(lo):
            out.add(tid)
    return out


def normalize_apostrophe(s: str) -> str:
    for c in "\u2019\u2018’‘":
        s = s.replace(c, "'")
    return s


def first_content_token(lo: str) -> str:
    lo = normalize_apostrophe(lo)
    toks = lo.split()
    i = 0
    while i < len(toks) and toks[i] == ".i":
        i += 1
    return toks[i] if i < len(toks) else ""


# Longest match first: (Lojban prefix, stable family id for capping).
_ATT_PREFIXES: list[tuple[str, str]] = sorted(
    [
        (".a'onaicai", "att_hope"),
        (".a'onai", "att_hope"),
        (".a'o", "att_hope"),
        (".aunai", "att_want"),
        (".au", "att_want"),
        (".e'enai", "att_request"),
        (".e'e", "att_request"),
        (".e'unai", "att_request"),
        (".e'u", "att_request"),
        (".e'onai", "att_request"),
        (".e'o", "att_request"),
        (".einai", "att_obligation"),
        (".ei", "att_obligation"),
        (".a'acu'i", "att_attention"),
        (".a'anai", "att_attention"),
        (".a'a", "att_attention"),
        (".a'ecai", "att_alertness"),
        (".a'enai", "att_alertness"),
        (".a'e", "att_alertness"),
        (".a'icai", "att_effort"),
        (".a'inai", "att_effort"),
        (".a'i", "att_effort"),
        (".a'ucai", "att_interest"),
        (".a'unai", "att_interest"),
        (".a'u", "att_interest"),
        (".ainai", "att_intent"),
        (".ai", "att_intent"),
        (".oinai", "att_pain"),
        (".oi", "att_pain"),
        (".uinai", "att_wonder"),
        (".ui", "att_wonder"),
        (".uucai", "att_sorrow"),
        (".uu", "att_sorrow"),
        (".uanai", "att_insight"),
        (".ua", "att_insight"),
        (".iu", "att_approval"),
    ],
    key=lambda x: -len(x[0]),
)


def attitudinal_family(first_tok: str) -> str | None:
    raw = normalize_apostrophe(first_tok)
    t = raw.lower()
    # jb2en-style lines sometimes omit the leading dot on UI / request cmavo.
    if re.match(r"^e'o", t) or re.match(r"^e'e", t):
        return "att_request"
    if not raw.startswith("."):
        return None
    for pref, fam in _ATT_PREFIXES:
        if t.startswith(pref.lower()):
            return fam
    if len(t) >= 2:
        return "att_misc"
    return None


def phrase_family_slot(lo: str) -> str:
    tok = first_content_token(lo)
    if not tok:
        return "struct_empty"
    att = attitudinal_family(tok)
    if att is not None:
        return att
    t = normalize_apostrophe(tok).lower()
    if re.match(r"^coi", t) or re.match(r"^co'o", t) or t == "doi" or t.startswith("doi"):
        return "struct_vocative"
    if t == "ko":
        return "struct_ko"
    if t == "lu" or t.startswith("lu"):
        return "struct_lu"
    if t.startswith("ganai"):
        return "struct_ganai"
    if t == "xu":
        return "struct_xu"
    if t == "ma":
        return "struct_ma"
    if t in ("le", "lei", "le'i"):
        return "struct_le"
    if t in ("lo", "loi"):
        return "struct_lo"
    if t == "mi":
        return "struct_mi"
    if t == "do":
        return "struct_do"
    if t.startswith("la"):
        return "struct_la"
    if t.startswith("noi") or t.startswith("poi"):
        return "struct_relative"
    if t in ("no", "na", "na'i", "nago'i", "naku"):
        return "struct_neg_head"
    return "struct_other"


def lojban_tokens(lo: str) -> list[str]:
    """Whitespace-separated tokens, lowercased (corpus frequency is over whole phrases)."""
    lo = normalize_apostrophe(lo)
    return [t.lower() for t in lo.split() if t]


def corpus_lojban_word_freq(rows: list[tuple[str, str]]) -> Counter[str]:
    wf: Counter[str] = Counter()
    for _, lo in rows:
        for t in lojban_tokens(lo):
            wf[t] += 1
    return wf


def phrase_token_freq_mass(lo: str, wf: Counter[str]) -> int:
    """Sum of corpus occurrence counts for each token in this phrase; lower => rarer words overall."""
    return sum(wf.get(t, 0) for t in lojban_tokens(lo))


# Per-family ceilings so one construction (e.g. .au) cannot dominate the phrase list.
PHRASE_FAMILY_CAPS: dict[str, int] = {
    "att_want": 1,
    "att_obligation": 1,
    "att_hope": 1,
    "att_attention": 2,
    "att_alertness": 2,
    "att_effort": 2,
    "att_interest": 1,
    "att_intent": 1,
    "att_pain": 1,
    "att_wonder": 1,
    "att_sorrow": 1,
    "att_insight": 1,
    "att_approval": 1,
    "att_misc": 5,
    "att_request": 4,
    "struct_vocative": 6,
    "struct_ko": 10,
    "struct_lu": 6,
    "struct_ganai": 10,
    "struct_xu": 10,
    "struct_ma": 10,
    "struct_le": 22,
    "struct_lo": 14,
    "struct_mi": 22,
    "struct_do": 12,
    "struct_la": 14,
    "struct_relative": 8,
    "struct_neg_head": 8,
    "struct_empty": 2,
    "struct_other": 9999,
}


def _family_cap(fam: str, caps: dict[str, int]) -> int:
    if fam in caps:
        return caps[fam]
    if fam.startswith("att_misc"):
        return caps.get("att_misc", 5)
    return caps.get("struct_other", 9999)


def math_phrase_score(en: str, lo: str) -> int:
    """Heuristic score for measurement, arithmetic, formal logic, and comparatives in Lojban."""
    s = 0
    el = en.lower()
    for pat, w in MATH_SCORE_EN:
        if re.search(pat, el, re.I):
            s += w
    for pat, w in MATH_SCORE_LO:
        if re.search(pat, lo, re.I):
            s += w
    return s


def select_math_prioritized(
    indexed: list[tuple[int, str, str]], quota: int
) -> tuple[list[tuple[str, str]], set[str]]:
    """Pick up to `quota` pairs with highest math_phrase_score (stable by file order)."""
    scored = [(math_phrase_score(en, lo), i, en, lo) for i, en, lo in indexed]
    scored.sort(key=lambda x: (-x[0], x[1]))
    out: list[tuple[str, str]] = []
    seen_lo: set[str] = set()

    def drain(min_sc: int) -> None:
        for sc, _i, en, lo in scored:
            if len(out) >= quota:
                return
            if sc < min_sc or lo in seen_lo:
                continue
            out.append((en, lo))
            seen_lo.add(lo)

    drain(4)
    drain(2)
    drain(1)
    # Filler: number `li`, measure, money, comparison (avoid generic `ganai` politeness).
    if len(out) < quota:
        weak = re.compile(
            r"\bli\s+|mitre\b|te'ai\b|rupne|dubmau|dubme|\ble\s+ni\s+"
        )
        extra = [(i, en, lo) for i, en, lo in indexed if lo not in seen_lo and weak.search(lo)]
        extra.sort(key=lambda x: x[0])
        for _i, en, lo in extra:
            if len(out) >= quota:
                break
            out.append((en, lo))
            seen_lo.add(lo)
    return out, seen_lo


def select_muplis_diverse(
    rows: list[tuple[str, str]],
    n: int,
    *,
    caps: dict[str, int] | None = None,
) -> list[tuple[str, str]]:
    """Greedy grammar coverage with per-family caps; tie-break by low sum of token corpus frequencies."""
    caps = caps or PHRASE_FAMILY_CAPS
    indexed = [(i, en, lo) for i, (en, lo) in enumerate(rows)]
    word_freq = corpus_lojban_word_freq(rows)
    remaining = indexed.copy()
    selected: list[tuple[str, str]] = []
    covered: set[str] = set()
    seen_lo: set[str] = set()
    fam_counts: Counter[str] = Counter()

    relax = 0
    max_relax = 45

    while len(selected) < n and remaining:
        best_item: tuple[int, str, str] | None = None
        best_key: tuple[int, int, int, int, int] | None = None
        for item in remaining:
            _i, _en, _lo = item
            if _lo in seen_lo:
                continue
            fam = phrase_family_slot(_lo)
            fc = fam_counts[fam]
            lim = _family_cap(fam, caps) + relax
            if fc >= lim:
                continue
            m = len(tags_for_lojban(_lo) - covered)
            mass = phrase_token_freq_mass(_lo, word_freq)
            # Prefer new grammar tags, then phrases whose tokens are less frequent in this pool
            # (spread attention across the whole sentence, not only the first cmavo).
            key = (m, -mass, -fc, -len(_lo), _i)
            if best_key is None or key > best_key:
                best_key = key
                best_item = item

        if best_item is None:
            relax += 1
            if relax > max_relax:
                break
            continue

        _i, en, lo = best_item
        selected.append((en, lo))
        seen_lo.add(lo)
        fam_counts[phrase_family_slot(lo)] += 1
        covered |= tags_for_lojban(lo)
        remaining = [x for x in remaining if x[2] not in seen_lo]

    # Fill: prefer phrases with lower total token frequency mass in this pool; same family soft-caps.
    if len(selected) < n:
        filler = [x for x in indexed if x[2] not in seen_lo]
        filler.sort(
            key=lambda it: (phrase_token_freq_mass(it[2], word_freq), it[0]),
        )
        for _i, en, lo in filler:
            if len(selected) >= n:
                break
            fam = phrase_family_slot(lo)
            fill_relax = 3 if fam.startswith("att_") else 14
            if fam_counts[fam] >= _family_cap(fam, caps) + fill_relax:
                continue
            selected.append((en, lo))
            seen_lo.add(lo)
            fam_counts[fam] += 1

    # Last resort: fill length (should be rare).
    if len(selected) < n:
        for _i, en, lo in sorted(indexed, key=lambda x: x[0]):
            if len(selected) >= n:
                break
            if lo in seen_lo:
                continue
            selected.append((en, lo))
            seen_lo.add(lo)

    return selected[:n]


def resolve_learn_lojban_book_dir() -> Path | None:
    env = os.environ.get("LEARN_LOJBAN_BOOK_DIR", "").strip()
    candidates: list[Path] = []
    if env:
        candidates.append(Path(env).expanduser())
    candidates.append(
        REPO.parent / "lojban.pw" / "data" / "pages" / "en" / "books" / "learn-lojban",
    )
    for p in candidates:
        try:
            if p.is_dir():
                return p.resolve()
        except OSError:
            continue
    return None


def strip_html_tags(s: str) -> str:
    t = re.sub(r"<[^>]+>", " ", s)
    return " ".join(t.split())


def looks_like_lojban(s: str) -> bool:
    """Heuristic for table columns: prefer recall on textbook Lojban."""
    s = s.strip()
    if len(s) < 2:
        return False
    if re.search(
        r"(^|\s)(\.i|cu|le[io']?|lo[io']?|lei |lai |mi |do |ko |nu |ka |du'u|lu |li'u|noi |poi |sei |xu |ma |mo |fa'a|zo |ni'o|kei|tu'a|jai )",
        s,
        re.I,
    ):
        return True
    if re.match(r"la[\s.]", s, re.I):
        return True
    if s.startswith(".") and len(s) > 1 and s[1].isalpha():
        return True
    return False


def valid_tutorial_pair(lo: str, en: str) -> bool:
    if len(lo) < 2 or len(en) < 4:
        return False
    if "**" in lo or "`" in lo:
        return False
    if "http" in en.lower() or "http" in lo.lower():
        return False
    if en.startswith(("particle ", "relation word", "name word")):
        return False
    return True


# Blockquote pattern: > **Lojban** then > _English_
BLOCKQUOTE_LOJBAN_PAIR = re.compile(
    r"^>[ \t]*\*\*(?P<lo>[^*\n]+)\*\*[ \t]*\r?\n>[ \t]*_(?P<en>[^_]+?)_[ \t]*$",
    re.MULTILINE,
)

TABLE_TD_PAIR = re.compile(
    r"<td>\s*<b>\s*([^<]+?)\s*</b>\s*</td>\s*(?:\n\s*)*<td>\s*(?:<i>)?\s*([^<]+?)\s*(?:</i>)?\s*</td>",
    re.IGNORECASE,
)


def extract_tutorial_pairs_from_markdown(text: str) -> list[tuple[str, str]]:
    pairs: list[tuple[str, str]] = []
    for m in BLOCKQUOTE_LOJBAN_PAIR.finditer(text):
        lo = strip_html_tags(m.group("lo")).strip()
        en = strip_html_tags(m.group("en")).strip()
        if valid_tutorial_pair(lo, en):
            pairs.append((en, lo))
    for m in TABLE_TD_PAIR.finditer(text):
        a = strip_html_tags(m.group(1)).strip()
        b = strip_html_tags(m.group(2)).strip()
        if looks_like_lojban(a) and not looks_like_lojban(b):
            lo, eng = a, b
        elif looks_like_lojban(b) and not looks_like_lojban(a):
            lo, eng = b, a
        else:
            continue
        if valid_tutorial_pair(lo, eng):
            pairs.append((eng, lo))
    return pairs


def iter_tutorial_markdown_files(book_dir: Path) -> list[Path]:
    paths: list[Path] = []
    for pat in ("*.md", "*.mdc"):
        paths.extend(book_dir.glob(pat))

    def sort_key(p: Path) -> tuple[int, str]:
        s = p.stem.lstrip("!")
        if s.isdigit():
            return (0, f"{int(s):04d}")
        return (1, s)

    return sorted(paths, key=sort_key)


def load_learn_lojban_example_pool(book_dir: Path) -> list[tuple[str, str]]:
    seen: set[str] = set()
    out: list[tuple[str, str]] = []
    for path in iter_tutorial_markdown_files(book_dir):
        try:
            raw = path.read_text(encoding="utf-8")
        except OSError:
            continue
        for en, lo in extract_tutorial_pairs_from_markdown(raw):
            key = re.sub(r"\s+", " ", lo.lower())
            if key in seen:
                continue
            seen.add(key)
            out.append((en, lo))
    return out


def main() -> int:
    for p in (GISMU_PATH, CMAVO_PATH, MUPLIS_PATH):
        if not p.is_file():
            print(f"missing: {p}", file=sys.stderr)
            return 1

    gismu = read_gismu_rows_prefer_non_numeric(GISMU_PATH, 150)
    cmavo = read_cmavo_core(CMAVO_PATH)

    muplis_rows: list[tuple[str, str]] = []
    with MUPLIS_PATH.open(newline="", encoding="utf-8") as f:
        r = csv.DictReader(f, delimiter="\t")
        for row in r:
            en = (row.get("English") or row.get("english") or "").strip()
            lo = (row.get("Lojban") or row.get("lojban") or "").strip()
            if en and lo:
                muplis_rows.append((en, lo))

    book_dir = resolve_learn_lojban_book_dir()
    tutorial_example_pool: list[tuple[str, str]] = []
    if book_dir is not None:
        tutorial_example_pool = load_learn_lojban_example_pool(book_dir)
        print(f"learn-lojban: {book_dir} — raw example pairs {len(tutorial_example_pool)}")
    else:
        print(
            "learn-lojban: directory not found (set LEARN_LOJBAN_BOOK_DIR); "
            "notions only, no scraped examples",
            file=sys.stderr,
        )

    tutorial_lo_keys = {
        re.sub(r"\s+", " ", lo.lower()) for _, lo in tutorial_example_pool
    }
    if tutorial_lo_keys:
        muplis_rows = [
            (e, l)
            for e, l in muplis_rows
            if re.sub(r"\s+", " ", l.lower()) not in tutorial_lo_keys
        ]

    tutorial_examples = (
        select_muplis_diverse(tutorial_example_pool, TUTORIAL_EXAMPLE_QUOTA)
        if tutorial_example_pool
        else []
    )

    indexed = [(i, en, lo) for i, (en, lo) in enumerate(muplis_rows)]
    math_quota = min(MUPLIS_MATH_PHRASE_QUOTA, MUPLIS_PHRASE_TOTAL)
    math_phrases, math_lo = select_math_prioritized(indexed, math_quota)
    general_pool = [(en, lo) for en, lo in muplis_rows if lo not in math_lo]
    general_n = MUPLIS_PHRASE_TOTAL - len(math_phrases)
    general_phrases = select_muplis_diverse(general_pool, general_n)

    lines: list[str] = [
        "# Assistant core reference dictionary (generated)",
        "# Source: archive/dict/{gismu,cmavo,muplis-database}.tsv; optional learn-lojban book Markdown.",
        "#   python3 scripts/build_assistant_core_dictionary.py",
        "# Format: every non-comment data line is `left ↔ right` (Unicode U+2194, spaces).",
        "#   Gismu section: valsi ↔ English definition. Cmavo section: valsi ↔ English definition.",
        "#   learn-lojban: English↔English notions + English↔Lojban examples (lojban.pw course).",
        "#   Phrase sections: English ↔ Lojban (muplis; grammar-diverse + math-leaning subset).",
        "",
        "## gismu (150; by corpus score; metrology powers-of-ten entries filled only after others)",
        "",
    ]
    lines.extend(format_word_definition_row(row) for row in gismu)
    lines.append("")
    lines.append(
        f"## cmavo ({len(cmavo)}; PA1 digits 0–9 then top {CMAVO_TOP_AFTER_DIGITS} by corpus score)"
    )
    lines.append("")
    lines.extend(format_word_definition_row(row) for row in cmavo)
    lines.append("")
    lines.append(
        "## learn-lojban tutorial (notions; English ↔ English, from lojban.pw course themes)"
    )
    lines.append("")
    lines.extend(TUTORIAL_NOTION_LINES)
    lines.append("")
    lines.append(
        f"## learn-lojban tutorial (examples; English ↔ Lojban; {len(tutorial_examples)} lines, "
        f"quota {TUTORIAL_EXAMPLE_QUOTA})"
    )
    lines.append("")
    if tutorial_examples:
        for en, lo in tutorial_examples:
            lines.append(f"{en}{COL_SEP}{lo}")
    else:
        lines.append(
            f"(No examples bundled: clone lojban.pw pages or set LEARN_LOJBAN_BOOK_DIR.)"
        )
    lines.append("")
    lines.append(
        f"## phrases ({len(general_phrases)}; grammar-diverse sample from muplis-database)"
    )
    lines.append("")
    for en, lo in general_phrases:
        lines.append(f"{en}{COL_SEP}{lo}")
    lines.append("")
    lines.append(
        f"## phrases ({len(math_phrases)}; math-, measure-, and formal-logic leaning from muplis-database)"
    )
    lines.append("")
    for en, lo in math_phrases:
        lines.append(f"{en}{COL_SEP}{lo}")

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Wrote {OUT} ({len(lines)} lines)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
