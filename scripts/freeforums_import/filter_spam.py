#!/usr/bin/env python3
"""Label and remove known spam posts from freeforums_dump/.

Writes spam_excluded.json (audit log) and rewrites posts.jsonl,
threads/**/*.md, and manifest.json without those posts.
Re-numbers post_num within each affected thread.
"""

from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

# Clear commercial / bot spam (no legitimate Lojban content).
# Mostly Russian SEO farms + pharmacy spam; none contain CJK but they are
# the obvious spam wave in this archive.
SPAM_POSTS: dict[int, str] = {
    # thread 23 — Fan translations of media
    97: "russian_seo_casino",
    98: "russian_seo_loans",
    100: "russian_seo_fifa",
    101: "russian_seo_rayban",
    109: "russian_seo_clothing",
    110: "russian_seo_petstore",
    111: "russian_seo_petstore",
    112: "russian_seo_petstore",
    113: "russian_seo_proxy",
    114: "russian_seo_proxy",
    115: "russian_seo_proxy",
    116: "russian_seo_proxy",
    # thread 14 — lonu mi .irci...
    104: "pharmacy_spam",
    105: "empty_spam_adjacent",  # blank guest spam filler
    106: "pharmacy_spam",
    107: "empty_spam_adjacent",
    108: "pharmacy_spam",
    # thread 16 — Créer un phrasebook illustré
    99: "cialis_spam",
}


def slugify_filename(text: str, max_len: int = 60) -> str:
    out = []
    for ch in text.lower():
        if ch.isalnum():
            out.append(ch)
        elif not out or out[-1] != "-":
            out.append("-")
    s = "".join(out).strip("-")
    return (s[:max_len] or "untitled").rstrip("-")


def write_thread_markdown(threads_dir: Path, posts: list[dict[str, Any]]) -> Path:
    p0 = posts[0]
    board_dir = threads_dir / f"board-{p0['board_id']}-{slugify_filename(p0['board_slug'])}"
    board_dir.mkdir(parents=True, exist_ok=True)
    path = board_dir / f"thread-{p0['thread_id']}-{slugify_filename(p0['thread_slug'])}.md"
    subject = p0.get("thread_subject") or f"thread-{p0['thread_id']}"
    lines = [
        "---",
        f"board_id: {p0['board_id']}",
        f"board_slug: {p0['board_slug']}",
        f"thread_id: {p0['thread_id']}",
        f"thread_subject: {json.dumps(subject, ensure_ascii=False)}",
        f"post_count: {len(posts)}",
        "---",
        "",
    ]
    for post in posts:
        ts = int(post.get("timestamp") or 0)
        dt = (
            datetime.fromtimestamp(ts, tz=timezone.utc).strftime("%Y-%m-%d")
            if ts
            else "?"
        )
        lines.append(f"## Post {post['post_id']} — {post['author_name']} — {dt}")
        lines.append("")
        body = (post.get("body_markdown") or "").replace("](images/", "](../images/")
        lines.append(body)
        lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")
    return path


def clean_dump(dump_dir: Path, extra_ids: set[int] | None = None) -> None:
    spam_map = dict(SPAM_POSTS)
    if extra_ids:
        for pid in extra_ids:
            spam_map.setdefault(pid, "manual")

    jsonl_path = dump_dir / "posts.jsonl"
    posts = [
        json.loads(line)
        for line in jsonl_path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    excluded: list[dict[str, Any]] = []
    kept: list[dict[str, Any]] = []
    for p in posts:
        pid = int(p["post_id"])
        if pid in spam_map:
            excluded.append(
                {
                    "post_id": pid,
                    "thread_id": p.get("thread_id"),
                    "board_id": p.get("board_id"),
                    "author_name": p.get("author_name"),
                    "author_id": p.get("author_id"),
                    "timestamp": p.get("timestamp"),
                    "source_url": p.get("source_url"),
                    "label": spam_map[pid],
                    "body_preview": re.sub(r"\s+", " ", (p.get("body_markdown") or ""))[
                        :200
                    ],
                }
            )
        else:
            kept.append(p)

    # Re-number within threads
    groups: dict[tuple[int, int], list[dict[str, Any]]] = defaultdict(list)
    for p in kept:
        groups[(int(p["board_id"]), int(p["thread_id"]))].append(p)
    renumbered: list[dict[str, Any]] = []
    for key in sorted(groups.keys()):
        thread_posts = sorted(
            groups[key], key=lambda x: (int(x.get("timestamp") or 0), int(x["post_id"]))
        )
        for i, p in enumerate(thread_posts, start=1):
            p["post_num"] = i
            renumbered.append(p)

    # Rewrite posts.jsonl
    with jsonl_path.open("w", encoding="utf-8") as f:
        for p in renumbered:
            f.write(json.dumps(p, ensure_ascii=False) + "\n")

    # Rewrite all thread markdown from kept posts
    threads_dir = dump_dir / "threads"
    if threads_dir.exists():
        for old in threads_dir.rglob("thread-*.md"):
            old.unlink()
        # remove empty board dirs later
    threads_dir.mkdir(parents=True, exist_ok=True)
    for key in sorted(groups.keys()):
        thread_posts = sorted(
            groups[key], key=lambda x: (int(x.get("timestamp") or 0), int(x["post_id"]))
        )
        write_thread_markdown(threads_dir, thread_posts)

    # Prune empty board directories
    for board_dir in list(threads_dir.iterdir()):
        if board_dir.is_dir() and not any(board_dir.iterdir()):
            board_dir.rmdir()

    # Audit file
    audit = {
        "removed_at": datetime.now(tz=timezone.utc).isoformat(),
        "removed_count": len(excluded),
        "kept_count": len(renumbered),
        "note": (
            "Commercial SEO / pharmacy spam removed from dump before Lensisku import. "
            "No CJK-script posts were present; labels describe spam type."
        ),
        "posts": sorted(excluded, key=lambda x: x["post_id"]),
    }
    (dump_dir / "spam_excluded.json").write_text(
        json.dumps(audit, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )

    # Update manifest
    manifest_path = dump_dir / "manifest.json"
    if manifest_path.exists():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    else:
        manifest = {}
    # Rebuild thread list from kept posts
    thread_meta: dict[int, dict[str, Any]] = {}
    for p in renumbered:
        tid = int(p["thread_id"])
        if tid not in thread_meta:
            thread_meta[tid] = {
                "thread_id": tid,
                "thread_slug": p.get("thread_slug"),
                "board_id": p.get("board_id"),
                "board_slug": p.get("board_slug"),
                "subject": p.get("thread_subject"),
                "post_count": 0,
            }
        thread_meta[tid]["post_count"] += 1
    manifest["threads"] = [thread_meta[t] for t in sorted(thread_meta)]
    manifest["thread_count"] = len(thread_meta)
    manifest["post_count"] = len(renumbered)
    manifest["spam_excluded_count"] = len(excluded)
    manifest["spam_excluded_file"] = "spam_excluded.json"
    images_dir = dump_dir / "images"
    if images_dir.exists():
        manifest["image_count"] = len([p for p in images_dir.iterdir() if p.is_file()])
    manifest_path.write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )

    print(
        f"Removed {len(excluded)} spam posts; kept {len(renumbered)} posts "
        f"across {len(thread_meta)} threads."
    )
    for e in excluded:
        print(f"  - post {e['post_id']} ({e['label']}) by {e['author_name']}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Remove labeled spam from freeforums dump")
    parser.add_argument(
        "--dump-dir",
        type=Path,
        default=Path(__file__).resolve().parent / "freeforums_dump",
    )
    parser.add_argument(
        "--extra-ids",
        type=str,
        default="",
        help="Comma-separated extra post IDs to exclude",
    )
    args = parser.parse_args()
    extra = {int(x) for x in args.extra_ids.split(",") if x.strip()}
    clean_dump(args.dump_dir.resolve(), extra or None)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
