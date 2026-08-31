# FreeForums archive import (lojban.freeforums.net via Wayback Machine)

One-time, offline-first workflow:

1. **Prepare** (on a machine with network): scrape Wayback → portable `freeforums_dump/`
2. **Filter spam** (local): remove labeled commercial SEO / pharmacy spam
3. **Import** (inside Lensisku **web** / **web-dev** Podman container): copy dump → run `import_dump.py`

No SQL migrations. Uses existing `import_source` / `import_ref` / `comment_media` columns.

## Prepare dump (local)

```bash
cd scripts/freeforums_import
uv venv .venv
uv pip install --python .venv/bin/python -r requirements.txt
uv run --python .venv/bin/python scrape_forum.py --out freeforums_dump
uv run --python .venv/bin/python filter_spam.py --dump-dir freeforums_dump
```

`filter_spam.py` removes known bot spam (Russian SEO, pharmacy, Cialis, etc.),
writes `spam_excluded.json` (audit labels), and rewrites `posts.jsonl` /
`threads/**/*.md` / `manifest.json`. There were no CJK-script posts in this
archive; the removals are the clear commercial spam wave.

Output layout:

```
freeforums_dump/
  manifest.json
  posts.jsonl
  spam_excluded.json  # labeled removals (not imported)
  import_dump.py      # travels with the dump
  threads/
    board-2-lojban-forum/
      thread-23-….md
  images/
    …
```

Expect **22 threads** and about **87** non-spam posts after filtering.
Check `manifest.json` and `spam_excluded.json`.

## Import on prod / dev (Podman)

The **web** image has `python3` + `python3-psycopg2` and `DB_*` env vars.
There is **no Rust toolchain** in the container — do not use `cargo run`.

Test on **web-dev** first, then prod **web**.

```bash
# Copy dump into the container's /data mount
podman cp scripts/freeforums_import/freeforums_dump/ <web-container>:/data/freeforums_dump/

# Dry run (rolls back)
podman exec <web-container> python3 /data/freeforums_dump/import_dump.py --dry-run

# Real import (one-time)
podman exec <web-container> python3 /data/freeforums_dump/import_dump.py
```

Interactive:

```bash
podman exec -it <web-container> bash
cd /data/freeforums_dump
python3 import_dump.py --dry-run
python3 import_dump.py
```

If freeforums rows already exist, the script exits unless you pass `--force`
(which can create duplicates). `import_dump.py` also skips any IDs still listed
in `spam_excluded.json` as a safety net.

### Verification

```sql
SELECT import_source, COUNT(*) FROM threads GROUP BY import_source;
SELECT import_source, COUNT(*) FROM comments GROUP BY import_source;
SELECT COUNT(*) FROM comment_media;
SELECT username FROM users WHERE username LIKE '%@lojban.freeforums.net' LIMIT 10;
```

Authors appear as `{name}_{hash8}@lojban.freeforums.net` with login disabled.

### Notes

- Inline image markdown points at `/comments/{comment_id}/media/{index}`. Serving those
  URLs needs a separate app endpoint if not already present; binaries are still stored
  in `comment_media`.
- Scraper rate-limits Wayback (~1 req/s). Full scrape is a few minutes.
- To exclude more posts later: edit `SPAM_POSTS` in `filter_spam.py` or pass
  `--extra-ids 123,456` and re-run the filter on the dump.
