"""Shared helpers for the Lojban dialogue drafts (dialogues.json).

Structured entries hold `lines` / `questions` with `jbo` + `en`; `script`
(dialogue markup, see src/dialogue/parser.rs) and `translation` (card back,
needed by frontend/src/utils/dialogueImport.ts) are rendered from them.
"""
import re

SPEECH_RE = re.compile(r"^(?:\[(?P<voice>\w+)\]\s*)?(?:(?P<narr>>)|(?P<spk>[A-Za-z][\w' -]*?)(?:\((?P<g>[MF])\))?:)\s*(?P<text>.*)$")


def speaker_gender(entry, name):
    for s in entry.get("speakers", []):
        if s["name"] == name:
            return s.get("gender")
    return None


def speaker_prefix(entry, name):
    if name in (">", "narrator"):
        return "> "
    if name in ("M", "F"):
        return f"{name}: "
    g = speaker_gender(entry, name)
    return f"{name}({g}): " if g else f"{name}: "


def title_jbo(entry):
    t = entry.get("title")
    return t.get("jbo") if isinstance(t, dict) else t


def title_en(entry):
    t = entry.get("title")
    return t.get("en") if isinstance(t, dict) else t


def render_script(entry):
    out = [f"# {title_jbo(entry)}"]
    for ln in entry["lines"]:
        out.append(speaker_prefix(entry, ln["speaker"]) + ln["jbo"])
        if ln.get("en"):
            out.append(f"~ {ln['en']}")
    qs = entry.get("questions", [])
    if qs:
        out.append("---")
    for q in qs:
        if q["type"] in ("fill", "bank"):
            sp = speaker_prefix(entry, q["speaker"]) if q.get("speaker") else ""
            line = f"? {q['type']}: {sp}{q['jbo']}"
            if q["type"] == "bank" and q.get("chips"):
                line += f" (chips: {', '.join(q['chips'])})"
            out.append(line)
        elif q["type"] == "choice":
            out.append(f"? choice: {q['jbo']}")
            for a in q["answers"]:
                out.append(("+ " if a["correct"] else "- ") + a["jbo"])
    return "\n".join(out)


def render_translation(entry):
    """Card back: English title + English transcript."""
    parts = [title_en(entry) + "."] if title_en(entry) else []
    for ln in entry["lines"]:
        if ln["speaker"] in (">", "narrator"):
            parts.append(ln["en"])
        else:
            parts.append(f"{ln['speaker']}: {ln['en']}")
    return "\n".join(parts)


def unblank(s):
    """`lo [ckiku|ckiku'u] cu …` -> first alternative, brackets removed."""
    return re.sub(r"\[([^\]|]*)(?:\|[^\]]*)?\]", r"\1", s)


def parse_script_entry(entry):
    """Convert a legacy script-only entry into structured lines/questions."""
    lines, questions, speakers = [], [], {}
    section = "lines"
    pending_choice = None
    for raw in entry["script"].split("\n"):
        t = raw.strip()
        if not t:
            continue
        if t.startswith("#"):
            continue
        if t == "---":
            section = "q"
            continue
        if section == "lines":
            if t.startswith("~"):
                lines[-1]["en"] = t[1:].strip()
                continue
            m = SPEECH_RE.match(t)
            spk = ">" if m.group("narr") else m.group("spk")
            if m.group("g"):
                speakers[spk] = m.group("g")
            lines.append({"speaker": spk, "jbo": m.group("text").strip(), "en": ""})
        else:
            if t[0] in "+-" and pending_choice is not None:
                pending_choice["answers"].append({"jbo": t[1:].strip(), "en": t[1:].strip(), "correct": t[0] == "+"})
                continue
            body = t[1:].strip()
            kind, rest = body.split(":", 1)
            rest = rest.strip()
            if kind == "choice":
                pending_choice = {"type": "choice", "jbo": rest, "en": rest, "answers": []}
                questions.append(pending_choice)
            else:
                pending_choice = None
                chips = None
                mm = re.search(r"\s*\(chips:\s*([^)]*)\)\s*$", rest)
                if mm:
                    chips = [c.strip() for c in mm.group(1).split(",")]
                    rest = rest[: mm.start()]
                m = SPEECH_RE.match(rest)
                spk, text = (None, rest)
                if m and (m.group("spk") or m.group("narr")):
                    spk = ">" if m.group("narr") else m.group("spk")
                    text = m.group("text")
                ans = re.search(r"\[([^\]]*)\]", text).group(1).split("|")[0]
                q = {"type": kind, "speaker": spk, "jbo": text.strip(), "answer": ans}
                if chips:
                    q["chips"] = chips
                questions.append(q)
    return lines, questions, [{"name": k, "gender": v} for k, v in speakers.items()]
