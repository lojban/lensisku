#!/usr/bin/env python3
"""
Download OGG sounds from La-Lojban/sutysisku-lojban-corpus-downloader (data/sance and
default-data/sance recursively) and import into valsi_sounds. Run from inside Docker with
DATABASE_URL set. Idempotent: re-run overwrites existing sounds (ON CONFLICT DO UPDATE).

Uses the Git Trees API (recursive) to list all files without the Contents API's 1,000-item-
per-directory limit. Tree API supports up to 100,000 entries per request.
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

GITHUB_API = "https://api.github.com/repos/La-Lojban/sutysisku-lojban-corpus-downloader"
RAW_BASE = "https://raw.githubusercontent.com/La-Lojban/sutysisku-lojban-corpus-downloader/main"

SANCE_PREFIXES = ("data/sance/", "default-data/sance/")


def list_ogg_files_via_trees_api() -> list[tuple[str, str]]:
    """List (path, download_url) for every .ogg file under data/sance and default-data/sance
    using the Git Trees API (recursive). Avoids the Contents API 1,000-item-per-dir limit."""
    url = f"{GITHUB_API}/git/trees/main?recursive=1"
    req = urllib.request.Request(url, headers={"Accept": "application/vnd.github.v3+json"})
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            data = resp.read().decode()
    except Exception as e:
        print(f"Error: failed to fetch tree {url}: {e}", file=sys.stderr)
        return []
    try:
        payload = json.loads(data)
    except Exception as e:
        print(f"Error: invalid JSON from tree API: {e}", file=sys.stderr)
        return []
    tree = payload.get("tree")
    if not isinstance(tree, list):
        print("Error: tree API response missing 'tree' array", file=sys.stderr)
        return []
    if payload.get("truncated"):
        print("Warning: Git tree was truncated (repo has >100k entries). Some OGG files may be missing.", file=sys.stderr)
    out: list[tuple[str, str]] = []
    for item in tree:
        if item.get("type") != "blob":
            continue
        path = item.get("path") or ""
        if not path.lower().endswith(".ogg"):
            continue
        if not any(path.startswith(prefix) for prefix in SANCE_PREFIXES):
            continue
        # Prefer raw URL to avoid per-file API calls; same as previous behavior
        download_url = f"{RAW_BASE}/{path}"
        out.append((path, download_url))
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

    paths_to_fetch = list_ogg_files_via_trees_api()
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
