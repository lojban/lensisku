#!/usr/bin/env python3
"""Build the progressive flashcard course from scripts/course_data.py.

Outputs (next to dialogues.json):
  course/course.json                                  structured lessons (validated by validate_dialogues.py)
  course/lojban-through-dialogues_collection.json     lensisku collection export (import format of
                                                      archive/*/..._collection.json).
Level standard (course/COURSE-STANDARD.md): ONE level per lesson = info card, isolated phrase
cards, then the lesson's full dialogue as the last card. The 45 existing stories get levels of
the same shape: info card, up to 5 short lines of the story as phrase cards, story card last.
Usage: python3 scripts/build_course.py
"""
import json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)
import course_data as C  # noqa: E402
from dialogue_lib import render_script, render_translation  # noqa: E402
from collection_lib import Collection, direction_for  # noqa: E402


def block_of(n):
    for a, b, name in C.BLOCKS:
        if a <= n <= b:
            return name


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
        "story_courses": [{"file": f"{a}_collection.json", "name": b, "stories": [f"batch2-{k:02d}" for k in n]} for a, b, _, n in C.STORY_COURSES],
    }
    os.makedirs(os.path.join(ROOT, "course"), exist_ok=True)
    json.dump(course, open(os.path.join(ROOT, "course", "course.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=2)

    out = os.path.join(ROOT, "course")
    main_c = Collection("Lojban through dialogues",
                        "Progressive 30-lesson course (Lifri, Mentu, Linto): one level per lesson = info card, "
                        "phrases, dialogue last; stories 01-05 interleaved. All Lojban validated with egglog-lojban.")
    for l in ls:
        n = l["number"]
        words = ", ".join("{%s}" % w if " " not in w and "..." not in w else w for w in l["new_words"])
        info_back = f"#### New words\n{words or '(review)'}\n\n#### Grammar\n{l['grammar']}"
        cards = [(f"Lesson {n}: {l['title']['en']} — {l['title']['jbo']}", info_back, "just_information", "")]
        for p in l["phrases"]:
            cards.append((p["jbo"], p["en"], p["direction"], ("doubtful: " + p["doubtful"]) if p.get("doubtful") else ""))
        # The lesson's full dialogue is the LAST card of the same level (COURSE-STANDARD.md).
        cards.append((render_script(dict(l["dialogue"])), render_translation(dict(l["dialogue"])), "dialogue", l["id"]))
        main_c.add_level(f"Lesson {n}: {l['title']['en']}", f"{l['block']} · {l['title']['jbo']}", cards)
        for key in l["readings_after"]:
            main_c.add_story(key, sid[key])
    print(main_c.write(os.path.join(out, "lojban-through-dialogues_collection.json")))
    # Batch-2 stories: separate themed story courses (course_data.STORY_COURSES).
    for slug, name, desc, nums in C.STORY_COURSES:
        c = Collection(name, desc)
        for k in nums:
            c.add_story(f"batch2-{k:02d}", sid[f"batch2-{k:02d}"])
        print(c.write(os.path.join(out, f"{slug}_collection.json")))


if __name__ == "__main__":
    main()
