#!/usr/bin/env python3
"""Import the CLL as navigable native wiki pages (TOC, chapters, and sections)."""
from __future__ import annotations
import argparse, base64, getpass, hashlib, json, os, re, shlex, subprocess, sys
from dataclasses import dataclass
from pathlib import Path
from urllib.error import HTTPError
from urllib.parse import quote, urlparse
from urllib.request import Request, urlopen

SECTION = re.compile(r'<span id="sec-(\d+)-(\d+)"></span>')
IDS = re.compile(r'<span id="([^"]+)"></span>')
LINK = re.compile(r"(?:(?:https?://[^/ )]+)?/)?(?:en/)?books/cll/(\d+)(?:/)?(?:#([^\s)\"']*))?")
@dataclass
class Page:
    title: str
    body: str

def b64(value: str) -> str:
    return base64.b64encode(value.encode()).decode()

def make_pages(source: Path) -> list[Page]:
    files = sorted((p for p in source.glob('*.md') if p.stem.isdigit() and 1 <= int(p.stem) <= 21), key=lambda p: int(p.stem))
    if len(files) != 21:
        raise ValueError(f'expected CLL chapters 1–21 in {source}, found {len(files)}')
    chapter_titles: dict[int, str] = {}
    sections: list[tuple[int, int, str, str]] = []
    chapter_bodies: dict[int, str] = {}
    anchor_to_title: dict[str, str] = {}
    for path in files:
        chapter = int(path.stem)
        text = path.read_text(encoding='utf-8')
        heading = re.search(r'^## Chapter\s+\d+\.\s*(.+)$', text, re.M)
        chapter_titles[chapter] = heading.group(1).strip() if heading else f'Chapter {chapter}'
        matches = list(SECTION.finditer(text))
        if not matches:
            raise ValueError(f'no numbered sections in {path}')
        chapter_bodies[chapter] = text[:matches[0].start()].strip()
        for index, match in enumerate(matches):
            ch, sec = map(int, match.groups())
            end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
            body = text[match.start():end].strip()
            section_heading = re.search(r'^###\s+(.+?)\s*#*$', body, re.M)
            label = section_heading.group(1).strip() if section_heading else f'Section {ch}.{sec}'
            title = f'Chapter {ch}. Section {sec}'
            sections.append((ch, sec, title, body))
            for anchor in IDS.findall(body):
                anchor_to_title[anchor] = title
        for anchor in IDS.findall(chapter_bodies[chapter]):
            anchor_to_title[anchor] = f'Chapter {chapter}'

    # Rewrite the original corpus links using exact source anchor ownership.
    def fix_links(body: str, owner_title: str) -> str:
        # An anchor-only HTML line immediately followed by a Markdown
        # blockquote makes CommonMark treat the quote marker as part of the
        # same HTML block. Marked then emits a literal `&gt;` inside a <p>.
        # Keep the anchors as HTML, but start the quote in a fresh Markdown
        # block so examples render as semantic <blockquote> elements.
        body = re.sub(
            r'(?m)^((?:<span id="[^"]+"></span>)+)\n(?=>)',
            r'\1\n\n',
            body,
        )
        # CLL examples are written as consecutive Markdown blockquote lines. A
        # normal Markdown soft break renders as a space, which collapses the
        # Lojban line, literal gloss, and translation into one run-on line.
        # Emit explicit breaks because the wiki Markdown renderer normalizes
        # trailing spaces inside blockquotes before parsing them.
        body = re.sub(r'(?m)^(>[^\n]*)(\n)(?=>)', r'\1<br>\2', body)

        def replace(match: re.Match[str]) -> str:
            chapter = int(match.group(1))
            fragment = match.group(2)
            title = anchor_to_title.get(fragment) if fragment else None
            if fragment and title is None:
                owner_chapter = re.match(r'Chapter (\d+)', owner_title)
                title = owner_title if owner_chapter and int(owner_chapter.group(1)) == chapter else f'Chapter {chapter}'
                fragment = None  # Source references a missing ID; land on the closest useful page.
            title = title or f'Chapter {chapter}'
            url = '/wiki/' + title.replace(' ', '_')
            if fragment:
                url += '#' + fragment
            return url
        return LINK.sub(replace, body)

    pages: list[Page] = []
    toc_title = 'CLL Table of Contents'
    toc = ['# The Complete Lojban Language', '', 'Select a chapter or a numbered section.']
    for chapter in sorted(chapter_titles):
        toc += ['', f'## [Chapter {chapter}: {chapter_titles[chapter]}](/wiki/Chapter_{chapter})']
        for ch, sec, title, body in sections:
            if ch != chapter:
                continue
            label_match = re.search(r'^###\s+(.+?)\s*#*$', body, re.M)
            label = label_match.group(1).strip() if label_match else f'Section {ch}.{sec}'
            toc.append(f'- [{ch}.{sec} {label}](/wiki/{title.replace(" ", "_")})')
    pages.append(Page(toc_title, '\n'.join(toc)))

    # Chapter landing pages preserve chapter-level targets and provide entry navigation.
    for chapter in sorted(chapter_titles):
        chapter_sections = [item for item in sections if item[0] == chapter]
        first = chapter_sections[0][2]
        nav = f'[← Table of Contents](/wiki/CLL_Table_of_Contents) · [Start Chapter {chapter}: Section {chapter_sections[0][1]} →](/wiki/{first.replace(" ", "_")})'
        body = fix_links(chapter_bodies[chapter], f'Chapter {chapter}')
        pages.append(Page(f'Chapter {chapter}', f'{nav}\n\n{body}\n\n{nav}'))

    for index, (chapter, number, title, body) in enumerate(sections):
        previous = sections[index - 1][2] if index else None
        following = sections[index + 1][2] if index + 1 < len(sections) else None
        links = []
        if previous:
            links.append(f'[← Previous: {previous}](/wiki/{previous.replace(" ", "_")})')
        links.append('[Contents](/wiki/CLL_Table_of_Contents)')
        if following:
            links.append(f'[Next: {following} →](/wiki/{following.replace(" ", "_")})')
        nav = ' · '.join(links)
        pages.append(Page(title, f'{nav}\n\n{fix_links(body, title)}\n\n---\n\n{nav}'))
    return pages

def import_via_api(pages: list[Page], api_url: str, username: str | None) -> int:
    """Use native wiki upserts; read each saved page back before continuing."""
    api_url = api_url.rstrip('/')
    if urlparse(api_url).scheme != 'https' and urlparse(api_url).hostname not in ('localhost', '127.0.0.1'):
        raise ValueError('remote API imports require HTTPS')
    token = os.environ.get('CLL_ACCESS_TOKEN')

    def request(method: str, path: str, payload: dict | None = None, missing_ok: bool = False):
        headers = {'Accept': 'application/json', 'User-Agent': 'Lensisku-CLL-Importer/1.0'}
        if token:
            headers['Authorization'] = 'Bearer ' + token
        data = None
        if payload is not None:
            headers['Content-Type'] = 'application/json'
            data = json.dumps(payload).encode('utf-8')
        try:
            with urlopen(Request(api_url + path, data=data, headers=headers, method=method), timeout=120) as response:
                return json.load(response)
        except HTTPError as error:
            if missing_ok and error.code == 404:
                return None
            # Do not print responses from authentication, which could contain secrets.
            challenge = error.headers.get('cf-mitigated') == 'challenge'
            detail = ' (Cloudflare challenge blocks API access)' if challenge else ''
            if path != '/auth/login' and not challenge:
                try:
                    response_detail = error.read(2048).decode('utf-8', errors='replace')
                    try:
                        parsed_detail = json.loads(response_detail)
                        response_detail = str(parsed_detail.get('error') or parsed_detail.get('message') or response_detail)
                    except (ValueError, AttributeError):
                        pass
                    response_detail = re.sub(r'\s+', ' ', response_detail).strip()[:500]
                    if response_detail:
                        detail += f' ({response_detail})'
                except OSError:
                    pass
            raise RuntimeError(f'{method} {path}: HTTP {error.code}{detail}') from None

    # Check API reachability before asking for credentials or writing pages.
    languages = request('GET', '/language/languages')
    if not any(item.get('id', item.get('langid')) == 2 for item in languages):
        raise RuntimeError('API does not expose English language ID 2')
    if not token:
        if not username:
            raise ValueError('pass --username or set CLL_USERNAME (or CLL_ACCESS_TOKEN)')
        password = os.environ.get('CLL_PASSWORD')
        if not password:
            if not sys.stdin.isatty():
                raise ValueError('a password prompt requires a terminal; set CLL_PASSWORD or CLL_ACCESS_TOKEN for automation')
            password = getpass.getpass('Lensisku password: ')
        auth = request('POST', '/auth/login', {'username_or_email': username, 'password': password})
        token = auth['access_token']
        del password, auth

    saved = skipped = 0
    for index, page in enumerate(pages, 1):
        path = '/jbovlaste/valsi/' + quote(page.title, safe='') + '/wiki'
        existing = request('GET', path, missing_ok=True)
        digest = hashlib.sha256(page.body.encode('utf-8')).hexdigest()
        if existing and (existing.get('metadata') or {}).get('source') != 'CLL':
            raise RuntimeError(f'{page.title}: existing page is not marked as a CLL import; refusing to overwrite')
        existing_metadata = existing.get('metadata') or {} if existing else {}
        source_anchors = set(IDS.findall(page.body))
        stored_anchors = set(IDS.findall(existing.get('definition', ''))) if existing else set()
        expected_quotes = sum(line.startswith('>') for line in page.body.splitlines())
        stored_definition = existing.get('definition', '') if existing else ''
        stored_quotes = sum(line.startswith('>') for line in stored_definition.splitlines())
        quotes_preserved = stored_quotes >= expected_quotes
        if existing and source_anchors.issubset(stored_anchors) and quotes_preserved and (
            existing_metadata.get('cll_import_sha256') == digest
            or existing.get('definition') == page.body
        ):
            skipped += 1
            print(f'[{index}/{len(pages)}] Unchanged: {page.title}', flush=True)
            continue
        payload = {
            'word': page.title, 'definition': page.body, 'lang_id': 2,
            'source_langid': 1, 'is_wiki': True, 'owner_only': False,
            'metadata': {
                'source': 'CLL', 'import': 'cll-markdown', 'cll_import_sha256': digest,
            },
            'commit_message': 'Imported CLL content',
        }
        if existing:
            payload['expected_time'] = existing['time']
        result = request('POST', '/jbovlaste/valsi', payload)
        if not result.get('success'):
            raise RuntimeError(f'{page.title}: API rejected import: {result.get("error", "unknown error")}')
        stored = request('GET', path)
        if stored.get('definitionid') != result['definition_id'] or stored.get('valsiword') != page.title:
            raise RuntimeError(f'{page.title}: saved page identity verification failed')
        if not set(IDS.findall(page.body)).issubset(set(IDS.findall(stored['definition']))):
            raise RuntimeError(f'{page.title}: API did not preserve the source anchors')
        expected_quotes = sum(line.startswith('>') for line in page.body.splitlines())
        stored_quotes = sum(line.startswith('>') for line in stored['definition'].splitlines())
        if stored_quotes < expected_quotes:
            raise RuntimeError(f'{page.title}: API did not preserve Markdown blockquote markers')
        saved += 1
        print(f'[{index}/{len(pages)}] Imported and verified: {page.title}', flush=True)
    print(f'Complete: {saved} imported, {skipped} unchanged, {len(pages)} total.', flush=True)
    return 0

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path('/home/user/lojban/korpora2/cll'))
    parser.add_argument('--dsn', default=os.environ.get('DATABASE_URL'))
    parser.add_argument('--psql-command', default=os.environ.get('CLL_PSQL_COMMAND'), help='full psql command, for example: docker exec -i lenpostgres psql -U lojban -d lojban_lens')
    parser.add_argument('--user-id', type=int, default=int(os.environ.get('CLL_IMPORT_USER_ID', '1')))
    parser.add_argument('--dry-run', action='store_true', help='print SQL or the API import plan without connecting')
    parser.add_argument('--api-url', help='API base URL, e.g. https://lensisku-dev.lojban.org/api')
    parser.add_argument('--username', default=os.environ.get('CLL_USERNAME'))
    args = parser.parse_args()
    pages = make_pages(args.source)
    if args.api_url:
        if args.dry_run:
            print(json.dumps({'api_url': args.api_url, 'pages': [page.title for page in pages]}, indent=2))
            return 0
        try:
            return import_via_api(pages, args.api_url, args.username)
        except (RuntimeError, ValueError, OSError) as error:
            print(f'Import stopped: {error}. Re-running resumes by skipping unchanged pages.', file=sys.stderr)
            return 1
    if not args.dry_run and not args.dsn and not args.psql_command:
        parser.error('set DATABASE_URL, pass --dsn, or provide --psql-command')
    sql = [
        'BEGIN;',
        "SET LOCAL client_encoding = 'UTF8';",
        "UPDATE valsi v SET word = CASE "
        "WHEN v.word ~ '^CLL [0-9]+\\.[0-9]+$' THEN regexp_replace(v.word, '^CLL ([0-9]+)\\.([0-9]+)$', 'Chapter \\1. Section \\2') "
        "ELSE regexp_replace(v.word, '^CLL ([0-9]+)$', 'Chapter \\1') END "
        "WHERE v.typeid=16 AND v.source_langid=1 AND (v.word ~ '^CLL [0-9]+$' OR v.word ~ '^CLL [0-9]+\\.[0-9]+$') "
        "AND EXISTS (SELECT 1 FROM definitions d WHERE d.valsiid=v.valsiid AND d.metadata->>'source'='CLL');",
    ]
    for index, page in enumerate(pages, 1):
        content = b64(page.body)
        digest = hashlib.sha256(page.body.encode('utf-8')).hexdigest()
        title = page.title.replace("'", "''")
        sql += [
            f"INSERT INTO valsi (word,typeid,userid,time,source_langid) VALUES ('{title}',16,{args.user_id},extract(epoch from now())::int,1) ON CONFLICT (word,source_langid) WHERE typeid=16 DO UPDATE SET word=EXCLUDED.word RETURNING valsiid \\gset v{index}_",
            f"INSERT INTO definitions (langid,valsiid,definitionnum,definition,userid,time,metadata) VALUES (2,:v{index}_valsiid,1,convert_from(decode('{content}','base64'),'UTF8'),{args.user_id},extract(epoch from now())::int,'{{\"source\":\"CLL\",\"import\":\"cll-markdown\",\"cll_import_sha256\":\"{digest}\"}}'::jsonb) ON CONFLICT (langid,valsiid,definitionnum) DO UPDATE SET definition=EXCLUDED.definition,userid=EXCLUDED.userid,time=EXCLUDED.time,metadata=EXCLUDED.metadata RETURNING definitionid \\gset d{index}_",
            f"INSERT INTO definition_versions (definition_id,langid,valsiid,definition,user_id,message,word) SELECT :d{index}_definitionid,2,:v{index}_valsiid,convert_from(decode('{content}','base64'),'UTF8'),{args.user_id},'Imported CLL content','{title}' WHERE NOT EXISTS (SELECT 1 FROM definition_versions WHERE definition_id=:d{index}_definitionid AND definition=convert_from(decode('{content}','base64'),'UTF8'));",
        ]
    sql.append('COMMIT;')
    script = '\n'.join(sql) + '\n'
    if args.dry_run:
        print(f'-- {len(pages)} pages: TOC, 21 chapter pages, {len(pages)-22} sections', file=sys.stderr)
        sys.stdout.write(script)
        return 0
    command = shlex.split(args.psql_command) if args.psql_command else ['psql', args.dsn]
    command += ['-X', '-v', 'ON_ERROR_STOP=1']
    return subprocess.run(command, input=script, text=True).returncode
if __name__ == '__main__':
    raise SystemExit(main())
