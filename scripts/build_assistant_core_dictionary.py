#!/usr/bin/env python3
"""
Build src/assistant/assistant_system_prompt.txt — the full assistant system prompt for the LLM.

Static instructions live in scripts/assistant_system_instructions_static.txt (role, tools, guardrails, …).
This script appends the **Core reference dictionary** sections (gismu, cmavo, tutorial notions, phrases)
from archive/dict TSVs and korpora. Rebuild after editing either the static instructions or dictionary inputs.

- gismu.tsv: top **GISMU_REFERENCE_TOP_N** gismu by corpus score (metrology powers-of-ten after others within the slice), with **GISMU_REFERENCE_ENSURE** always included;
  cmavo.tsv: all cmavo;
  all cmavo grouped by **selma'o** (`### SELMAHO`); PA1 digits no…so (0–9) in fixed order within PA1;
  tutorial/teaching notes per selma'o as `# …` comment lines before that class's entries.
- English ↔ Lojban pairs from korpora TSVs (see `PHRASE_SOURCE_TSV_NAMES` under `KORPORA_DIR`) plus optional Markdown lesson examples (merged into one pool):
  greedy **lexical diversity** — each bare word (not cmevla-shaped) is tracked; phrases are chosen to maximize
  **new** such words per phrase (approximating minimum phrases to expose vocabulary); cmevla tokens (name-shape:
  end in a consonant) are ignored for this score; tie-break by lower corpus token-frequency mass in the pool.
  A reserved math- / logic-leaning batch is still picked first by heuristics.
- Optional **tutorial** block from Markdown: English↔English notions only (Lojban lesson lines are merged into the phrase pool).
  A common auto-detected path is `…/data/pages/en/books/learn-lojban` (e.g. Grav book sources next to this repo).

Environment:
  ASSISTANT_TUTORIAL_BOOK_DIR — directory with numbered lesson `*.md` (e.g. `1.md` … `13.md`).
  If unset, the build tries LEARN_LOJBAN_BOOK_DIR (legacy), then auto-detects `*/data/pages/en/books/*`
  under sibling directories of this repo when `1.md` is present (includes **learn-lojban** when checked out alongside lensisku).

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
SCRIPT_DIR = Path(__file__).resolve().parent
# Editable LLM instructions (jbovlaste behavior); concatenated with reference data below.
ASSISTANT_STATIC_INSTRUCTIONS_PATH = SCRIPT_DIR / "assistant_system_instructions_static.txt"
OUT = REPO / "src" / "assistant" / "assistant_system_prompt.txt"

GISMU_PATH = ARCHIVE / "gismu.tsv"
CMAVO_PATH = ARCHIVE / "cmavo.tsv"
# Bundled copy used only when sibling korpora checkout is absent.
MUPLIS_FALLBACK_PATH = ARCHIVE / "muplis-database.tsv"

# Sibling checkout: lojban/korpora2/korpora/… (paths are stable relative to this repo).
KORPORA_DIR = REPO.parent / "korpora2" / "korpora"
PHRASE_SOURCE_TSV_NAMES: tuple[str, ...] = (
    "forest-nymph.tsv",
    "how-the-enemy-came-to-thlunrana.tsv",
    "muplis-database.tsv",
    "terry-the-tiger-visits-the-big-city.tsv",
    "the-north-wind-and-the-sun.tsv",
    "tlon-uqbar.tsv",
)

# Quotas: merged korpora phrase TSVs + tutorial Markdown examples, then math batch + lexical-diversity fill.
MUPLIS_PHRASE_TOTAL = 300
MUPLIS_MATH_PHRASE_QUOTA = 115

# Gismu section: top-N by corpus score (non-metrology first); these lemmas are always present even if below rank N.
GISMU_REFERENCE_TOP_N = 300
GISMU_REFERENCE_ENSURE: frozenset[str] = frozenset({"cusku", "simxu"})

# Single column separator for every data line (valsi↔gloss, English↔Lojban). U+2194, spaced.
COL_SEP = " ↔ "

# LLM-facing preamble at the top of the bundled file (not build logs—describes sections and sources).
REFERENCE_INTRO_LINES: list[str] = [
    "## About this reference",
    "",
    "The reference sections below are the assistant's offline Lojban bundle. Every data row in those sections uses the same separator: spaced ↔ (Unicode U+2194).",
    "",
    "The **gismu** section lists the top gismu by corpus score (fixed cap at build time; metrology powers-of-ten gismu deprioritized within that cap). A small set of high-value lemmas is always included even if they would fall below the cutoff.",
    "",
    "The **cmavo** section lists all cmavo by selma'o (`### SELMAHO` headings). Optional `#` lines introduce each class; rows are `particle ↔ English gloss`. Within **PA1**, digit cmavo appear in fixed order 0–9 (no…so).",
    "",
    "The **tutorial (notions)** section gives short English↔English notes (alphabet, bridi shape, word classes); they are a compact summary, not the full course.",
    "",
    "The **phrases** sections are English↔Lojban in two parts: (1) lexical spread from korpora; (2) math-, measure-, and logic-leaning lines.",
    "",
    "For lujvo, fu'ivla, experimental entries, long jbovlaste notes, or exhaustive examples, use jbovlaste search when the tool is available.",
    "",
]

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


def one_line(s: str) -> str:
    return " ".join(s.replace("\n", " ").split())


def _is_numeric_scale_gismu(definition: str) -> bool:
    """High-score metrology gismu (powers of ten); deprioritize for learner-focused core list."""
    d = definition
    return "$10^" in d or "$10^{" in d or "10^{-" in d


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


def select_gismu_reference_rows(path: Path) -> list[dict[str, str]]:
    """Top `GISMU_REFERENCE_TOP_N` gismu by score; always include `GISMU_REFERENCE_ENSURE` (swap out lowest-ranked)."""
    top = read_gismu_rows_prefer_non_numeric(path, want=GISMU_REFERENCE_TOP_N)
    have = {(r.get("word") or "").strip().lower() for r in top}
    missing = sorted(GISMU_REFERENCE_ENSURE - have)
    if not missing:
        return top
    full = read_gismu_rows_prefer_non_numeric(path, want=None)
    by_word = {(r.get("word") or "").strip().lower(): r for r in full}
    for w in missing:
        if w not in by_word:
            print(f"warning: GISMU_REFERENCE_ENSURE missing from TSV: {w!r}", file=sys.stderr)
    to_add = [by_word[w] for w in missing if w in by_word]
    if not to_add:
        return top
    # Drop lowest-priority rows from the tail to keep length N.
    combined = top[: GISMU_REFERENCE_TOP_N - len(to_add)] + to_add
    combined.sort(key=lambda x: (-x["_score"], x.get("word", "")))
    seen: set[str] = set()
    out: list[dict[str, str]] = []
    for r in combined:
        w = (r.get("word") or "").strip().lower()
        if w in seen:
            continue
        seen.add(w)
        out.append(r)
        if len(out) >= GISMU_REFERENCE_TOP_N:
            break
    return out


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


def normalize_apostrophe(s: str) -> str:
    for c in "\u2019\u2018’‘":
        s = s.replace(c, "'")
    return s


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


_LOJBAN_VOWELS = frozenset("aeiouy")


def is_lojban_cmevla_shape(bare: str) -> bool:
    """True if bare token looks like a cmevla (content word that ends in a consonant letter)."""
    if not bare:
        return False
    last = bare[-1].lower()
    if not last.isalpha():
        return False
    return last not in _LOJBAN_VOWELS


def diversity_lexicon_words(lo: str) -> set[str]:
    """Bare-word types for phrase diversity, excluding cmevla-shaped names and non-letter tokens."""
    out: set[str] = set()
    for t in lojban_tokens(lo):
        b = bare_token_for_lexicon(t)
        if not b or not any(c.isalpha() for c in b):
            continue
        if is_lojban_cmevla_shape(b):
            continue
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


def select_muplis_diverse(rows: list[tuple[str, str]], n: int) -> list[tuple[str, str]]:
    """Greedy lexical spread: prefer phrases that introduce the most new diversity words (non-cmevla bare types)."""
    indexed = [(i, en, lo) for i, (en, lo) in enumerate(rows)]
    word_freq = corpus_lojban_word_freq(rows)
    covered_words: set[str] = set()
    selected: list[tuple[str, str]] = []
    seen_lo: set[str] = set()
    remaining = [x for x in indexed if x[2] not in seen_lo]

    while len(selected) < n and remaining:
        best_item: tuple[int, str, str] | None = None
        best_key: tuple[int, int, int, int] | None = None
        for item in remaining:
            _i, _en, lo = item
            div = diversity_lexicon_words(lo)
            new_n = len(div - covered_words)
            mass = phrase_token_freq_mass(lo, word_freq)
            key = (new_n, -mass, -len(lo), _i)
            if best_key is None or key > best_key:
                best_key = key
                best_item = item
        if best_item is None:
            break
        _i, en, lo = best_item
        selected.append((en, lo))
        seen_lo.add(lo)
        covered_words |= diversity_lexicon_words(lo)
        remaining = [x for x in remaining if x[2] != lo]

    if len(selected) < n:
        for _i, en, lo in sorted(indexed, key=lambda x: x[0]):
            if len(selected) >= n:
                break
            if lo in seen_lo:
                continue
            selected.append((en, lo))
            seen_lo.add(lo)

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


def merge_phrase_sources(
    tutorial_first: list[tuple[str, str]],
    corpus_pairs: list[tuple[str, str]],
) -> list[tuple[str, str]]:
    """Tutorial pairs first (deduped), then corpus rows whose Lojban text was not already seen."""

    def norm_lo(lo: str) -> str:
        return re.sub(r"\s+", " ", lo.lower())

    seen: set[str] = set()
    out: list[tuple[str, str]] = []
    for en, lo in tutorial_first:
        k = norm_lo(lo)
        if k in seen:
            continue
        seen.add(k)
        out.append((en, lo))
    for en, lo in corpus_pairs:
        k = norm_lo(lo)
        if k in seen:
            continue
        seen.add(k)
        out.append((en, lo))
    return out


def _phrase_en_lo_keys(fieldnames: list[str]) -> tuple[str, str] | None:
    """Map TSV header row to (English column name, Lojban column name)."""
    stripped = [f.strip() for f in fieldnames if f and f.strip()]
    lower_to_orig: dict[str, str] = {}
    for f in stripped:
        lower_to_orig.setdefault(f.lower(), f)
    lo = lower_to_orig.get("lojban") or lower_to_orig.get("lojbo")
    en = lower_to_orig.get("english") or lower_to_orig.get("glico")
    if not lo or not en:
        return None
    return (en, lo)


def phrase_source_tsv_paths() -> list[Path]:
    """Load listed korpora TSVs when present; otherwise bundled archive muplis only."""
    if not KORPORA_DIR.is_dir():
        if MUPLIS_FALLBACK_PATH.is_file():
            print(
                f"korpora dir not found ({KORPORA_DIR}); using {MUPLIS_FALLBACK_PATH}",
                file=sys.stderr,
            )
            return [MUPLIS_FALLBACK_PATH]
        return []
    out: list[Path] = []
    missing: list[str] = []
    for name in PHRASE_SOURCE_TSV_NAMES:
        p = KORPORA_DIR / name
        if p.is_file():
            out.append(p)
        else:
            missing.append(name)
    if missing:
        print("missing korpora phrase TSV(s): " + ", ".join(missing), file=sys.stderr)
    if out:
        return out
    if MUPLIS_FALLBACK_PATH.is_file():
        print(f"no korpora phrase files; using {MUPLIS_FALLBACK_PATH}", file=sys.stderr)
        return [MUPLIS_FALLBACK_PATH]
    return []


def load_phrase_pairs_from_tsv(path: Path) -> list[tuple[str, str]]:
    with path.open(newline="", encoding="utf-8") as f:
        r = csv.DictReader(f, delimiter="\t")
        fn = r.fieldnames
        if not fn:
            return []
        keys = _phrase_en_lo_keys(list(fn))
        if keys is None:
            print(f"skip phrases (need English+glico and Lojban+lojbo columns): {path}", file=sys.stderr)
            return []
        en_key, lo_key = keys
        rows: list[tuple[str, str]] = []
        for row in r:
            en = (row.get(en_key) or "").strip()
            lo = (row.get(lo_key) or "").strip()
            if en and lo:
                rows.append((en, lo))
    return rows


def main() -> int:
    for p in (GISMU_PATH, CMAVO_PATH, ASSISTANT_STATIC_INSTRUCTIONS_PATH):
        if not p.is_file():
            print(f"missing: {p}", file=sys.stderr)
            return 1

    phrase_paths = phrase_source_tsv_paths()
    if not phrase_paths:
        print(
            "missing phrase sources: install korpora under "
            f"{KORPORA_DIR} or provide {MUPLIS_FALLBACK_PATH}",
            file=sys.stderr,
        )
        return 1

    gismu = select_gismu_reference_rows(GISMU_PATH)
    cmavo_grouped, cmavo_n = cmavo_grouped_by_selmaho(CMAVO_PATH)

    phrase_corpus_rows: list[tuple[str, str]] = []
    for path in phrase_paths:
        chunk = load_phrase_pairs_from_tsv(path)
        phrase_corpus_rows.extend(chunk)
        print(f"phrases {path.name}: {len(chunk)} pairs")

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

    phrase_pool = merge_phrase_sources(tutorial_example_pool, phrase_corpus_rows)

    indexed = [(i, en, lo) for i, (en, lo) in enumerate(phrase_pool)]
    math_quota = min(MUPLIS_MATH_PHRASE_QUOTA, MUPLIS_PHRASE_TOTAL)
    math_phrases, math_lo = select_math_prioritized(indexed, math_quota)
    general_pool = [(en, lo) for en, lo in phrase_pool if lo not in math_lo]
    general_n = MUPLIS_PHRASE_TOTAL - len(math_phrases)
    general_phrases = select_muplis_diverse(general_pool, general_n)

    static = ASSISTANT_STATIC_INSTRUCTIONS_PATH.read_text(encoding="utf-8").rstrip()

    ref_lines: list[str] = list(REFERENCE_INTRO_LINES)
    ref_lines.extend(
        [
            f"## gismu (top {len(gismu)} by corpus score).",
            "",
        ]
    )
    ref_lines.extend(format_word_definition_row(row) for row in gismu)
    ref_lines.append("")
    ref_lines.append(
        f"## cmavo ({cmavo_n}; full list by selma'o; PA1 digits 0–9 in order within PA1; "
        f"other classes sorted by corpus score then word)"
    )
    ref_lines.append("")
    ref_lines.extend(format_cmavo_selmaho_sections(cmavo_grouped))
    ref_lines.append("## tutorial (notions; English ↔ English)")
    ref_lines.append("")
    ref_lines.extend(TUTORIAL_NOTION_LINES)
    ref_lines.append("")
    ref_lines.append(
        f"## phrases ({len(general_phrases)}; lexical spread over korpora phrase TSVs + lesson examples)"
    )
    ref_lines.append("")
    for en, lo in general_phrases:
        ref_lines.append(f"{en}{COL_SEP}{lo}")
    ref_lines.append("")
    ref_lines.append(
        f"## phrases ({len(math_phrases)}; math-, measure-, formal-logic; merged pool)"
    )
    ref_lines.append("")
    for en, lo in math_phrases:
        ref_lines.append(f"{en}{COL_SEP}{lo}")

    body = (
        static
        + "\n\n## Core reference dictionary\n\n"
        + "\n".join(ref_lines)
        + "\n"
    )
    nlines = len(body.splitlines())

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(body, encoding="utf-8")
    print(f"Wrote {OUT} ({nlines} lines)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
