#!/usr/bin/env python3
"""Build the progressive flashcard course from scripts/course_data.py.

Outputs (next to dialogues.json):
  course/course.json                                  structured lessons (validated by validate_dialogues.py)
  course/lojban-through-dialogues_collection.json     lensisku collection export (import format of
                                                      archive/*/..._collection.json): per lesson an info card,
                                                      isolated phrase cards and one 'dialogue' card; the
                                                      existing 45 stories from dialogues.json as reading levels.
Usage: python3 scripts/build_course.py
"""
import json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)
import course_data as C  # noqa: E402
from dialogue_lib import render_script, render_translation  # noqa: E402


def block_of(n):
    for a, b, name in C.BLOCKS:
        if a <= n <= b:
            return name


def direction_for(jbo, k):
    if len(jbo.replace(".i ", "").split()) <= 2:
        return "quiz_direct"
    return "wordbank_reverse" if k % 2 == 0 else "wordbank"


def lessons():
    out = []
    for i, t in enumerate(C.L, 1):
        title_en, title_jbo, words, grammar, phrases, lines = t[:6]
        doubtful = t[6] if len(t) > 6 else []
        dmap = {w: (cand, note) for w, cand, note in doubtful}
        lid = f"lesson-{i:02d}"
        used = []
        spk = []
        for s, _, _ in lines:
            if s != ">" and s not in spk:
                spk.append(s)
        ph = []
        for k, (jbo, en) in enumerate(phrases):
            d = {"jbo": jbo, "en": en, "direction": direction_for(jbo, k)}
            if f"phrase {k}" in dmap:
                d["doubtful"] = dmap[f"phrase {k}"][1]
            ph.append(d)
        ls = []
        for k, (s, jbo, en) in enumerate(lines):
            d = {"speaker": s, "jbo": jbo, "en": en}
            if f"line {k}" in dmap:
                d["doubtful"] = dmap[f"line {k}"][1]
            ls.append(d)
        out.append({
            "id": lid, "number": i, "block": block_of(i),
            "title": {"en": title_en, "jbo": title_jbo},
            "new_words": words, "grammar": grammar, "phrases": ph,
            "dialogue": {"title": {"en": title_en, "jbo": title_jbo},
                         "speakers": [{"name": s, "gender": C.SPEAKERS[s]} for s in spk],
                         "lines": ls},
            "doubtful": [{"where": w, "jbo": "", "candidate": c, "note": n} for w, c, n in doubtful],
            "readings_after": C.READINGS.get(i, []),
        })
    return out


def main():
    stories = json.load(open(os.path.join(ROOT, "dialogues.json"), encoding="utf-8"))
    sid = {}
    for k, e in enumerate(stories):
        sid[e.get("draft_id") or f"batch2-{k - 4:02d}"] = e
    ls = lessons()
    course = {
        "title": {"en": "Lojban through dialogues", "jbo": "la .lojban. fo lo casnu"},
        "plan": "lojban.pw data/pages/en/books/lojban-through-dialogues/new-dialogue-plan.md",
        "characters": [{"name": n, "cmevla": C.NAMES[n], "gender": C.SPEAKERS[n]} for n in C.NAMES],
        "plan_deviations": C.PLAN_DEVIATIONS,
        "lessons": ls,
        "readings_after_course": C.READINGS_AFTER_COURSE,
    }
    os.makedirs(os.path.join(ROOT, "course"), exist_ok=True)
    json.dump(course, open(os.path.join(ROOT, "course", "course.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=2)

    items, levels = [], []
    seen_fronts = set()

    def add_level(name, desc, cards, min_cards):
        li = len(levels)
        pos = []
        cards = [c for c in cards if c[0] not in seen_fronts]  # importer skips duplicate free-content fronts
        for j, (front, back, direction, note) in enumerate(cards):
            seen_fronts.add(front)
            p = len(items)
            items.append({"item_id": p + 1, "position": p, "collection_note": note, "free_content_front": front,
                          "free_content_back": back, "level_index": li, "position_in_level": j, "direction": direction})
            pos.append(p)
        levels.append({"name": name, "description": desc, "min_cards": min_cards, "min_success_rate": 0.8,
                       "position": li, "prerequisite_positions": [li - 1] if li else [], "item_positions": pos})

    def add_story(key):
        e = sid[key]
        t = e["title"]
        add_level(f"Reading: {t['jbo']} ({t['en']})", "Story from the duostories adaptation (dialogues.json).",
                  [(render_script(e), render_translation(e), "dialogue", key)], 1)

    for l in ls:
        n = l["number"]
        words = ", ".join("{%s}" % w if " " not in w and "..." not in w else w for w in l["new_words"])
        info_back = f"#### New words\n{words or '(review)'}\n\n#### Grammar\n{l['grammar']}"
        cards = [(f"Lesson {n}: {l['title']['en']} — {l['title']['jbo']}", info_back, "just_information", "")]
        for p in l["phrases"]:
            cards.append((p["jbo"], p["en"], p["direction"], ("doubtful: " + p["doubtful"]) if p.get("doubtful") else ""))
        add_level(f"Lesson {n}: {l['title']['en']}", f"{l['block']} · {l['title']['jbo']}", cards, min(5, len(cards) - 1))
        dia = dict(l["dialogue"])
        add_level(f"Lesson {n} dialogue: {l['title']['jbo']}", l["title"]["en"],
                  [(render_script(dia), render_translation(dia), "dialogue", l["id"])], 1)
        for key in l["readings_after"]:
            add_story(key)
    for key in C.READINGS_AFTER_COURSE:
        add_story(key)
    coll = {"collection": {"name": "Lojban through dialogues",
                           "description": "Progressive 30-lesson course (Lifri, Mentu, Linto): isolated phrases and full dialogues, then 45 graded stories. All Lojban validated with egglog-lojban.",
                           "is_public": False},
            "items": items, "levels": levels}
    json.dump(coll, open(os.path.join(ROOT, "course", "lojban-through-dialogues_collection.json"), "w", encoding="utf-8"),
              ensure_ascii=False, indent=2)
    print(f"lessons={len(ls)} levels={len(levels)} items={len(items)}")


if __name__ == "__main__":
    main()
