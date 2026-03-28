#!/usr/bin/env python3
"""
Build src/assistant/core_reference_dictionary.txt from archive/dict TSVs.

- Full gismu.tsv and cmavo.tsv: all gismu (score order, metrology powers-of-ten after others);
  all cmavo grouped by **selma'o** (`### SELMAHO`); PA1 digits no…so (0–9) in fixed order within PA1;
  tutorial/teaching notes per selma'o as `# …` comment lines before that class's entries.
- English ↔ Lojban pairs from muplis-database.tsv (same ` ↔ ` separator as word lines):
  grammar-diverse greedy cover with **per-family caps** (attitudinals, vocatives, structural openers),
  preferring new **grammar-tag** coverage and new **gismu** tokens (top corpus-score gismu subset),
  tie-breaking by **low sum of per-token corpus frequencies** in the pool (whole phrase, not just the start),
  and a reserved math- / logic-leaning batch.
- Optional **tutorial** block from Markdown lesson files: core notions + example pairs scraped from Markdown.

Environment:
  ASSISTANT_TUTORIAL_BOOK_DIR — directory with numbered lesson `*.md` (e.g. `1.md` … `13.md`).
  If unset, the build tries LEARN_LOJBAN_BOOK_DIR (legacy), then auto-detects `*/data/pages/en/books/*`
  under sibling directories of this repo when `1.md` is present.

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

# Tutorial example pairs cap (diversity-selected) when Markdown lessons are available.
TUTORIAL_EXAMPLE_QUOTA = 40

# Single column separator for every data line (valsi↔gloss, English↔Lojban). U+2194, spaced.
COL_SEP = " ↔ "

# One-line notions: course-wide themes not tied to a single selma'o (particle detail lives under **## cmavo**).
TUTORIAL_NOTION_LINES: list[str] = [
    f"Using this dictionary{COL_SEP}Morphology (alphabet, brivla, cmavo, cmevla), bridi shape, and most particle usage are explained under **### selma'o** headings inside **## cmavo** (teaching notes as `#` lines before each class).",
    f"Alphabet{COL_SEP}Latin letters; ' = /h/ between vowels; . = pause before vowel-initial words; stress on the second-to-last vowel.",
    f"selbrivla (brivla){COL_SEP}Relation/content word: consonant cluster within first five sounds, ends in a vowel (e.g. gleki, klama).",
    f"cmavo{COL_SEP}Particle: consonant + vowel (+ optional 'V sequences); may be run together (e.g. lenu, naku); grammar class = **selma'o** (subsection titles in **## cmavo**).",
    f"cmevla{COL_SEP}Name word: ends in a consonant; often written .name. with pauses matching dots.",
    f"Bridi{COL_SEP}A clause: sumti fill numbered places x₁ x₂ … of one selbri (relation).",
]

# Per-selma'o teaching notes (`# …` lines) below each `### SELMAHO` heading before word rows.
_SELMAHO_FAhA_LEARN: list[str] = [
    "# Teaching note: spatial/directional modals (FAhA); combine with VA distance (vi/va/vu) the way PU time combines with ZI (zi/za/zu).",
    "# With a following sumti, location/direction is relative to that argument; bare, relative to the speech situation.",
]

SELMAHO_NOTE_LINES: dict[str, list[str]] = {
    "PA1": [
        "# PA1: decimal digit cmavo (no=0 … so=9); combine with higher PA selma'o for larger numbers and grammar.",
    ],
    "BAI": [
        "# Modal tags (BAI, …): attach circumstances like cause, amount, or situation to the bridi; they are not ordinary numbered sumti places unless linked with FA/fi'o.",
        "# Without a following sumti, many modals anchor to the speaker's here/now; with a sumti, relative to that event or state (e.g. la'u for extent).",
        "# Teaching note: read strings of tense/location modals left-to-right as an imaginary journey; order changes scope (e.g. mo'u co'a vs co'a mo'u; pu ba vs ba pu).",
        "# Modal particles do not remove fa–fe–fi–fo–fu; place tags and modals apply together (e.g. mi klama se ka'a le rirxe le dinju still fills klama's places).",
        "# fau: in the event of / in the same situation as… (non-causal modal nuance).",
    ],
    "BE": [
        "# Links a sumti into a tanru or description (typically the x₂ of the head selbri); bei chains further linked sumti; be'o closes the substructure.",
    ],
    "CAI": [
        "# Intensity scalars on attitudinal roots: cu'i neutral/middle; sai stronger; cai extreme; ru'e weak (many introductory courses stress cu'i, nai, sai).",
        "# pei: turns the cluster into an attitudinal question (how do you feel?); bare pei invites an attitudinal reply.",
    ],
    "CEhE": [
        "# ce'e: afterthought termset marker—joins terms (e.g. modals) at the same level so they are not nested under the default left-to-right journey reading.",
    ],
    "COI": [
        "# Vocatives: greetings, thanks, hailing (coi, co'o, ki'e, …); take optional names or sumti after.",
        "# Vocatives use the same CAI intensity scalars as interjections (e.g. ki'e sai = thank you very much).",
        "# Put sentence-wide interjections before the vocative; an interjection right after the vocative scopes that vocative or its argument (typical introductory lesson 1 pattern).",
    ],
    "CU": [
        "# Separates the sumti cluster from the selbri when needed; often dropped before nu/du'u/ka abstractions used as the selbri.",
    ],
    "CUhE": [
        "# nau: resets tense/space to the speaker's here-and-now—useful inside nested bridi (English-style sequence of tenses).",
    ],
    "FA": [
        "# fa fe fi fo fu: tag sumti to numbered places x₁–x₅; reorder freely without changing the underlying place structure (contrast SE conversion).",
        "# After be/bei, FA tags mark places inside the linked sumti (e.g. le klama be fi le tcadu).",
    ],
    "GI": [
        "# Forethought logical connectives: gi marks the branch between connected pieces (see also JA for tanru-internal afterthought connectives).",
    ],
    "GOI": [
        "# goi, ne, no'u, …: assign or resume sumti (ko'a-series, Latin-style 'i.e.' bridges); ge'u may close the relative phrase.",
    ],
    "GOhA": [
        "# mo: bridi/selbri question—asks what relationship holds (predicate question).",
    ],
    "I": [
        "# .i (I): starts a new sentence in the same discourse; often elided when the speaker changes.",
    ],
    "JA": [
        "# Tanru-internal afterthought connectives (je, ja, …): combine predicates inside a tanru before the outer bridi shape is closed.",
    ],
    "JOI": [
        "# Non-logical connectives between sumti: mixture, sequence, union, etc. (joi, jo'u, ce'o, …).",
    ],
    "KEI": [
        "# kei: closes nu/du'u/ka (etc.) abstractions; often elidable when grammar is clear.",
    ],
    "KOhA7": [
        "# ma: sumti question—asks who/what fills a sumti place.",
    ],
    "KU": [
        "# ku: closes LE/LO descriptions and similar sumti; elidable in many positions.",
    ],
    "LE": [
        "# Non-veridical descriptors (le, le'i, …): 'the one(s) described as…'; close with ku.",
    ],
    "LIhU": [
        "# li'u: closes lu … quotation; often elidable at end of text.",
    ],
    "LO": [
        "# Veridical descriptors (lo, lo'i, …): 'some of those which really are…'.",
    ],
    "LU": [
        "# lu … li'u: grammatical Lojban quotation as a sumti; nestable.",
    ],
    "NA": [
        "# na: contradictory bridi negation (whole bridi in scope); distinct from naku and from na'e-style scalar negation.",
    ],
    "NAI": [
        "# On attitudinal and many other cmavo: nai selects the opposite pole (e.g. ui vs ui nai).",
        "# More broadly: negation-related flip attached to the preceding cmavo (see each word's definition).",
    ],
    "NU": [
        "# nu: event abstraction; du'u: proposition/bridi abstract; ka: property abstract; close with kei where applicable.",
    ],
    "PU": [
        "# pu ca ba: time direction—before / simultaneous with / after; default past / present / future when used alone.",
        "# ca with a sumti: at the same time as that event (present-like relative to that bridi).",
    ],
    "SE": [
        "# se te ve xe: conversion—swap x₁ with x₂…x₅ so another place is in the x₁ slot; **reshuffles** the place structure (unlike FA tags, which only reorder surface sumti).",
        "# In tanru, conversion applies to the immediately following selbri (e.g. se klama emphasizes the goer).",
    ],
    "SEI": [
        "# sei … se'u: incidental metalinguistic bridi commenting on the host utterance.",
    ],
    "UI1": [
        "# Attitudinal interjections: modify the construct immediately before them; at the start of a bridi they scope the whole bridi (typical introductory lesson 1 pattern).",
        "# Cluster shape: attitudinal root (UI1–UI7, etc.) plus optional CAI scalars on the root; optional UI5 suffixes (dai, zo'o, …); suffixes may take CAI/nai too (e.g. ie zo'o nai = agree, not kidding).",
    ],
    "UI5": [
        "# dai: empathy—attribute the feeling to someone else (e.g. ui nai dai).",
        "# zo'o / zo'o nai: humor vs serious tone (I'm joking / not joking).",
    ],
    "UI6": [
        "# xu: yes/no truth-value question (discursive).",
    ],
    "VA": [
        "# vi va vu: spatial distance from the reference—here / medium / far (subjective scale; vowel i–a–u = near/medium/long, like ZI for time).",
    ],
    "VAU": [
        "# vau: closes the sumti of a simple bridi; in compound bridi separates trailing sumti shared across conjuncts.",
    ],
    "ZO": [
        "# zo: quotes exactly one Lojban word immediately after it.",
    ],
    "ZI": [
        "# zi za zu: subjective near/medium/long **time** distance; combine with pu/ba (and ca) for 'how far' in time.",
    ],
}

for _sk in ("FAhA1", "FAhA2", "FAhA3", "FAhA4"):
    SELMAHO_NOTE_LINES[_sk] = list(_SELMAHO_FAhA_LEARN)

# Phrase selection: total lines from muplis, split between grammar diversity and math emphasis.
MUPLIS_PHRASE_TOTAL = 268
# Reserved for high math_phrase_score() lines (numbers, measure, logic, comparison) before greedy fill.
# Kept in proportion to MUPLIS_PHRASE_TOTAL (same ~28.6% share as at 168/48).
MUPLIS_MATH_PHRASE_QUOTA = 77

# Phrase diversity: which gismu tokens count toward "new gismu" coverage (not the full gismu.tsv).
PHRASE_DIVERSITY_GISMU_TOP_N = 150
PHRASE_DIVERSITY_GISMU_ENSURE: frozenset[str] = frozenset({"cusku", "simxu"})

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

def load_cmavo_rows(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as f:
        rows = list(csv.DictReader(f, delimiter="\t"))
    for row in rows:
        try:
            row["_score"] = int((row.get("score") or "0").strip() or "0")
        except ValueError:
            row["_score"] = 0
    return rows


def _sort_cmavo_in_selmaho(sel: str, rows: list[dict[str, str]]) -> None:
    if sel == "PA1":
        rows.sort(
            key=lambda r: CMOVO_DIGIT_ORDER.get((r.get("word") or "").strip().lower(), 1000),
        )
    else:
        rows.sort(key=lambda r: (-r["_score"], (r.get("word") or "").strip()))


def cmavo_grouped_by_selmaho(path: Path) -> tuple[list[tuple[str, list[dict[str, str]]]], int]:
    """Return (ordered list of (selmaho, rows), total cmavo count). PA1 first; other selma'o A–Z."""
    raw = load_cmavo_rows(path)
    buckets: dict[str, list[dict[str, str]]] = {}
    for row in raw:
        sel = (row.get("selmaho") or "").strip() or "(missing)"
        buckets.setdefault(sel, []).append(row)
    keys = sorted(buckets.keys())
    if "PA1" in keys:
        keys.remove("PA1")
        keys.insert(0, "PA1")
    out: list[tuple[str, list[dict[str, str]]]] = []
    n = 0
    for sel in keys:
        grp = buckets[sel]
        _sort_cmavo_in_selmaho(sel, grp)
        out.append((sel, grp))
        n += len(grp)
    return out, n


def read_gismu_rows_prefer_non_numeric(
    path: Path, want: int | None = None
) -> list[dict[str, str]]:
    """Score order with non-metrology gismu before metrology (powers-of-ten) at the tail.
    If `want` is None, return the full TSV."""
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
    if want is None:
        return primary + secondary
    out = primary[:want]
    if len(out) < want:
        out.extend(secondary[: want - len(out)])
    return out[:want]


def load_gismu_word_set(path: Path) -> frozenset[str]:
    """Top `PHRASE_DIVERSITY_GISMU_TOP_N` gismu by corpus score (non-metrology first), plus ensured lemmas."""
    rows = read_gismu_rows_prefer_non_numeric(path, want=PHRASE_DIVERSITY_GISMU_TOP_N)
    words = {
        (r.get("word") or "").strip().lower()
        for r in rows
        if (r.get("word") or "").strip()
    }
    words |= set(PHRASE_DIVERSITY_GISMU_ENSURE)
    return frozenset(words)


def format_word_definition_row(row: dict[str, str]) -> str:
    """Head word ↔ English definition (section implies gismu vs cmavo)."""
    w = (row.get("word") or "").strip()
    d = one_line(row.get("definition") or "")
    return f"{w}{COL_SEP}{d}"


def format_cmavo_selmaho_sections(
    grouped: list[tuple[str, list[dict[str, str]]]],
) -> list[str]:
    lines: list[str] = []
    for sel, rows in grouped:
        lines.append(f"### {sel}")
        lines.append("")
        notes = SELMAHO_NOTE_LINES.get(sel)
        if notes:
            lines.extend(notes)
            lines.append("")
        lines.extend(format_word_definition_row(r) for r in rows)
        lines.append("")
    return lines


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


def bare_token_for_lexicon(tok: str) -> str:
    """Strip common leading/trailing punctuation so `klama` matches `klama,` and `.klama`."""
    t = normalize_apostrophe(tok).lower().strip()
    t = t.lstrip(".")
    while len(t) > 1 and t[-1] in ".,!?;:)]}\"'":
        t = t[:-1]
    return t


def gismu_tokens_in(lo: str, gismu_words: frozenset[str]) -> set[str]:
    """Tokens that appear in `lo` and in the phrase-diversity gismu subset."""
    out: set[str] = set()
    for t in lojban_tokens(lo):
        b = bare_token_for_lexicon(t)
        if b in gismu_words:
            out.add(b)
    return out


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
# Scaled with MUPLIS_PHRASE_TOTAL (from a 168-line baseline) so greedy + filler can keep similar diversity.
_PHRASE_CAP_SCALE = MUPLIS_PHRASE_TOTAL / 168.0


def _scaled_phrase_cap(n: int) -> int:
    return max(1, round(n * _PHRASE_CAP_SCALE))


PHRASE_FAMILY_CAPS: dict[str, int] = {
    "att_want": _scaled_phrase_cap(1),
    "att_obligation": _scaled_phrase_cap(1),
    "att_hope": _scaled_phrase_cap(1),
    "att_attention": _scaled_phrase_cap(2),
    "att_alertness": _scaled_phrase_cap(2),
    "att_effort": _scaled_phrase_cap(2),
    "att_interest": _scaled_phrase_cap(1),
    "att_intent": _scaled_phrase_cap(1),
    "att_pain": _scaled_phrase_cap(1),
    "att_wonder": _scaled_phrase_cap(1),
    "att_sorrow": _scaled_phrase_cap(1),
    "att_insight": _scaled_phrase_cap(1),
    "att_approval": _scaled_phrase_cap(1),
    "att_misc": _scaled_phrase_cap(5),
    "att_request": _scaled_phrase_cap(4),
    "struct_vocative": _scaled_phrase_cap(6),
    "struct_ko": _scaled_phrase_cap(10),
    "struct_lu": _scaled_phrase_cap(6),
    "struct_ganai": _scaled_phrase_cap(10),
    "struct_xu": _scaled_phrase_cap(10),
    "struct_ma": _scaled_phrase_cap(10),
    "struct_le": _scaled_phrase_cap(22),
    "struct_lo": _scaled_phrase_cap(14),
    "struct_mi": _scaled_phrase_cap(22),
    "struct_do": _scaled_phrase_cap(12),
    "struct_la": _scaled_phrase_cap(14),
    "struct_relative": _scaled_phrase_cap(8),
    "struct_neg_head": _scaled_phrase_cap(8),
    "struct_empty": _scaled_phrase_cap(2),
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
    gismu_words: frozenset[str] | None = None,
) -> list[tuple[str, str]]:
    """Greedy grammar + gismu coverage with per-family caps; tie-break by pool token frequency mass."""
    caps = caps or PHRASE_FAMILY_CAPS
    indexed = [(i, en, lo) for i, (en, lo) in enumerate(rows)]
    word_freq = corpus_lojban_word_freq(rows)
    remaining = indexed.copy()
    selected: list[tuple[str, str]] = []
    covered: set[str] = set()
    covered_gismu: set[str] = set()
    seen_lo: set[str] = set()
    fam_counts: Counter[str] = Counter()

    relax = 0
    max_relax = 45

    while len(selected) < n and remaining:
        best_item: tuple[int, str, str] | None = None
        best_key: tuple[int, int, int, int, int, int] | None = None
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
            g_new = (
                len(gismu_tokens_in(_lo, gismu_words) - covered_gismu)
                if gismu_words
                else 0
            )
            mass = phrase_token_freq_mass(_lo, word_freq)
            # Prefer new grammar tags, then new gismu, then rarer tokens in this pool (whole phrase).
            key = (m, g_new, -mass, -fc, -len(_lo), _i)
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
        if gismu_words:
            covered_gismu |= gismu_tokens_in(lo, gismu_words)
        remaining = [x for x in remaining if x[2] not in seen_lo]

    # Fill: low token-frequency mass, then new gismu; same family soft-caps.
    if len(selected) < n:
        filler = [x for x in indexed if x[2] not in seen_lo]
        while len(selected) < n and filler:
            best_fill: tuple[int, str, str] | None = None
            best_fill_key: tuple[int, int, int] | None = None
            for item in filler:
                _i, _en, lo = item
                fam = phrase_family_slot(lo)
                fill_relax = 3 if fam.startswith("att_") else 14
                if fam_counts[fam] >= _family_cap(fam, caps) + fill_relax:
                    continue
                mass = phrase_token_freq_mass(lo, word_freq)
                g_new = (
                    len(gismu_tokens_in(lo, gismu_words) - covered_gismu)
                    if gismu_words
                    else 0
                )
                fk = (mass, -g_new, _i)
                if best_fill_key is None or fk < best_fill_key:
                    best_fill_key = fk
                    best_fill = item
            if best_fill is None:
                break
            _i, en, lo = best_fill
            selected.append((en, lo))
            seen_lo.add(lo)
            fam_counts[phrase_family_slot(lo)] += 1
            if gismu_words:
                covered_gismu |= gismu_tokens_in(lo, gismu_words)
            filler = [x for x in filler if x[2] != lo]

    # Last resort: fill length (should be rare).
    if len(selected) < n:
        for _i, en, lo in sorted(indexed, key=lambda x: x[0]):
            if len(selected) >= n:
                break
            if lo in seen_lo:
                continue
            selected.append((en, lo))
            seen_lo.add(lo)
            if gismu_words:
                covered_gismu |= gismu_tokens_in(lo, gismu_words)

    return selected[:n]


def resolve_tutorial_book_dir() -> Path | None:
    """Markdown lesson directory (numbered `*.md`). Env first, then legacy env, then sibling auto-detect."""
    candidates: list[Path] = []
    for key in ("ASSISTANT_TUTORIAL_BOOK_DIR", "LEARN_LOJBAN_BOOK_DIR"):
        v = os.environ.get(key, "").strip()
        if v:
            candidates.append(Path(v).expanduser())
    try:
        for book_root in sorted(REPO.parent.glob("*/data/pages/en/books/*")):
            if book_root.is_dir() and (book_root / "1.md").is_file():
                candidates.append(book_root)
    except OSError:
        pass
    seen: set[Path] = set()
    for p in candidates:
        try:
            r = p.resolve()
        except OSError:
            continue
        if not r.is_dir() or r in seen:
            continue
        seen.add(r)
        return r
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


def load_tutorial_example_pool(book_dir: Path) -> list[tuple[str, str]]:
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

    gismu = read_gismu_rows_prefer_non_numeric(GISMU_PATH)
    gismu_words = load_gismu_word_set(GISMU_PATH)
    cmavo_grouped, cmavo_n = cmavo_grouped_by_selmaho(CMAVO_PATH)

    muplis_rows: list[tuple[str, str]] = []
    with MUPLIS_PATH.open(newline="", encoding="utf-8") as f:
        r = csv.DictReader(f, delimiter="\t")
        for row in r:
            en = (row.get("English") or row.get("english") or "").strip()
            lo = (row.get("Lojban") or row.get("lojban") or "").strip()
            if en and lo:
                muplis_rows.append((en, lo))

    book_dir = resolve_tutorial_book_dir()
    tutorial_example_pool: list[tuple[str, str]] = []
    if book_dir is not None:
        tutorial_example_pool = load_tutorial_example_pool(book_dir)
        print(f"tutorial Markdown: {book_dir} — raw example pairs {len(tutorial_example_pool)}")
    else:
        print(
            "tutorial Markdown: directory not found (set ASSISTANT_TUTORIAL_BOOK_DIR "
            "or LEARN_LOJBAN_BOOK_DIR, or place lessons under */data/pages/en/books/*/); "
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
        select_muplis_diverse(
            tutorial_example_pool, TUTORIAL_EXAMPLE_QUOTA, gismu_words=gismu_words
        )
        if tutorial_example_pool
        else []
    )

    indexed = [(i, en, lo) for i, (en, lo) in enumerate(muplis_rows)]
    math_quota = min(MUPLIS_MATH_PHRASE_QUOTA, MUPLIS_PHRASE_TOTAL)
    math_phrases, math_lo = select_math_prioritized(indexed, math_quota)
    general_pool = [(en, lo) for en, lo in muplis_rows if lo not in math_lo]
    general_n = MUPLIS_PHRASE_TOTAL - len(math_phrases)
    general_phrases = select_muplis_diverse(
        general_pool, general_n, gismu_words=gismu_words
    )

    lines: list[str] = [
        "# Assistant core reference dictionary (generated)",
        "# Source: archive/dict/{gismu,cmavo,muplis-database}.tsv; optional tutorial lesson Markdown.",
        "#   python3 scripts/build_assistant_core_dictionary.py",
        "# Format: every non-comment data line is `left ↔ right` (Unicode U+2194, spaces).",
        "#   Gismu: valsi ↔ English definition.",
        "#   Cmavo: `### selma'o` subsections; optional `#` teaching note lines; then valsi ↔ English.",
        "#   Tutorial: English↔English notions + English↔Lojban examples (from bundled Markdown when present).",
        "#   Phrase sections: English ↔ Lojban (muplis; grammar-diverse + math-leaning subset).",
        "",
        f"## gismu ({len(gismu)}; full list; by corpus score; metrology powers-of-ten after others)",
        "",
    ]
    lines.extend(format_word_definition_row(row) for row in gismu)
    lines.append("")
    lines.append(
        f"## cmavo ({cmavo_n}; full list by selma'o; PA1 digits 0–9 in order within PA1; "
        f"other classes sorted by corpus score then word)"
    )
    lines.append("")
    lines.extend(format_cmavo_selmaho_sections(cmavo_grouped))
    lines.append("## tutorial (notions; English ↔ English)")
    lines.append("")
    lines.extend(TUTORIAL_NOTION_LINES)
    lines.append("")
    lines.append(
        f"## tutorial (examples; English ↔ Lojban; {len(tutorial_examples)} lines, "
        f"quota {TUTORIAL_EXAMPLE_QUOTA})"
    )
    lines.append("")
    if tutorial_examples:
        for en, lo in tutorial_examples:
            lines.append(f"{en}{COL_SEP}{lo}")
    else:
        lines.append(
            "(No examples bundled: set ASSISTANT_TUTORIAL_BOOK_DIR or LEARN_LOJBAN_BOOK_DIR, "
            "or add auto-discovered */data/pages/en/books/*/1.md.)"
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
