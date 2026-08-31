#!/usr/bin/env python3
"""One-time datafill: import freeforums_dump/ into Lensisku Postgres.

Designed to run inside the web / web-dev Podman container which already has:
  - python3 + python3-psycopg2
  - DB_USER, DB_PASSWORD, DB_NAME env vars (DB_HOST defaults to localhost)

Usage (inside container):
  cd /data/freeforums_dump
  python3 import_dump.py --dry-run
  python3 import_dump.py
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

IMPORT_SOURCE = "freeforums"
USERNAME_SUFFIX = "@lojban.freeforums.net"
DISABLED_PASSWORD = "DISABLED"
MAX_USERNAME_LEN = 64


def _psycopg2():
    try:
        import psycopg2
        import psycopg2.extras  # noqa: F401
    except ImportError as e:
        raise SystemExit(
            "psycopg2 is required (python3-psycopg2 in the web container)."
        ) from e
    return psycopg2


def connect_db() -> Any:
    psycopg2 = _psycopg2()
    host = os.environ.get("DB_HOST", "localhost")
    port = int(os.environ.get("DB_PORT", "5432"))
    user = os.environ.get("DB_USER")
    password = os.environ.get("DB_PASSWORD")
    dbname = os.environ.get("DB_NAME")
    if not all([user, password, dbname]):
        raise SystemExit(
            "Missing DB_USER / DB_PASSWORD / DB_NAME env vars "
            "(set automatically in the web container)."
        )
    return psycopg2.connect(
        host=host,
        port=port,
        user=user,
        password=password,
        dbname=dbname,
    )


def slugify(name: str, max_len: int = 24) -> str:
    out: list[str] = []
    for ch in name:
        if ch.isascii() and ch.isalnum():
            out.append(ch.lower())
        elif not out or out[-1] != "_":
            out.append("_")
    trimmed = "".join(out).strip("_")
    if not trimmed:
        return "anon"
    return trimmed[:max_len]


def make_username(author_name: str, author_id: int) -> str:
    key = f"{author_name}|{author_id}"
    digest = hashlib.md5(key.encode("utf-8")).hexdigest()[:8]
    base = slugify(author_name or "anon")
    # Fit within varchar(64): base_digest@suffix
    suffix = f"_{digest}{USERNAME_SUFFIX}"
    max_base = MAX_USERNAME_LEN - len(suffix)
    if max_base < 1:
        raise RuntimeError("username suffix too long")
    return f"{base[:max_base]}{suffix}"


def load_posts(jsonl_path: Path) -> list[dict[str, Any]]:
    posts: list[dict[str, Any]] = []
    with jsonl_path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            posts.append(json.loads(line))
    return posts


def load_spam_excluded_ids(dump_dir: Path) -> set[int]:
    """Skip IDs listed in spam_excluded.json if present (safety net)."""
    path = dump_dir / "spam_excluded.json"
    if not path.exists():
        return set()
    data = json.loads(path.read_text(encoding="utf-8"))
    ids: set[int] = set()
    for item in data.get("posts") or []:
        if isinstance(item, dict) and "post_id" in item:
            ids.add(int(item["post_id"]))
        elif isinstance(item, int):
            ids.add(item)
    return ids


def already_imported(cur: Any) -> int:
    cur.execute(
        """
        SELECT COUNT(*) FROM comments
        WHERE import_source = %s
        """,
        (IMPORT_SOURCE,),
    )
    return int(cur.fetchone()[0])


def ensure_user(
    cur: Any,
    cache: dict[str, int],
    author_name: str,
    author_id: int,
    stats: dict[str, int],
) -> int:
    key = f"{author_name}|{author_id}"
    if key in cache:
        return cache[key]
    username = make_username(author_name, author_id)
    email = username
    created_at = datetime.now(tz=timezone.utc)
    cur.execute(
        """
        INSERT INTO users (username, email, password, created_at, role, email_confirmed, votesize)
        VALUES (%s, %s, %s, %s, %s, true, %s)
        ON CONFLICT (username)
        DO UPDATE SET username = EXCLUDED.username
        RETURNING userid
        """,
        (username, email, DISABLED_PASSWORD, created_at, "user", 1.0),
    )
    user_id = int(cur.fetchone()[0])
    cache[key] = user_id
    stats["users"] += 1
    return user_id


def rewrite_body(
    body: str,
    lensisku_thread_id: int,
    post_id_map: dict[int, int],
    comment_id: int,
    image_rel_paths: list[str],
) -> str:
    """Rewrite dump-relative image paths and ProBoards post links."""
    rewritten = body

    # Map images/foo.ext → /comments/{comment_id}/media/{index}
    for idx, rel in enumerate(image_rel_paths):
        # Markdown may use images/... or ../images/...
        patterns = [
            f"]({rel})",
            f"](../{rel})",
            f"](./{rel})",
        ]
        media_url = f"](/comments/{comment_id}/media/{idx})"
        for pat in patterns:
            rewritten = rewritten.replace(pat, media_url)

    # Generic images/{postid}_{n}.ext leftovers
    def img_repl(m: re.Match[str]) -> str:
        fname = m.group(1)
        for idx, rel in enumerate(image_rel_paths):
            if rel.endswith(fname) or rel == f"images/{fname}":
                return f"](/comments/{comment_id}/media/{idx})"
        return m.group(0)

    rewritten = re.sub(r"\]\(\.?\.?/?images/([^)]+)\)", img_repl, rewritten)

    # /post/{id} → lensisku comment link
    def post_repl(m: re.Match[str]) -> str:
        legacy = int(m.group(1))
        new_id = post_id_map.get(legacy)
        if new_id is None:
            return m.group(0)
        return (
            f"](/comments/?thread_id={lensisku_thread_id}"
            f"&comment_id={new_id}&scroll_to={new_id})"
        )

    rewritten = re.sub(
        r"\]\((?:https?://lojban\.freeforums\.net)?/post/(\d+)[^)]*\)",
        post_repl,
        rewritten,
    )
    return rewritten


def run_import(dump_dir: Path, dry_run: bool, force: bool) -> None:
    jsonl_path = dump_dir / "posts.jsonl"
    if not jsonl_path.exists():
        raise SystemExit(f"missing {jsonl_path}")

    posts = load_posts(jsonl_path)
    spam_ids = load_spam_excluded_ids(dump_dir)
    if spam_ids:
        before = len(posts)
        posts = [p for p in posts if int(p["post_id"]) not in spam_ids]
        skipped = before - len(posts)
        if skipped:
            print(f"Skipping {skipped} posts listed in spam_excluded.json")
    if not posts:
        raise SystemExit("posts.jsonl is empty")

    conn = connect_db()
    conn.autocommit = False
    stats = {
        "threads": 0,
        "comments": 0,
        "users": 0,
        "media_inserted": 0,
        "missing_media": 0,
    }

    try:
        with conn.cursor() as cur:
            existing = already_imported(cur)
            if existing and not force:
                raise SystemExit(
                    f"Found {existing} existing comments with import_source="
                    f"'{IMPORT_SOURCE}'. Re-run with --force to import anyway "
                    "(may create duplicates)."
                )

            # Group by (board_id, thread_id)
            groups: dict[tuple[int, int], list[dict[str, Any]]] = defaultdict(list)
            for p in posts:
                groups[(int(p["board_id"]), int(p["thread_id"]))].append(p)

            user_cache: dict[str, int] = {}
            post_id_map: dict[int, int] = {}  # legacy post_id → commentid
            # (comment_id, post dict, lensisku_thread_id) for second pass
            rewrite_queue: list[tuple[int, dict[str, Any], int]] = []

            for (_board_id, _legacy_thread_id), thread_posts in sorted(groups.items()):
                thread_posts.sort(
                    key=lambda p: (int(p.get("timestamp") or 0), int(p["post_id"]))
                )
                op = thread_posts[0]
                op_user = ensure_user(
                    cur,
                    user_cache,
                    op.get("author_name") or "anon",
                    int(op.get("author_id") or 0),
                    stats,
                )
                thread_ref = {
                    "board_id": int(op["board_id"]),
                    "board_slug": op.get("board_slug"),
                    "legacy_thread_id": int(op["thread_id"]),
                    "thread_slug": op.get("thread_slug"),
                    "thread_subject": op.get("thread_subject"),
                }
                cur.execute(
                    """
                    INSERT INTO threads (
                        valsiid, natlangwordid, definitionid, definition_link_id,
                        target_user_id, import_source, import_ref
                    )
                    VALUES (NULL, NULL, NULL, NULL, %s, %s, %s)
                    RETURNING threadid
                    """,
                    (op_user, IMPORT_SOURCE, json.dumps(thread_ref)),
                )
                lensisku_thread_id = int(cur.fetchone()[0])
                stats["threads"] += 1

                for idx, post in enumerate(thread_posts):
                    user_id = ensure_user(
                        cur,
                        user_cache,
                        post.get("author_name") or "anon",
                        int(post.get("author_id") or 0),
                        stats,
                    )
                    subject = post.get("thread_subject") if idx == 0 else None
                    if subject is not None:
                        subject = str(subject).strip() or None

                    content_parts: list[dict[str, Any]] = []
                    if subject:
                        content_parts.append({"type": "header", "data": subject})
                    content_parts.append(
                        {"type": "text", "data": post.get("body_markdown") or ""}
                    )
                    image_rels = list(post.get("image_rel_paths") or [])
                    for rel in image_rels:
                        content_parts.append({"type": "image", "data": rel})

                    import_ref = {
                        "board_id": int(post["board_id"]),
                        "board_slug": post.get("board_slug"),
                        "legacy_thread_id": int(post["thread_id"]),
                        "legacy_post_id": int(post["post_id"]),
                        "source_url": post.get("source_url"),
                        "image_rel_paths": image_rels,
                    }
                    ts = int(post.get("timestamp") or 0)
                    if ts > 2_147_483_647:
                        ts = 2_147_483_647
                    comment_num = int(post.get("post_num") or (idx + 1))

                    cur.execute(
                        """
                        INSERT INTO comments (
                            threadid, parentid, userid, commentnum, time,
                            subject, content, import_source, import_ref
                        )
                        VALUES (%s, NULL, %s, %s, %s, %s, %s, %s, %s)
                        RETURNING commentid
                        """,
                        (
                            lensisku_thread_id,
                            user_id,
                            comment_num,
                            ts,
                            subject,
                            json.dumps(content_parts),
                            IMPORT_SOURCE,
                            json.dumps(import_ref),
                        ),
                    )
                    comment_id = int(cur.fetchone()[0])
                    cur.execute(
                        """
                        INSERT INTO comment_counters (comment_id, total_reactions, total_replies)
                        VALUES (%s, 0, 0)
                        ON CONFLICT (comment_id) DO NOTHING
                        """,
                        (comment_id,),
                    )
                    post_id_map[int(post["post_id"])] = comment_id
                    rewrite_queue.append((comment_id, post, lensisku_thread_id))
                    stats["comments"] += 1

                    for rel in image_rels:
                        # rel is like "images/85_0.jpg"
                        img_path = dump_dir / rel
                        if not img_path.exists():
                            # also try basename under images/
                            img_path = dump_dir / "images" / Path(rel).name
                        if img_path.exists():
                            data = img_path.read_bytes()
                            cur.execute(
                                """
                                INSERT INTO comment_media (comment_id, media_type, media_data)
                                VALUES (%s, 'image', %s)
                                """,
                                (comment_id, _psycopg2().Binary(data)),
                            )
                            stats["media_inserted"] += 1
                        else:
                            print(f"missing image: {rel}", file=sys.stderr)
                            stats["missing_media"] += 1

            # Second pass: rewrite markdown links / image URLs
            for comment_id, post, lensisku_thread_id in rewrite_queue:
                subject = None
                # Keep OP header if present
                if post.get("thread_subject") and int(post.get("post_num") or 0) == 1:
                    subject = str(post["thread_subject"]).strip() or None
                body = rewrite_body(
                    post.get("body_markdown") or "",
                    lensisku_thread_id,
                    post_id_map,
                    comment_id,
                    list(post.get("image_rel_paths") or []),
                )
                content_parts = []
                if subject:
                    content_parts.append({"type": "header", "data": subject})
                content_parts.append({"type": "text", "data": body})
                for rel in post.get("image_rel_paths") or []:
                    content_parts.append({"type": "image", "data": rel})
                cur.execute(
                    "UPDATE comments SET content = %s WHERE commentid = %s",
                    (json.dumps(content_parts), comment_id),
                )

        if dry_run:
            conn.rollback()
            print("Dry-run complete (rolled back).")
        else:
            conn.commit()
            print("Import committed.")

        print(
            "threads={threads} comments={comments} users={users} "
            "media={media_inserted} missing_media={missing_media}".format(**stats)
        )
    except Exception:
        conn.rollback()
        raise
    finally:
        conn.close()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Import freeforums_dump into Lensisku Postgres (one-time datafill)."
    )
    parser.add_argument(
        "--dump-dir",
        type=Path,
        default=Path(__file__).resolve().parent,
        help="Path to freeforums_dump/ (default: directory containing this script)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Run import inside a transaction and roll back",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Import even if freeforums rows already exist",
    )
    args = parser.parse_args()
    dump_dir = args.dump_dir.resolve()
    run_import(dump_dir, dry_run=args.dry_run, force=args.force)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
