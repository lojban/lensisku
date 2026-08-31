#!/usr/bin/env python3
"""Scrape archived lojban.freeforums.net from Wayback Machine into a portable dump.

Produces freeforums_dump/ with:
  posts.jsonl, manifest.json, threads/**/*.md, images/, import_dump.py
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
import time
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

import httpx
from bs4 import BeautifulSoup
from markdownify import markdownify as html_to_md

SNAPSHOT = "20221203123124"
ORIGIN = "https://lojban.freeforums.net"
WAYBACK_PREFIX = f"https://web.archive.org/web/{SNAPSHOT}/"
WAYBACK_IM = f"https://web.archive.org/web/{SNAPSHOT}im_/"

USER_AGENT = "lensisku-freeforums-scraper/1.0 (+https://lensisku.lojban.org)"
REQUEST_DELAY_SEC = 1.0
MAX_RETRIES = 5

# Skip UI / avatar / smiley images inside posts
SKIP_IMG_RE = re.compile(
    r"(?:/smiley/|/icons/|/bbcode/|/avatar/|/stars/|/chat/|/forum/images/)",
    re.I,
)
POST_ID_RE = re.compile(r"^post-(\d+)$")
THREAD_LINK_RE = re.compile(r"thread-link\s+thread-(\d+)")
BOARD_LINK_RE = re.compile(r"/board/(\d+)/([A-Za-z0-9_-]+)")
THREAD_URL_RE = re.compile(r"/thread/(\d+)/([A-Za-z0-9_-]+)")
THREAD_SUBJECT_RE = re.compile(
    r"\['thread_subject',\s*\"((?:\\.|[^\"\\])*)\"\]"
)
OG_TITLE_RE = re.compile(
    r'<meta\s+property="og:title"\s+content="([^"]*)"',
    re.I,
)
AUTHOR_FROM_TITLE_RE = re.compile(
    r"Post by\s+(.+?)\s+on\s+", re.I
)
USER_CLASS_RE = re.compile(r"\buser-(\d+)\b")
# ASCII-only slugs when Wayback 404s on unicode path segments
THREAD_SLUG_FALLBACKS = {
    16: "cr-er-un-phrasebook-illustr",
}


@dataclass
class PostRecord:
    board_id: int
    board_slug: str
    thread_id: int
    thread_slug: str
    thread_subject: str
    post_id: int
    post_num: int
    timestamp: int
    author_name: str
    author_id: int
    body_markdown: str
    image_rel_paths: list[str] = field(default_factory=list)
    source_url: str = ""


class WaybackClient:
    def __init__(self, delay: float = REQUEST_DELAY_SEC) -> None:
        self.delay = delay
        self._last_request = 0.0
        self.client = httpx.Client(
            headers={"User-Agent": USER_AGENT},
            follow_redirects=True,
            timeout=60.0,
        )

    def close(self) -> None:
        self.client.close()

    def _throttle(self) -> None:
        elapsed = time.monotonic() - self._last_request
        if elapsed < self.delay:
            time.sleep(self.delay - elapsed)

    def get(self, url: str, binary: bool = False) -> bytes | str:
        last_err: Exception | None = None
        for attempt in range(MAX_RETRIES):
            self._throttle()
            try:
                self._last_request = time.monotonic()
                resp = self.client.get(url)
                if resp.status_code == 404:
                    resp.raise_for_status()
                if resp.status_code in (429, 503, 504):
                    wait = min(30, 2**attempt)
                    print(f"  retry {attempt + 1}: HTTP {resp.status_code}, sleep {wait}s", file=sys.stderr)
                    time.sleep(wait)
                    continue
                resp.raise_for_status()
                return resp.content if binary else resp.text
            except httpx.HTTPStatusError as e:
                if e.response is not None and e.response.status_code == 404:
                    raise
                last_err = e
                wait = min(30, 2**attempt)
                print(f"  retry {attempt + 1}: {e}, sleep {wait}s", file=sys.stderr)
                time.sleep(wait)
            except Exception as e:  # noqa: BLE001
                last_err = e
                wait = min(30, 2**attempt)
                print(f"  retry {attempt + 1}: {e}, sleep {wait}s", file=sys.stderr)
                time.sleep(wait)
        raise RuntimeError(f"failed to fetch {url}: {last_err}")


def wayback_url(path_or_url: str) -> str:
    if path_or_url.startswith("https://web.archive.org/"):
        return path_or_url
    if path_or_url.startswith("//"):
        path_or_url = "https:" + path_or_url
    if path_or_url.startswith("http://") or path_or_url.startswith("https://"):
        return WAYBACK_PREFIX + path_or_url
    if not path_or_url.startswith("/"):
        path_or_url = "/" + path_or_url
    return WAYBACK_PREFIX + ORIGIN + path_or_url


def wayback_image_url(src: str) -> str:
    if src.startswith("https://web.archive.org/"):
        # Normalize to im_ rewrite for binary assets
        return re.sub(r"/web/\d+(?:[a-z]+_)?/", f"/web/{SNAPSHOT}im_/", src, count=1)
    if src.startswith("//"):
        src = "https:" + src
    if src.startswith("http://") or src.startswith("https://"):
        return WAYBACK_IM + src
    if not src.startswith("/"):
        src = "/" + src
    return WAYBACK_IM + ORIGIN + src


def unwrap_wayback_href(href: str) -> str:
    """Strip Wayback wrapper from href, return original path or absolute URL."""
    if not href:
        return href
    m = re.search(r"https?://web\.archive\.org/web/\d+(?:[a-z]+_)?/(https?://.+)", href)
    if m:
        return m.group(1)
    m = re.search(r"/web/\d+(?:[a-z]+_)?/(https?://.+)", href)
    if m:
        return m.group(1)
    return href


def extract_proboards_data(html: str) -> dict[str, Any]:
    """Parse proboards.data([['key', value], ...]) into a flat dict."""
    out: dict[str, Any] = {}
    m = re.search(r"proboards\.data\(\[(.+)\]\);?\s*</script>", html, re.S)
    if not m:
        # Fallback: shorter match
        m = re.search(r"proboards\.data\(\[(.+)\]\)", html, re.S)
    if not m:
        return out
    blob = m.group(1)
    # Find ['key', value] pairs — values can be objects/arrays/strings/numbers
    for km in re.finditer(r"\['([^']+)',\s*", blob):
        key = km.group(1)
        start = km.end()
        # Find matching end of this value (comma or closing of array element)
        depth = 0
        i = start
        in_str = False
        str_ch = ""
        escaped = False
        while i < len(blob):
            ch = blob[i]
            if in_str:
                if escaped:
                    escaped = False
                elif ch == "\\":
                    escaped = True
                elif ch == str_ch:
                    in_str = False
            else:
                if ch in ('"', "'"):
                    in_str = True
                    str_ch = ch
                elif ch in "[{(":
                    depth += 1
                elif ch in "]})":
                    depth -= 1
                elif ch == "," and depth == 0:
                    break
                elif ch == "]" and depth < 0:
                    break
            i += 1
        raw = blob[start:i].rstrip().rstrip(",")
        try:
            # JS-ish → JSON-ish
            fixed = raw
            if fixed.startswith("'") and fixed.endswith("'"):
                fixed = json.dumps(fixed[1:-1])
            else:
                fixed = re.sub(r"(?<![\\w$])'", '"', fixed)
                # Only replace remaining single-quoted strings carefully
            out[key] = json.loads(fixed)
        except Exception:  # noqa: BLE001
            out[key] = raw.strip("'\"")
    return out


def slugify_filename(text: str, max_len: int = 60) -> str:
    out = []
    for ch in text.lower():
        if ch.isalnum():
            out.append(ch)
        elif not out or out[-1] != "-":
            out.append("-")
    s = "".join(out).strip("-")
    return (s[:max_len] or "untitled").rstrip("-")


def normalize_origin_link(href: str) -> str:
    href = unwrap_wayback_href(href)
    if href.startswith(ORIGIN):
        return href[len(ORIGIN) :] or "/"
    if href.startswith("http://lojban.freeforums.net"):
        return href[len("http://lojban.freeforums.net") :] or "/"
    return href


def html_message_to_markdown(
    message_el: Any,
    post_id: int,
    images_dir: Path,
    client: WaybackClient,
    downloaded: dict[str, str],
) -> tuple[str, list[str]]:
    """Convert div.message to markdown; download eligible images."""
    rel_paths: list[str] = []
    img_index = 0

    for img in list(message_el.find_all("img")):
        src = img.get("src") or ""
        raw_src = unwrap_wayback_href(src) if "web.archive.org" in src else src
        if not raw_src or SKIP_IMG_RE.search(raw_src):
            img.decompose()
            continue
        # Only keep author-uploaded / content images (proboards storage images or remote)
        fetch_url = wayback_image_url(src if src else raw_src)
        # Deduplicate by original URL key
        key = raw_src.split("?")[0]
        if key in downloaded:
            rel = downloaded[key]
        else:
            try:
                data = client.get(fetch_url, binary=True)
            except Exception as e:  # noqa: BLE001
                print(f"  warn: image fetch failed ({fetch_url}): {e}", file=sys.stderr)
                img.decompose()
                continue
            ext = guess_ext(key, data)
            filename = f"{post_id}_{img_index}{ext}"
            img_index += 1
            dest = images_dir / filename
            dest.write_bytes(data)
            rel = f"images/{filename}"
            downloaded[key] = rel
            print(f"  saved image {rel} ({len(data)} bytes)")
        rel_paths.append(rel)
        # Replace img with a placeholder the markdown converter keeps
        img["src"] = rel
        img["alt"] = img.get("alt") or ""

    # Strip wayback wrappers from remaining <a href>
    for a in message_el.find_all("a", href=True):
        a["href"] = normalize_origin_link(a["href"])

    html = str(message_el)
    md = html_to_md(html, heading_style="ATX", bullets="-")
    md = re.sub(r"\n{3,}", "\n\n", md).strip()
    return md, rel_paths


def guess_ext(url: str, data: bytes) -> str:
    path = urlparse(url).path.lower()
    for ext in (".jpg", ".jpeg", ".png", ".gif", ".webp", ".svg"):
        if path.endswith(ext):
            return ".jpg" if ext == ".jpeg" else ext
    if data[:3] == b"\xff\xd8\xff":
        return ".jpg"
    if data[:8] == b"\x89PNG\r\n\x1a\n":
        return ".png"
    if data[:6] in (b"GIF87a", b"GIF89a"):
        return ".gif"
    if data[:4] == b"RIFF" and data[8:12] == b"WEBP":
        return ".webp"
    return ".bin"


def parse_author(row: Any, pb_post: dict[str, Any] | None) -> tuple[str, int]:
    author_id = 0
    author_name = "Guest"
    classes = " ".join(row.get("class") or [])
    # options_menu often has user-N
    for el in row.find_all(class_=True):
        cm = USER_CLASS_RE.search(" ".join(el.get("class") or []))
        if cm:
            author_id = int(cm.group(1))
            break
    h3 = row.select_one("h3.title")
    if h3:
        text = h3.get_text(" ", strip=True)
        m = AUTHOR_FROM_TITLE_RE.search(text)
        if m:
            author_name = m.group(1).strip()
    # Sidebar username link
    user_link = row.select_one("a.user-link, .mini-profile a[href*='/user/']")
    if user_link:
        name = user_link.get_text(strip=True)
        if name:
            author_name = name
    if pb_post and isinstance(pb_post.get("created_by"), int):
        author_id = pb_post["created_by"] or author_id
    # Guest: Name pattern
    if author_name.lower().startswith("guest:"):
        author_name = author_name.split(":", 1)[1].strip() or "Guest"
        author_id = 0
    return author_name, author_id


def parse_timestamp(row: Any, pb_post: dict[str, Any] | None) -> int:
    abbr = row.select_one("abbr.o-timestamp, abbr.time, abbr[data-timestamp]")
    if abbr and abbr.get("data-timestamp"):
        try:
            ms = int(abbr["data-timestamp"])
            return ms // 1000 if ms > 10_000_000_000 else ms
        except ValueError:
            pass
    if pb_post and pb_post.get("created_on"):
        return int(pb_post["created_on"])
    return 0


def discover_boards(html: str) -> list[tuple[int, str]]:
    boards: dict[int, str] = {}
    for m in BOARD_LINK_RE.finditer(html):
        boards[int(m.group(1))] = m.group(2)
    soup = BeautifulSoup(html, "lxml")
    for a in soup.select("a.js-board__link, a.board-link"):
        href = unwrap_wayback_href(a.get("href") or "")
        m = BOARD_LINK_RE.search(href)
        if m:
            boards[int(m.group(1))] = m.group(2)
    return sorted(boards.items())


def extract_thread_subject(html: str, pb: dict[str, Any], thread_id: int) -> str:
    m = THREAD_SUBJECT_RE.search(html)
    if m:
        try:
            return json.loads(f'"{m.group(1)}"').strip()
        except json.JSONDecodeError:
            return m.group(1).replace('\\"', '"').strip()
    # proboards flat key (may be truncated by naive parser)
    raw = pb.get("thread_subject")
    if isinstance(raw, str) and raw.strip() and "'],['" not in raw:
        return raw.strip()
    page = pb.get("page")
    if isinstance(page, dict):
        thr = page.get("thread") or {}
        if isinstance(thr, dict):
            subj = thr.get("subject_unescaped") or thr.get("subject")
            if subj:
                return BeautifulSoup(str(subj), "lxml").get_text(strip=True)
    if isinstance(page, str) and "subject_unescaped" in page:
        m2 = re.search(r'"subject_unescaped"\s*:\s*"((?:\\.|[^"\\])*)"', page)
        if m2:
            try:
                return json.loads(f'"{m2.group(1)}"').strip()
            except json.JSONDecodeError:
                return m2.group(1).strip()
    m3 = OG_TITLE_RE.search(html)
    if m3:
        title = BeautifulSoup(f"<t>{m3.group(1)}</t>", "lxml").get_text()
        # "Subject | lojban Forums"
        if " | " in title:
            title = title.rsplit(" | ", 1)[0]
        return title.strip()
    return f"thread-{thread_id}"


def discover_threads(html: str) -> list[tuple[int, str]]:
    threads: dict[int, str] = {}
    for m in THREAD_URL_RE.finditer(html):
        threads[int(m.group(1))] = m.group(2)
    soup = BeautifulSoup(html, "lxml")
    for a in soup.select("a.thread-link, a.js-thread__link"):
        href = unwrap_wayback_href(a.get("href") or "")
        m = THREAD_URL_RE.search(href)
        if m:
            threads[int(m.group(1))] = m.group(2)
            continue
        m2 = re.search(r"/threads/recent/(\d+)", href)
        if m2:
            tid = int(m2.group(1))
            slug = slugify_filename(a.get_text(strip=True) or str(tid))
            # Prefer ASCII slug from TEXT only if we have no URL slug yet
            threads.setdefault(tid, slug)
    for m in THREAD_LINK_RE.finditer(html):
        tid = int(m.group(1))
        threads.setdefault(tid, THREAD_SLUG_FALLBACKS.get(tid, str(tid)))
    # Apply known ASCII fallbacks
    for tid, slug in THREAD_SLUG_FALLBACKS.items():
        if tid in threads:
            threads[tid] = slug
    return sorted(threads.items())


def scrape_thread_pages(
    client: WaybackClient,
    thread_id: int,
    thread_slug: str,
    board_id: int,
    board_slug: str,
    images_dir: Path,
    downloaded_images: dict[str, str],
) -> tuple[str, list[PostRecord]]:
    posts: list[PostRecord] = []
    seen_post_ids: set[int] = set()
    thread_subject = f"thread-{thread_id}"
    slug_candidates = [thread_slug]
    if thread_id in THREAD_SLUG_FALLBACKS:
        fb = THREAD_SLUG_FALLBACKS[thread_id]
        if fb not in slug_candidates:
            slug_candidates.append(fb)
    # Also try bare /thread/{id}
    active_slug = slug_candidates[0]
    page = 1
    while True:
        if page == 1:
            tried = False
            html = None
            last_err: Exception | None = None
            for slug in slug_candidates + [""]:
                path = f"/thread/{thread_id}/{slug}" if slug else f"/thread/{thread_id}"
                path = path.rstrip("/")
                url = wayback_url(ORIGIN + path)
                print(f"  thread {thread_id} page {page}: {url}")
                try:
                    html = client.get(url)
                    assert isinstance(html, str)
                    active_slug = slug or active_slug
                    tried = True
                    break
                except Exception as e:  # noqa: BLE001
                    last_err = e
                    continue
            if not tried or html is None:
                raise RuntimeError(f"thread {thread_id}: {last_err}")
        else:
            path = f"/thread/{thread_id}/{active_slug}?page={page}"
            url = wayback_url(ORIGIN + path)
            print(f"  thread {thread_id} page {page}: {url}")
            html = client.get(url)
            assert isinstance(html, str)

        soup = BeautifulSoup(html, "lxml")
        pb = extract_proboards_data(html)
        thread_subject = extract_thread_subject(html, pb, thread_id)
        page_info = pb.get("page") or {}
        if isinstance(page_info, str):
            # Try to pull board id from serialized page JSON
            bm = re.search(r'"board"\s*:\s*\{[^}]*"id"\s*:\s*(\d+)', page_info)
            if bm:
                board_id = int(bm.group(1))
            um = re.search(r'"url"\s*:\s*"/board/\d+/([A-Za-z0-9_-]+)"', page_info)
            if um:
                board_slug = um.group(1)
        elif isinstance(page_info, dict):
            brd = page_info.get("board") or {}
            if isinstance(brd, dict) and brd.get("id"):
                board_id = int(brd["id"])
            if isinstance(brd, dict) and brd.get("url"):
                m = BOARD_LINK_RE.search(str(brd["url"]))
                if m:
                    board_slug = m.group(2)

        pb_posts = pb.get("proboards.post") or {}
        if not isinstance(pb_posts, dict):
            pb_posts = {}

        rows = soup.select("tr.item.post.js-post, tr.post.js-post, tr[id^=post-]")
        new_on_page = 0
        for row in rows:
            rid = row.get("id") or ""
            m = POST_ID_RE.match(rid)
            if not m:
                continue
            post_id = int(m.group(1))
            if post_id in seen_post_ids:
                continue
            seen_post_ids.add(post_id)
            new_on_page += 1
            pb_post = pb_posts.get(str(post_id)) or pb_posts.get(post_id) or {}
            if not isinstance(pb_post, dict):
                pb_post = {}
            author_name, author_id = parse_author(row, pb_post)
            ts = parse_timestamp(row, pb_post)
            message = row.select_one("div.message")
            if message is None:
                body_md, imgs = "", []
            else:
                body_md, imgs = html_message_to_markdown(
                    message, post_id, images_dir, client, downloaded_images
                )
            posts.append(
                PostRecord(
                    board_id=board_id,
                    board_slug=board_slug,
                    thread_id=thread_id,
                    thread_slug=active_slug or thread_slug,
                    thread_subject=thread_subject,
                    post_id=post_id,
                    post_num=0,
                    timestamp=ts,
                    author_name=author_name,
                    author_id=author_id,
                    body_markdown=body_md,
                    image_rel_paths=imgs,
                    source_url=f"{ORIGIN}/post/{post_id}",
                )
            )

        if new_on_page == 0:
            break
        next_link = soup.select_one(f'a[href*="page={page + 1}"]')
        pagination_pages = [
            int(x)
            for x in re.findall(rf"/thread/{thread_id}/[^\"'?]+\?page=(\d+)", html)
        ]
        if next_link or (pagination_pages and max(pagination_pages, default=page) > page):
            page += 1
            continue
        if new_on_page >= 15:
            page += 1
            continue
        break

    posts.sort(key=lambda p: (p.timestamp, p.post_id))
    for i, p in enumerate(posts, start=1):
        p.post_num = i
        p.thread_subject = thread_subject
        p.board_id = board_id
        p.board_slug = board_slug
        p.thread_slug = active_slug or thread_slug
    return thread_subject, posts


def write_thread_markdown(
    out_dir: Path,
    posts: list[PostRecord],
    thread_subject: str,
) -> Path:
    if not posts:
        raise ValueError("no posts")
    p0 = posts[0]
    board_dir = out_dir / f"board-{p0.board_id}-{slugify_filename(p0.board_slug)}"
    board_dir.mkdir(parents=True, exist_ok=True)
    path = board_dir / f"thread-{p0.thread_id}-{slugify_filename(p0.thread_slug)}.md"
    lines = [
        "---",
        f"board_id: {p0.board_id}",
        f"board_slug: {p0.board_slug}",
        f"thread_id: {p0.thread_id}",
        f"thread_subject: {json.dumps(thread_subject, ensure_ascii=False)}",
        f"post_count: {len(posts)}",
        "---",
        "",
    ]
    for post in posts:
        dt = datetime.fromtimestamp(post.timestamp, tz=timezone.utc).strftime("%Y-%m-%d") if post.timestamp else "?"
        lines.append(f"## Post {post.post_id} — {post.author_name} — {dt}")
        lines.append("")
        # Rewrite images/ → ../images/ for thread-relative paths
        body = post.body_markdown
        body = body.replace("](images/", "](../images/")
        lines.append(body)
        lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")
    return path


def main() -> int:
    parser = argparse.ArgumentParser(description="Scrape lojban.freeforums.net from Wayback")
    script_dir = Path(__file__).resolve().parent
    parser.add_argument(
        "--out",
        type=Path,
        default=script_dir / "freeforums_dump",
        help="Output dump directory",
    )
    parser.add_argument("--delay", type=float, default=REQUEST_DELAY_SEC)
    parser.add_argument(
        "--boards",
        type=str,
        default="",
        help="Comma-separated board IDs to scrape (default: all)",
    )
    args = parser.parse_args()
    out: Path = args.out.resolve()
    images_dir = out / "images"
    threads_dir = out / "threads"
    images_dir.mkdir(parents=True, exist_ok=True)
    threads_dir.mkdir(parents=True, exist_ok=True)

    # Copy import script into dump so it travels with the folder
    import_src = script_dir / "import_dump.py"
    if import_src.exists():
        shutil.copy2(import_src, out / "import_dump.py")

    client = WaybackClient(delay=args.delay)
    errors: list[str] = []
    all_posts: list[PostRecord] = []
    downloaded_images: dict[str, str] = {}
    thread_manifest: list[dict[str, Any]] = []

    try:
        forum_url = wayback_url(ORIGIN + "/forum")
        print(f"Fetching forum index: {forum_url}")
        forum_html = client.get(forum_url)
        assert isinstance(forum_html, str)
        boards = discover_boards(forum_html)
        if args.boards:
            allow = {int(x) for x in args.boards.split(",") if x.strip()}
            boards = [(b, s) for b, s in boards if b in allow]
        print(f"Found {len(boards)} boards: {boards}")

        thread_map: dict[int, tuple[str, int, str]] = {}  # tid -> (slug, board_id, board_slug)
        for board_id, board_slug in boards:
            board_url = wayback_url(ORIGIN + f"/board/{board_id}/{board_slug}")
            print(f"Board {board_id}/{board_slug}: {board_url}")
            try:
                board_html = client.get(board_url)
                assert isinstance(board_html, str)
            except Exception as e:  # noqa: BLE001
                errors.append(f"board {board_id}: {e}")
                continue
            for tid, tslug in discover_threads(board_html):
                thread_map.setdefault(tid, (tslug, board_id, board_slug))

        print(f"Discovered {len(thread_map)} unique threads")

        for tid, (tslug, board_id, board_slug) in sorted(thread_map.items()):
            try:
                subject, posts = scrape_thread_pages(
                    client,
                    tid,
                    tslug,
                    board_id,
                    board_slug,
                    images_dir,
                    downloaded_images,
                )
                if not posts:
                    errors.append(f"thread {tid}: no posts parsed")
                    continue
                write_thread_markdown(threads_dir, posts, subject)
                all_posts.extend(posts)
                thread_manifest.append(
                    {
                        "thread_id": tid,
                        "thread_slug": tslug,
                        "board_id": board_id,
                        "board_slug": board_slug,
                        "subject": subject,
                        "post_count": len(posts),
                    }
                )
                print(f"  -> {len(posts)} posts: {subject!r}")
            except Exception as e:  # noqa: BLE001
                errors.append(f"thread {tid}: {e}")
                print(f"  ERROR thread {tid}: {e}", file=sys.stderr)

        # Deduplicate posts by post_id (same post shouldn't appear twice)
        by_id: dict[int, PostRecord] = {}
        for p in all_posts:
            by_id[p.post_id] = p
        unique_posts = sorted(by_id.values(), key=lambda p: (p.board_id, p.thread_id, p.timestamp, p.post_id))

        # Re-number post_num within each thread
        from collections import defaultdict

        groups: dict[tuple[int, int], list[PostRecord]] = defaultdict(list)
        for p in unique_posts:
            groups[(p.board_id, p.thread_id)].append(p)
        for posts in groups.values():
            posts.sort(key=lambda x: (x.timestamp, x.post_id))
            for i, p in enumerate(posts, start=1):
                p.post_num = i

        jsonl_path = out / "posts.jsonl"
        with jsonl_path.open("w", encoding="utf-8") as f:
            for p in unique_posts:
                f.write(json.dumps(asdict(p), ensure_ascii=False) + "\n")

        manifest = {
            "snapshot": SNAPSHOT,
            "origin": ORIGIN,
            "scraped_at": datetime.now(tz=timezone.utc).isoformat(),
            "boards": [{"board_id": b, "board_slug": s} for b, s in boards],
            "threads": thread_manifest,
            "thread_count": len(thread_manifest),
            "post_count": len(unique_posts),
            "image_count": len(list(images_dir.glob("*"))),
            "errors": errors,
        }
        (out / "manifest.json").write_text(
            json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        print(
            f"Done: {manifest['thread_count']} threads, {manifest['post_count']} posts, "
            f"{manifest['image_count']} images -> {out}"
        )
        if errors:
            print(f"Errors ({len(errors)}):", file=sys.stderr)
            for e in errors:
                print(f"  - {e}", file=sys.stderr)
        return 0 if not errors or unique_posts else 1
    finally:
        client.close()


if __name__ == "__main__":
    raise SystemExit(main())
