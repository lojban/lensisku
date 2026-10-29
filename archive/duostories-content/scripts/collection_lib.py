"""Shared builder for lensisku collection import files that follow course/COURSE-STANDARD.md:
one level per lesson/story = info card (just_information) first, phrase cards, ONE dialogue card last.
Levels are chained linearly; duplicate free-content fronts are dropped (the importer would skip them)."""
import json
from dialogue_lib import render_script, render_translation


def direction_for(jbo, k):
    if len(jbo.replace(".i ", "").split()) <= 2:
        return "quiz_direct"
    return "wordbank_reverse" if k % 2 == 0 else "wordbank"


class Collection:
    def __init__(self, name, description, is_public=False):
        self.meta = {"name": name, "description": description, "is_public": is_public}
        self.items, self.levels, self.seen = [], [], set()
        self.story_no = 0

    def add_level(self, name, desc, cards, min_cards=None):
        """cards: [(front, back, direction, note)]; first must be just_information, last dialogue."""
        last = cards[-1]
        cards = [c for c in cards if c[0] not in self.seen]
        assert cards and cards[-1] is last and last[2] == "dialogue", name
        assert cards[0][2] == "just_information", name
        assert sum(c[2] == "dialogue" for c in cards) == 1, name
        li = len(self.levels)
        pos = []
        for j, (front, back, direction, note) in enumerate(cards):
            self.seen.add(front)
            p = len(self.items)
            self.items.append({"item_id": p + 1, "position": p, "collection_note": note, "free_content_front": front,
                               "free_content_back": back, "level_index": li, "position_in_level": j,
                               "direction": direction})
            pos.append(p)
        if min_cards is None:
            min_cards = min(5, len(cards) - 1)
        self.levels.append({"name": name, "description": desc, "min_cards": min_cards, "min_success_rate": 0.8,
                            "position": li, "prerequisite_positions": [li - 1] if li else [],
                            "item_positions": pos})

    def story_phrases(self, e, limit=5):
        """Short spoken lines of a story (2-7 words, not doubtful, not already used) as practice cards."""
        doubtful = {d.get("jbo", "").strip() for d in e.get("doubtful", []) or []}
        out = []
        for ln in e["lines"]:
            jbo = ln["jbo"].strip()
            front = jbo[3:] if jbo.startswith(".i ") else jbo
            if ln["speaker"] == ">" or ln.get("doubtful") or jbo in doubtful or not (2 <= len(front.split()) <= 7):
                continue
            if front in self.seen or any(front == c[0] for c in out):
                continue
            out.append((front, ln["en"], direction_for(front, len(out)), ""))
            if len(out) == limit:
                break
        return out

    def add_story(self, key, e):
        self.story_no += 1
        n, t = self.story_no, e["title"]
        who = ", ".join(sp["name"] for sp in e.get("speakers", []))
        info = (f"Story {n}: {t['jbo']} — {t['en']}",
                f"#### Before you read\nCharacters: {who or '(narrator)'}\n\nPractise a few lines from the story, "
                f"then read the whole story and answer its questions.", "just_information", key)
        ph = self.story_phrases(e)
        story = (render_script(e), render_translation(e), "dialogue", key)
        self.add_level(f"Story {n}: {t['jbo']} ({t['en']})", "Story from the duostories adaptation (dialogues.json).",
                       [info] + ph + [story], min(5, len(ph) + 1))

    def write(self, path):
        json.dump({"collection": self.meta, "items": self.items, "levels": self.levels},
                  open(path, "w", encoding="utf-8"), ensure_ascii=False, indent=2)
        return f"{self.meta['name']}: levels={len(self.levels)} items={len(self.items)}"
