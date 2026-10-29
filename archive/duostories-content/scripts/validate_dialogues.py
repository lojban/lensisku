#!/usr/bin/env python3
"""Validate every Lojban sentence in the dialogue drafts and the course file.

Checks per sentence (egglog-lojban parser = `egglojban-analysis`):
  parse    – CLL syntax rejected (or word layer rejected)
  morph    – a non-name word rejected by morphology, or a brivla whose shape
             class disagrees with the dictionaries (e.g. gismu-shaped non-gismu)
  unknown  – word not in gismu/cmavo/lujvo/fu'ivla dictionaries (korpora2)
  name     – invalid cmevla (morphology rejection, e.g. C19 banned la/lai/doi)
             Brivla used as names after la/lai/la'i/doi are valid (la lifri, la lensisku): a
             lujvo/fu'ivla there only needs valid morphology; a gismu shape must be a real gismu.
  speaker  – every speaker label (dialogues.json speakers/lines, course.json characters and
             dialogue lines, speakers of collection dialogue cards) must be a name: a valid
             cmevla (`mi'e la .x.`) or a valid brivla (`mi'e la x`)
  type     – place-type warning from formal-gismu.tsv (typeWarnings)
  parser_limitation – a type warning matched by a reviewed LIMITATION_RULES class (with evidence);
             every other egglog `typeWarnings` entry is a type FAILURE
Doubtful sentences (listed in an entry's `doubtful` list) are reported
separately and do not fail the run.

Every checked sentence gets a refreshed field in the input JSON (unless --no-write):
  "validation": {"status": "ok|warning|doubtful|error",
                 "warnings": [{"kind": "type|parse|parser_limitation|name|dictionary|morphology",
                               "message": <exact egglog output or our check>, "note": <classification>}]}
Collection cards carry one record per card (dialogue warnings also have "where" and "jbo").
The lensisku importer (src/export/models.rs, serde without deny_unknown_fields) ignores the extra field.

Usage:
  python3 scripts/validate_dialogues.py [--json OUT] [--report OUT.md] [FILES...]
Defaults: dialogues.json, course/course.json and course/*_collection.json next to this folder
(collection exports: phrase-card fronts plus every line/question of dialogue cards).
Env: EGGLOJBAN_BIN (parser binary), KORPORA_DICT (dictionaries dir).
"""
import argparse, collections, glob, concurrent.futures as cf, hashlib, json, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)
from dialogue_lib import parse_script_entry, unblank, title_jbo  # noqa: E402

BIN = os.environ.get("EGGLOJBAN_BIN", "/home/user/lojban/egglog-lojban/target/debug/egglojban-analysis")
DICT = os.environ.get("KORPORA_DICT", "/home/user/lojban/korpora2/dictionaries")
CACHE = os.path.join(HERE, ".parse-cache.json")


def load_words(*names):
    out = {}
    for n in names:
        p = os.path.join(DICT, n)
        if not os.path.exists(p):
            continue
        with open(p, encoding="utf-8") as f:
            next(f)
            for line in f:
                w = line.split("\t", 1)[0].strip().lower()
                if w:
                    out.setdefault(w, n)
    return out


GISMU = load_words("formal-gismu.tsv", "gismu.tsv")
EXP_GISMU = load_words("experimental_gismu.tsv")
LUJVO = load_words("lujvo.tsv", "zei-lujvo.tsv")
FUHIVLA = load_words("fuhivla.tsv")
CMAVO = load_words("cmavo.tsv", "cmavo-compound.tsv", "bu-letteral.tsv")
EXP_CMAVO = load_words("experimental_cmavo.tsv")
CMEVLA = load_words("cmevla.tsv")
BRIVLA = {**GISMU, **LUJVO, **FUHIVLA}


def split_cmavo(word):
    return re.findall(r"[^aeiouy',.]?[aeiouy']+", word.replace(",", ""))


# ---------------------------------------------------------------- extraction
def entry_id(e, i):
    return e.get("id") or e.get("draft_id") or f"entry-{i}"


def sentences_of_entry(e, i):
    eid = entry_id(e, i)
    if "lines" in e:
        lines, qs = e["lines"], e.get("questions", [])
        structured = True
    else:
        lines, qs, _ = parse_script_entry(e)
        structured = False
    tj = title_jbo(e)
    if tj:
        yield eid, "title", tj
    for n, ln in enumerate(lines):
        yield eid, f"line {n}", ln["jbo"]
    for n, q in enumerate(qs):
        if q["type"] in ("fill", "bank"):
            yield eid, f"q{n} {q['type']}", unblank(q["jbo"])
            for c in q.get("chips", []) or []:
                yield eid, f"q{n} chip", c
        elif q["type"] == "choice":
            # Legacy script-only entries had English choices; skip those.
            if structured or q.get("jbo_checked"):
                yield eid, f"q{n} choice", q["jbo"]
                for k, a in enumerate(q["answers"]):
                    yield eid, f"q{n} answer {k}", a["jbo"]


def sentences_of_collection(c):
    """lensisku collection export: phrase-card fronts, and every line/question of dialogue cards."""
    for k, it in enumerate(c["items"]):
        d, front = it.get("direction"), it.get("free_content_front") or ""
        iid = f"item {k + 1}"
        if d == "just_information" or not front:
            continue
        if d != "dialogue":
            yield iid, "phrase", front
            continue
        lines, qs, _ = parse_script_entry({"script": front})
        for n, ln in enumerate(lines):
            yield iid, f"line {n}", ln["jbo"]
        for n, q in enumerate(qs):
            if q["type"] in ("fill", "bank"):
                yield iid, f"q{n} {q['type']}", unblank(q["jbo"])
            elif q["type"] == "choice":
                yield iid, f"q{n} choice", q["jbo"]
                for m, ans in enumerate(q["answers"]):
                    yield iid, f"q{n} answer {m}", ans["jbo"]


def sentences_of_course(c):
    for li, lesson in enumerate(c.get("lessons", [])):
        lid = lesson.get("id", f"lesson-{li}")
        for n, ph in enumerate(lesson.get("phrases", [])):
            yield lid, f"phrase {n}", ph["jbo"]
        dia = lesson.get("dialogue") or {}
        if (dia.get("title") or {}).get("jbo"):
            yield lid, "dialogue title", dia["title"]["jbo"]
        for n, ln in enumerate(dia.get("lines", [])):
            yield lid, f"line {n}", ln["jbo"]


SKIP_LABELS = {">", "narrator", "M", "F", ""}


def speaker_labels(data):
    """(id, label) for every speaker label in a dialogues list, course.json or collection export."""
    out = set()
    if isinstance(data, dict) and "items" in data:
        for k, it in enumerate(data["items"]):
            if it.get("direction") == "dialogue":
                lines, qs, spk = parse_script_entry({"script": it.get("free_content_front") or ""})
                out |= {(f"item {k + 1}", x["name"]) for x in spk}
                out |= {(f"item {k + 1}", x["speaker"]) for x in lines + qs if x.get("speaker")}
    elif isinstance(data, dict) and "lessons" in data:
        out |= {("characters", c["name"]) for c in data.get("characters", [])}
        for li, l in enumerate(data["lessons"]):
            d = l.get("dialogue") or {}
            out |= {(l.get("id", f"lesson-{li}"), x["name"]) for x in d.get("speakers", [])}
            out |= {(l.get("id", f"lesson-{li}"), x["speaker"]) for x in d.get("lines", [])}
    else:
        for i, e in enumerate(data):
            out |= {(entry_id(e, i), x["name"]) for x in e.get("speakers", [])}
            out |= {(entry_id(e, i), x["speaker"]) for x in e.get("lines", []) + e.get("questions", []) if x.get("speaker")}
    return sorted((i, l) for i, l in out if l not in SKIP_LABELS)


def label_probes(label):
    w = label.lower()
    return [f"mi'e la .{w}.", f"mi'e la {w}"]


def label_issues(label, cache):
    """[] if the label is a valid cmevla or a valid brivla (gismu in the gismu list, lujvo/fu'ivla by shape)."""
    w = label.lower()
    if not re.fullmatch(r"[a-z',]+", w):
        return [f"{label}: not written with Lojban letters only"]
    cm, br = (cache[p] for p in label_probes(label))
    tok = lambda r: [t for t in r.get("tokens", [])][2:]
    t = tok(cm)
    if cm.get("accepted") and len(t) == 1 and t[0]["accepted"] and t[0]["classes"] == ["cmevla"]:
        return []
    t = tok(br)
    if br.get("accepted") and len(t) == 1 and t[0]["accepted"]:
        cls = t[0]["classes"] or []
        if "gismu" in cls and (w in GISMU or w in FUHIVLA):
            return []
        if "gismu" not in cls and ("lujvo" in cls or any(c.startswith("fu") or "ivla" in c for c in cls)):
            return []
    return [f"{label}: neither a valid cmevla nor a valid brivla ({','.join((tok(br) or [{}])[0].get('classes') or []) or 'rejected'})"]


def doubtful_set(obj):
    s = set()
    for d in obj.get("doubtful", []) or []:
        s.add(d.get("jbo", "").strip())
        if d.get("candidate"):
            s.add(d["candidate"].strip())
    for ln in obj.get("lines", []) or []:
        if ln.get("doubtful"):
            s.add(ln["jbo"].strip())
    for ln in (obj.get("dialogue") or {}).get("lines", []) or []:
        if ln.get("doubtful"):
            s.add(ln["jbo"].strip())
    for ph in obj.get("phrases", []) or []:
        if ph.get("doubtful"):
            s.add(ph["jbo"].strip())
    return s


# ---------------------------------------------------------------- parsing
def bin_key():
    st = os.stat(BIN)
    return f"{st.st_size}-{int(st.st_mtime)}"


def run_parser(text):
    p = subprocess.run([BIN, text], capture_output=True, text=True, timeout=120)
    if p.returncode != 0:
        return {"error": p.stderr[-500:]}
    d = json.loads(p.stdout)
    c = d["cll"]
    return {
        "accepted": c.get("accepted"),
        "gaps": c.get("gaps", []),
        "tokens": [
            {k: t.get(k) for k in ("text", "accepted", "classes", "rejected_by", "notes")}
            for t in (c.get("morphology") or {}).get("tokens", [])
        ],
        "lexemes": [
            {"surface": l["surface"], "terminal": l["terminal"],
             "ind": [a if isinstance(a, str) else a.get("surface", "") for a in l.get("attached_indicators", [])]}
            for l in c.get("lexemes", [])
        ],
        "typeWarnings": d.get("typeWarnings", []),
    }


TYPE_RE = re.compile(r"^x\d of '[^']+' expects (?P<exp>.+?), but got (?P<got>.+)$")


NON_ENTITY = {"Event", "State", "Proposition", "Abstraction", "Process", "TimeInterval",
              "Text", "Symbol", "Number", "Sound", "Idea", "Language", "Dimension"}
PRONOUNS = {"mi", "do", "ko", "mi'o", "mi'a", "ma'a", "do'o", "ta", "ti", "tu", "ri", "ra", "ru"}
ABSTRACT_GOT = re.compile(r"^lo (ka|du'u|si'o|nu|ni|jei|li'i|pu'u|zu'o|mu'e)\b|Abstraction")

# Reviewed type-warning classes that are limitations of egglog-lojban's checker, not Lojban errors.
# Every other type warning is a FAILURE (unless the sentence is on the doubtful list).
# (rule id, evidence) -- the evidence is copied into the "validation" note of each sentence.
LIMITATION_RULES = [
    ("unverified", "egglog reports the place as unverified: no argument type information, nothing was checked"),
    ("name", "names (la + cmevla or la + brivla) are untyped for the checker; the same frame with a typed "
             "description of the same referent passes (e.g. `mi xabju la .moskov.` vs `mi xabju lo tcadu`)"),
    ("pronoun", "pronouns (mi, do, ri ...) are untyped terms for the checker; their referents (people, things) "
                "fit the expected Entity/Organism/Person/Location/Group sort"),
    ("variable", "bound variables (da, su'o da, quantified descriptions), la'e di'u, joi-masses and zoi quotes are "
                 "opaque terms for the checker"),
    ("number", "li-sumti (ti'u li N, se jdima li N) are typed opaquely as 'a mathematical value'"),
    ("set", "formal-gismu types the place as Set<>/Group<>/Sequence<> but xorlo lo-descriptions and pronouns denote "
            "pluralities/individuals; the checker has no individual->Set coercion (CLL/xorlo: `mi te vecnu lo cidja` "
            "is standard; not flagged in korpora2 dwe.tsv type_discrepancy)"),
    ("location", "any object can be a location (CLL 10.2: `vi` + any sumti; `zvati lo zdani`); formal-gismu has no "
                 "Entity/Organism -> Location coercion"),
    ("srana", "`le ti X` / `lo mi X` is typed as srana(ti) (Entity) by the checker, losing the type of X"),
    ("document", "document-like brivla (xatra, cukta, karni ...) are typed Entity in formal-gismu but carry text"),
    ("typevar", "the place has a bare type variable (T extends Entity/Organism); the description's brivla is typed "
                "only Entity in formal-gismu although its referent is of the right sort (e.g. bersa -> Organism)"),
    ("time-tag", "time tag with a quantified bare brivla sumti (ca ro detma'i): the checker cannot type `ro X` "
                 "without lo; the referent is a time interval"),
]
LIMITATION_NOTE = dict(LIMITATION_RULES)


def limitation_rule(w):
    """Return the LIMITATION_RULES id that explains type warning `w`, or None (= real type error)."""
    if "is unverified" in w:
        return "unverified"
    if w.startswith("unresolved: time tag"):
        return "time-tag"
    m = TYPE_RE.match(w)
    if not m:
        return None
    exp, got = m.group("exp"), m.group("got")
    alts = [a.strip() for a in exp.split("|")]
    entityish = not all(re.sub(r"<.*", "", a) in NON_ENTITY for a in alts)
    if got == "a mathematical value":
        return "number"
    if (re.fullmatch(r"la [a-z'.,]+", got) or got == "a named predicate") and entityish:
        return "name"
    if got in PRONOUNS and entityish:
        return "pronoun"
    if got.startswith("bound var") or got in ("a term", "a qualified term", "da", "de", "di", "ma"):
        return "variable"
    if got == "lo srana (Entity)":
        return "srana"
    if ("Text" in alts or "Symbol" in alts) and re.match(r"lo (xatra|cukta|karni|notci|papri|prina|lisri|pemci) ", got):
        return "document"
    if any(a.startswith(("Set<", "Group<", "Sequence<")) for a in alts) and not re.match(r"lo (ka|du'u|si'o)\b", got):
        return "set"
    if "Location" in alts and re.search(r"\((Entity|Organism|Location|Group<Entity>)\)$", got):
        return "location"
    if re.fullmatch(r"[A-Z]", exp) and not ABSTRACT_GOT.search(got):
        return "typevar"
    return None


def analyze(text, res):
    issues = collections.defaultdict(list)
    if "error" in res:
        issues["parse"].append("parser crashed: " + res["error"])
        return issues
    if not res["accepted"]:
        issues["parse"].append("; ".join(res["gaps"]) or "rejected")
    foreign = set()
    for l in res["lexemes"]:
        if l["terminal"] == "FOREIGN_TEXT":
            foreign.update(l["surface"].lower().split())
    in_name = False
    for t in res["tokens"]:
        w = t["text"].lower()
        if w in foreign:
            continue
        cls = t["classes"] or []
        name_ctx, in_name = in_name, w in ("la", "lai", "la'i", "doi")
        if not t["accepted"]:
            notes = "; ".join(n for n in t["notes"] if not n.startswith(("pause", "C04", "C08", "C09")))
            if "cmevla" in cls or re.search(r"[^aeiouy'.,]$", w):
                issues["name"].append(f"{w}: {','.join(t['rejected_by'])} {notes}".strip())
            else:
                issues["morph"].append(f"{w}: {','.join(t['rejected_by'])} {notes}".strip())
            continue
        if "cmevla" in cls:
            continue
        if "gismu" in cls:
            if w in GISMU:
                in_name = name_ctx
                continue
            if w in EXP_GISMU:
                issues["unknown"].append(f"{w}: experimental gismu")
            elif w in FUHIVLA:
                continue
            elif w in LUJVO:
                issues["morph"].append(f"{w}: listed as lujvo but has gismu shape")
            else:
                issues["unknown"].append(f"{w}: gismu-shaped word not in gismu lists")
        elif name_ctx and t["accepted"] and ("lujvo" in cls or any(c.startswith("fu") or "ivla" in c for c in cls)):
            in_name = True  # brivla used as a name (la lensisku, la barda plise): morphology is enough
        elif "lujvo" in cls:
            if w not in LUJVO and w not in FUHIVLA:
                issues["unknown"].append(f"{w}: lujvo not in lujvo.tsv")
        elif any(c.startswith("fu") or "ivla" in c for c in cls):
            if w not in FUHIVLA and w not in LUJVO:
                issues["unknown"].append(f"{w}: fu'ivla not in fuhivla.tsv")
        elif "cmavo" in cls or "cmavo_compound" in cls:
            parts = [w] if w in CMAVO else split_cmavo(w)
            for p in parts:
                if p in CMAVO:
                    continue
                if p in EXP_CMAVO:
                    issues["unknown"].append(f"{p}: experimental cmavo")
                else:
                    issues["unknown"].append(f"{p}: cmavo not in cmavo.tsv")
        else:
            if w not in BRIVLA:
                issues["morph"].append(f"{w}: unclassified ({','.join(cls)})")
    for w in res["typeWarnings"]:
        issues["parser_limitation" if limitation_rule(w) else "type"].append(w)
    return issues


KIND = {"parse": "parse", "morph": "morphology", "unknown": "dictionary", "name": "name", "speaker": "name",
        "type": "type", "parser_limitation": "parser_limitation"}
VAL_KEYS = ("validation", "chips_validation", "speaker_validation")


def warning_list(iss, doubt):
    out = []
    for k, msgs in iss.items():
        for m in msgs:
            if k == "parser_limitation":
                r = limitation_rule(m)
                note = f"parser limitation ({r}): {LIMITATION_NOTE[r]}"
            elif doubt:
                note = "doubtful: kept on the doubtful list with this diagnostic"
            else:
                note = {"type": "type violation (failure)"}.get(k, f"{KIND[k]} failure")
            out.append({"kind": KIND[k], "message": m, "note": note})
    return out


def status_of(doubt, iss):
    if doubt:
        return "doubtful"
    if any(k != "parser_limitation" for k in iss):
        return "error"
    return "warning" if iss else "ok"


def validation_record(doubt, iss):
    return {"status": status_of(doubt, iss), "warnings": warning_list(iss, doubt)}


def strip_validation(o):
    if isinstance(o, dict):
        for k in VAL_KEYS:
            o.pop(k, None)
        for v in o.values():
            strip_validation(v)
    elif isinstance(o, list):
        for v in o:
            strip_validation(v)


RANK = {"ok": 0, "warning": 1, "doubtful": 2, "error": 3}


def write_validation(path, per):
    """Refresh the `validation` field of every checked sentence in `path` (re-run = refresh)."""
    base = os.path.basename(path)
    raw = open(path, encoding="utf-8").read()
    data = json.loads(raw)
    strip_validation(data)
    mine = {sid: v for (f, sid), v in per.items() if f == base}
    if isinstance(data, dict) and "items" in data and "levels" in data:
        for k, it in enumerate(data["items"]):
            rec = mine.get(f"item {k + 1}")
            if not rec:
                continue
            ws, st = [], "ok"
            for where, text, doubt, iss in rec:
                r = validation_record(doubt, iss)
                st = max(st, r["status"], key=RANK.get)
                for w in r["warnings"]:
                    ws.append({**({"where": where, "jbo": text} if where != "phrase" else {}), **w})
            it["validation"] = {"status": st, "warnings": ws}
    elif isinstance(data, dict) and "lessons" in data:
        for li, lesson in enumerate(data["lessons"]):
            for where, text, doubt, iss in mine.get(lesson.get("id", f"lesson-{li}"), []):
                r = validation_record(doubt, iss)
                kind, _, n = where.rpartition(" ")
                if where.startswith("phrase "):
                    lesson["phrases"][int(n)]["validation"] = r
                elif where.startswith("line "):
                    lesson["dialogue"]["lines"][int(n)]["validation"] = r
                elif where == "dialogue title":
                    lesson["dialogue"]["title"]["validation"] = r
                else:
                    lesson.setdefault("speaker_validation", []).append({"label": text, **r})
        for where, text, doubt, iss in mine.get("characters", []):
            data.setdefault("speaker_validation", []).append({"label": text, **validation_record(doubt, iss)})
    else:
        for i, e in enumerate(data):
            for where, text, doubt, iss in mine.get(entry_id(e, i), []):
                r = validation_record(doubt, iss)
                m = re.fullmatch(r"q(\d+) (\w+)(?: (\d+))?", where)
                if where == "title" and isinstance(e.get("title"), dict):
                    e["title"]["validation"] = r
                elif where.startswith("line "):
                    e["lines"][int(where.split()[1])]["validation"] = r
                elif m and m.group(2) == "chip":
                    e["questions"][int(m.group(1))].setdefault("chips_validation", []).append({"jbo": text, **r})
                elif m and m.group(2) == "answer":
                    e["questions"][int(m.group(1))]["answers"][int(m.group(3))]["validation"] = r
                elif m:
                    e["questions"][int(m.group(1))]["validation"] = r
                else:
                    e.setdefault("speaker_validation", []).append({"label": text, **r})
    out = json.dumps(data, ensure_ascii=False, indent=2) + ("\n" if raw.endswith("\n") else "")
    if out != raw:
        open(path, "w", encoding="utf-8").write(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="*")
    ap.add_argument("--json")
    ap.add_argument("--report")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--quiet", action="store_true")
    ap.add_argument("--no-write", action="store_true",
                    help="do not refresh the per-sentence `validation` fields in the input JSON files")
    a = ap.parse_args()
    files = a.files or [p for p in (os.path.join(ROOT, "dialogues.json"), os.path.join(ROOT, "course", "course.json")) if os.path.exists(p)] \
        + sorted(glob.glob(os.path.join(ROOT, "course", "*_collection.json")))

    items = []  # (file, id, where, text, doubtful)
    labels = []  # (file, id, label)
    for f in files:
        labels += [(os.path.basename(f), i, l) for i, l in speaker_labels(json.load(open(f, encoding="utf-8")))]
    # Doubtful sentences known from the structured sources also count inside collection files.
    global_ds = set()
    for f in [os.path.join(ROOT, "dialogues.json"), os.path.join(ROOT, "course", "course.json")] + list(files):
        if not os.path.exists(f) or f.endswith("_collection.json"):
            continue
        data = json.load(open(f, encoding="utf-8"))
        for e in (data["lessons"] if isinstance(data, dict) and "lessons" in data else data):
            global_ds |= doubtful_set(e)
    for f in files:
        data = json.load(open(f, encoding="utf-8"))
        if isinstance(data, dict) and "items" in data and "levels" in data:
            notes = {f"item {k + 1}": (it.get("collection_note") or "").startswith("doubtful") for k, it in enumerate(data["items"])}
            for sid, where, text in sentences_of_collection(data):
                t = text.strip()
                items.append((os.path.basename(f), sid, where, text, t in global_ds or (".i " + t) in global_ds or notes[sid]))
            continue
        if isinstance(data, dict) and "lessons" in data:
            for li, lesson in enumerate(data["lessons"]):
                ds = doubtful_set(lesson)
                for sid, where, text in sentences_of_course({"lessons": [lesson]}):
                    items.append((os.path.basename(f), sid, where, text, text.strip() in ds))
            continue
        for i, e in enumerate(data):
            ds = doubtful_set(e)
            for sid, where, text in sentences_of_entry(e, i):
                items.append((os.path.basename(f), sid, where, text, text.strip() in ds))

    cache = {}
    if os.path.exists(CACHE):
        try:
            cache = json.load(open(CACHE))
        except Exception:
            cache = {}
    bk = bin_key()
    if cache.get("_bin") != bk:
        cache = {"_bin": bk}
    todo = sorted({t for *_, t, _d in items if t not in cache} | {p for *_, l in labels for p in label_probes(l) if p not in cache})
    with cf.ThreadPoolExecutor(a.jobs) as ex:
        for t, r in zip(todo, ex.map(run_parser, todo)):
            cache[t] = r
    json.dump(cache, open(CACHE, "w"))

    cats = ["parse", "morph", "unknown", "name", "type", "parser_limitation"]
    counts = {"sentences": len(items), "doubtful": 0, **{c: 0 for c in cats}, **{"doubtful_" + c: 0 for c in cats}}
    unique_words = {c: collections.Counter() for c in ("unknown", "name", "morph")}
    rows = []
    per = collections.defaultdict(list)  # (file, sid) -> [(where, text, doubt, issues)]
    for f, sid, where, text, doubt in items:
        iss = analyze(text, cache[text])
        per[(f, sid)].append((where, text, doubt, iss))
        if doubt:
            counts["doubtful"] += 1
        for c in cats:
            if iss.get(c):
                counts[("doubtful_" if doubt else "") + c] += 1
                if c in unique_words and not doubt:
                    for m in iss[c]:
                        unique_words[c][m.split(":")[0]] += 1
        if any(iss.values()):
            rows.append({"file": f, "id": sid, "where": where, "jbo": text, "doubtful": doubt, "issues": dict(iss)})
    bad_labels = {}
    for f, sid, l in labels:
        iss = label_issues(l, cache)
        if iss:
            per[(f, sid)].append(("speaker label", l, False, {"speaker": iss}))
            bad_labels.setdefault(l, [])
            rows.append({"file": f, "id": sid, "where": "speaker label", "jbo": l, "doubtful": False, "issues": {"speaker": iss}})
    counts["speaker_labels"] = len({l for *_, l in labels})
    counts["invalid_speaker_labels"] = len(bad_labels)
    failing = [r for r in rows if not r["doubtful"] and any(k != "parser_limitation" for k in r["issues"])]
    counts["failing_sentences"] = len(failing)
    counts["unique_unknown_words"] = len(unique_words["unknown"])
    counts["unique_invalid_names"] = len(unique_words["name"])
    print(json.dumps(counts, indent=1))
    if not a.quiet:
        for r in failing:
            print(f"- [{r['id']} {r['where']}] {r['jbo']}")
            for c, ms in r["issues"].items():
                for m in ms:
                    print(f"    {c}: {m}")
    if not a.no_write:
        for f in files:
            write_validation(f, per)
    if a.json:
        json.dump({"counts": counts, "rows": rows, "words": {k: dict(v) for k, v in unique_words.items()}}, open(a.json, "w"), ensure_ascii=False, indent=1)
    if a.report:
        with open(a.report, "w") as out:
            out.write("# Validation report\n\n")
            for k, v in counts.items():
                out.write(f"- {k}: {v}\n")
            for title, sel in (("Failing (non-doubtful)", failing), ("Doubtful with issues", [r for r in rows if r["doubtful"]])):
                out.write(f"\n## {title}\n\n")
                for r in sel:
                    out.write(f"- `{r['id']}` {r['where']}: `{r['jbo']}`\n")
                    for c, ms in r["issues"].items():
                        for m in ms:
                            out.write(f"  - {c}: {m}\n")
    sys.exit(1 if failing else 0)


if __name__ == "__main__":
    main()
