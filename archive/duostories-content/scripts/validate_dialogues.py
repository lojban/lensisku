#!/usr/bin/env python3
"""Validate every Lojban sentence in the dialogue drafts and the course file.

Checks per sentence (egglog-lojban parser = `egglojban-analysis`):
  parse    – CLL syntax rejected (or word layer rejected)
  morph    – a non-name word rejected by morphology, or a brivla whose shape
             class disagrees with the dictionaries (e.g. gismu-shaped non-gismu)
  unknown  – word not in gismu/cmavo/lujvo/fu'ivla dictionaries (korpora2)
  name     – invalid cmevla (morphology rejection, e.g. C19 banned la/lai/doi)
  type     – place-type warning from formal-gismu.tsv (typeWarnings)
Doubtful sentences (listed in an entry's `doubtful` list) are reported
separately and do not fail the run.

Usage:
  python3 scripts/validate_dialogues.py [--json OUT] [--report OUT.md] [FILES...]
Defaults: dialogues.json and course/course.json next to this folder.
Env: EGGLOJBAN_BIN (parser binary), KORPORA_DICT (dictionaries dir).
"""
import argparse, collections, concurrent.futures as cf, hashlib, json, os, re, subprocess, sys

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


def checker_limitation(w):
    """Type warnings that reflect checker/type-system limits, not Lojban errors.

    Calibrated on korpora2/korpora/dwe.tsv `type_discrepancy` (e.g. `mi pu te
    vecnu le plise`, `do xabju ma noi tcadu` are not flagged there).
    """
    if w.startswith("unresolved:"):
        return True
    m = TYPE_RE.match(w)
    if not m:
        return False
    exp, got = m.group("exp"), m.group("got")
    if got == "a mathematical value":
        return True  # ti'u li xa: li-sumti are typed opaquely by the checker
    alts = [a.strip() for a in exp.split("|")]
    if ("Text" in alts or "Symbol" in alts) and re.match(r"lo (xatra|cukta|karni|notci|papri|prina|lisri|pemci) ", got):
        return True  # document-like brivla typed Entity in formal-gismu
    # Pronouns and names are untyped for the checker; they still denote
    # entities, so they are only a limitation when an entity could fit.
    NON_ENTITY = {"Event", "State", "Proposition", "Abstraction", "Process", "TimeInterval",
                  "Text", "Symbol", "Number", "Sound", "Idea", "Language"}
    pronoun = got in ("mi", "do", "ko", "mi'o", "mi'a", "ma'a", "do'o", "ta", "ti", "tu") or re.fullmatch(r"la [a-z'.]+", got)
    if pronoun and not all(a in NON_ENTITY for a in alts):
        return True
    if got.startswith("bound var") or got in ("a term", "a qualified term", "ri", "ra", "ru", "da", "de", "di", "ma"):
        return True
    if got == "lo srana (Entity)":
        return True  # `le ti X` is typed as srana(ti) by the checker, losing X
        return True
    if re.fullmatch(r"[A-Z]", exp):
        return True  # bare type variable (dukse x2 T)
    if any(a.startswith(("Set<", "Group<")) for a in alts) and not re.match(r"lo (ka|du'u|si'o)\b", got):
        return True  # lo-description of members/masses vs Set<T>
    if "Location" in alts and ("(Entity)" in got or got.startswith("la ") or "(Location)" in got or "(Organism)" in got):
        return True
    return False


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
    for t in res["tokens"]:
        w = t["text"].lower()
        if w in foreign:
            continue
        cls = t["classes"] or []
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
                continue
            if w in EXP_GISMU:
                issues["unknown"].append(f"{w}: experimental gismu")
            elif w in FUHIVLA:
                continue
            elif w in LUJVO:
                issues["morph"].append(f"{w}: listed as lujvo but has gismu shape")
            else:
                issues["unknown"].append(f"{w}: gismu-shaped word not in gismu lists")
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
        if "unverified" in w:
            continue
        issues["type_checker" if checker_limitation(w) else "type"].append(w)
    return issues


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="*")
    ap.add_argument("--json")
    ap.add_argument("--report")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--quiet", action="store_true")
    a = ap.parse_args()
    files = a.files or [p for p in (os.path.join(ROOT, "dialogues.json"), os.path.join(ROOT, "course", "course.json")) if os.path.exists(p)]

    items = []  # (file, id, where, text, doubtful)
    for f in files:
        data = json.load(open(f, encoding="utf-8"))
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
    todo = sorted({t for *_, t, _d in items if t not in cache})
    with cf.ThreadPoolExecutor(a.jobs) as ex:
        for t, r in zip(todo, ex.map(run_parser, todo)):
            cache[t] = r
    json.dump(cache, open(CACHE, "w"))

    cats = ["parse", "morph", "unknown", "name", "type", "type_checker"]
    counts = {"sentences": len(items), "doubtful": 0, **{c: 0 for c in cats}, **{"doubtful_" + c: 0 for c in cats}}
    unique_words = {c: collections.Counter() for c in ("unknown", "name", "morph")}
    rows = []
    for f, sid, where, text, doubt in items:
        iss = analyze(text, cache[text])
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
    failing = [r for r in rows if not r["doubtful"] and any(k != "type_checker" for k in r["issues"])]
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
