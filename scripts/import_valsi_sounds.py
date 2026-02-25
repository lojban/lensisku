#!/usr/bin/env python3
"""
Download OGG sounds from La-Lojban/sutysisku-lojban-corpus-downloader (data/sance and
default-data/sance recursively) and import into valsi_sounds. Run from inside Docker with
DATABASE_URL set. Idempotent: re-run overwrites existing sounds (ON CONFLICT DO UPDATE).
"""
from __future__ import annotations

import json
import os
import sys
import urllib.request
from pathlib import Path

try:
    import psycopg2
except ImportError:
    print("psycopg2 required. Install with: apt-get install python3-psycopg2", file=sys.stderr)
    sys.exit(1)

GITHUB_API = "https://api.github.com/repos/La-Lojban/sutysisku-lojban-corpus-downloader/contents"
RAW_BASE = "https://raw.githubusercontent.com/La-Lojban/sutysisku-lojban-corpus-downloader/main"


def list_files_recursive(prefix: str) -> list[tuple[str, str]]:
    """List (path, download_url) for every file under prefix. prefix is e.g. 'data/sance'."""
    out: list[tuple[str, str]] = []
    to_visit = [prefix]
    while to_visit:
        path = to_visit.pop()
        url = f"{GITHUB_API}/{path}"
        req = urllib.request.Request(url, headers={"Accept": "application/vnd.github.v3+json"})
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                data = resp.read().decode()
        except Exception as e:
            print(f"Warning: failed to list {url}: {e}", file=sys.stderr)
            continue
        try:
            items = json.loads(data)
        except Exception as e:
            print(f"Warning: invalid JSON from {url}: {e}", file=sys.stderr)
            continue
        if not isinstance(items, list):
            items = [items]
        for item in items:
            name = item.get("name") or ""
            if item.get("type") == "dir":
                to_visit.append(f"{path}/{name}".rstrip("/"))
            elif item.get("type") == "file" and name.lower().endswith(".ogg"):
                dl = item.get("download_url")
                if dl:
                    out.append((f"{path}/{name}", dl))
    return out


def download(url: str) -> bytes | None:
    try:
        with urllib.request.urlopen(url, timeout=60) as resp:
            return resp.read()
    except Exception as e:
        print(f"Warning: download failed {url}: {e}", file=sys.stderr)
        return None


def main() -> int:
    database_url = os.environ.get("DATABASE_URL")
    if not database_url:
        print("Set DATABASE_URL (e.g. postgres://user:pass@host/db)", file=sys.stderr)
        return 1

    paths_to_fetch = []
    for prefix in ("data/sance", "default-data/sance"):
        paths_to_fetch.extend(list_files_recursive(prefix))

    print(f"Found {len(paths_to_fetch)} OGG file(s). Downloading and importing...")

    conn = psycopg2.connect(database_url)
    cur = conn.cursor()
    inserted = 0
    skipped = 0
    errors = 0

    for rel_path, download_url in paths_to_fetch:
        stem = Path(rel_path).stem
        if not stem:
            continue
        data = download(download_url)
        if not data:
            errors += 1
            continue
        cur.execute(
            """
            INSERT INTO valsi_sounds (valsi_id, sound_data, mime_type)
            SELECT v.valsiid, %s, 'audio/ogg'
            FROM valsi v
            WHERE LOWER(v.word) = LOWER(%s)
              AND v.source_langid = 1
            LIMIT 1
            ON CONFLICT (valsi_id) DO UPDATE
            SET sound_data = EXCLUDED.sound_data, mime_type = EXCLUDED.mime_type
            """,
            (data, stem),
        )
        if cur.rowcount:
            inserted += 1
        else:
            skipped += 1

    conn.commit()
    cur.close()
    conn.close()

    print(f"Done. Inserted/updated: {inserted}, no matching valsi: {skipped}, errors: {errors}.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
