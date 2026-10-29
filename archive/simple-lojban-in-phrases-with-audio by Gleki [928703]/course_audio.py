"""Audio lookup for the Simple Lojban archive: CSV row i ↔ collection JSON item i."""

from __future__ import annotations

import csv
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
CSV_PATH = HERE / "simple-lojban-in-phrases-with-audio [928703].csv"
AUDIO_DIR = HERE / "audio"
_SOUND_RE = re.compile(r"\[sound:([^\]]+)\]")


def audio_files_by_index(csv_path: Path = CSV_PATH, audio_dir: Path = AUDIO_DIR) -> list[Path | None]:
    """First referenced mp3 that exists on disk for each CSV row, else None.

    Rows listed in audio_exclude.json (items whose Lojban was corrected, see add_level_dialogues.py)
    get None: their recording no longer matches the text."""
    import json
    excl_path = HERE / "audio_exclude.json"
    excl = set(json.loads(excl_path.read_text())) if excl_path.exists() else set()
    out: list[Path | None] = []
    with open(csv_path, newline="", encoding="utf-8-sig") as f:
        for row in csv.reader(f):
            found = None
            for name in _SOUND_RE.findall(" ".join(row)):
                path = audio_dir / name
                if path.is_file():
                    found = path
                    break
            out.append(None if len(out) in excl else found)
    return out
