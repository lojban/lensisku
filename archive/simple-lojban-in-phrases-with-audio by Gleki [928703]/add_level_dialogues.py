#!/usr/bin/env python3
"""Bring "Simple Lojban in phrases with audio" to the course standard
(../duostories-content/course/COURSE-STANDARD.md): one level per lesson = info card, phrase cards,
and ONE dialogue card LAST (dialogues written in level_dialogues.py, reusing the level's phrases).

Also applies the validated Lojban fixes (level_dialogues.FIXES / BACK_FIXES) and doubtful notes.
Items keep their array index (CSV row i <-> item i, see course_audio.py); dialogue cards are
appended after the existing items.  Items whose Lojban changed are listed in audio_exclude.json so
embed_audio.py no longer attaches the old recording; their listen_wordbank direction becomes
wordbank_reverse.  Idempotent: rerunning on the output changes nothing.

Usage: python3 add_level_dialogues.py [collection.json]   (default: the course JSON; a pristine copy is
kept once as *.bak-pre-dialogues)
"""
import json, os, shutil, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "duostories-content", "scripts"))
import level_dialogues as L  # noqa: E402
from dialogue_lib import render_script, render_translation  # noqa: E402

DEFAULT = os.path.join(HERE, "simple-lojban-in-phrases-with-audio [928703]_collection.json")


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else DEFAULT
    bak = path + ".bak-pre-dialogues"
    if not os.path.exists(bak):
        shutil.copy2(path, bak)
    d = json.load(open(path, encoding="utf-8"))
    items, levels = d["items"], d["levels"]
    assert len(L.DIALOGUES) == len(levels), (len(L.DIALOGUES), len(levels))

    excl_path = os.path.join(HERE, "audio_exclude.json")
    excl = set(json.load(open(excl_path))) if os.path.exists(excl_path) else set()
    for n, (old, new, back, note) in L.FIXES.items():
        it = items[n - 1]
        if it["free_content_front"] == old:
            it["free_content_front"] = new
            if back:
                it["free_content_back"] = back
            it["collection_note"] = "fixed: " + note
        assert it["free_content_front"] == new, (n, it["free_content_front"])
        excl.add(n - 1)
        if it["direction"].startswith("listen_"):
            it["direction"] = "wordbank_reverse"
    for n, (old, new, note) in L.BACK_FIXES.items():
        it = items[n - 1]
        if it["free_content_back"] == old:
            it["free_content_back"] = new
            it["collection_note"] = "fixed: " + note
        assert it["free_content_back"] == new, n
    for n, note in L.DOUBTFUL.items():
        items[n - 1]["collection_note"] = "doubtful: " + note
    json.dump(sorted(excl), open(excl_path, "w"), indent=0)

    fronts = {it["free_content_front"] for it in items}
    for li, lv in enumerate(levels):
        dirs = [items[p]["direction"] for p in lv["item_positions"]]
        if "dialogue" in dirs:
            assert dirs[-1] == "dialogue" and dirs.count("dialogue") == 1, lv["name"]
            continue
        lines = L.DIALOGUES[li]
        entry = {"title": {"jbo": lv["name"], "en": lv["name"]},
                 "speakers": [{"name": s, "gender": L.SPEAKERS[s]} for s in dict.fromkeys(s for s, _, _ in lines)],
                 "lines": [{"speaker": s, "jbo": j, "en": e} for s, j, e in lines]}
        front = render_script(entry)
        assert front not in fronts
        p = len(items)
        note = ("doubtful: " + L.DOUBTFUL_DIALOGUES[li]) if li in L.DOUBTFUL_DIALOGUES else f"dialogue: level {li}"
        items.append({"item_id": max(x["item_id"] for x in items) + 1, "position": p, "collection_note": note,
                      "free_content_front": front, "free_content_back": render_translation(entry),
                      "level_index": li, "position_in_level": len(lv["item_positions"]), "direction": "dialogue"})
        lv["item_positions"].append(p)
        fronts.add(front)
    for lv in levels:  # standard: info card first, dialogue last
        first, last = items[lv["item_positions"][0]], items[lv["item_positions"][-1]]
        assert first["direction"] == "just_information" and last["direction"] == "dialogue", lv["name"]
    json.dump(d, open(path, "w", encoding="utf-8"), ensure_ascii=False, indent=2)
    print(f"levels={len(levels)} items={len(items)} dialogues={sum(x['direction'] == 'dialogue' for x in items)} "
          f"audio_excluded={len(excl)}")


if __name__ == "__main__":
    main()
