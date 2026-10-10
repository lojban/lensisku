#!/usr/bin/env python3
"""Import the CLL as navigable native wiki pages (TOC, chapters, and sections)."""
from __future__ import annotations
import argparse, base64, os, re, shlex, subprocess, sys
from dataclasses import dataclass
from pathlib import Path

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

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path('/home/user/lojban/korpora2/cll'))
    parser.add_argument('--dsn', default=os.environ.get('DATABASE_URL'))
    parser.add_argument('--psql-command', default=os.environ.get('CLL_PSQL_COMMAND'), help='full psql command, for example: docker exec -i lenpostgres psql -U lojban -d lojban_lens')
    parser.add_argument('--user-id', type=int, default=int(os.environ.get('CLL_IMPORT_USER_ID', '1')))
    parser.add_argument('--dry-run', action='store_true', help='print SQL without connecting')
    args = parser.parse_args()
    pages = make_pages(args.source)
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
        title = page.title.replace("'", "''")
        sql += [
            f"INSERT INTO valsi (word,typeid,userid,time,source_langid) VALUES ('{title}',16,{args.user_id},extract(epoch from now())::int,1) ON CONFLICT (word,source_langid) WHERE typeid=16 DO UPDATE SET word=EXCLUDED.word RETURNING valsiid \\gset v{index}_",
            f"INSERT INTO definitions (langid,valsiid,definitionnum,definition,userid,time,metadata) VALUES (2,:v{index}_valsiid,1,convert_from(decode('{content}','base64'),'UTF8'),{args.user_id},extract(epoch from now())::int,'{{\"source\":\"CLL\",\"import\":\"cll-markdown\"}}'::jsonb) ON CONFLICT (langid,valsiid,definitionnum) DO UPDATE SET definition=EXCLUDED.definition,userid=EXCLUDED.userid,time=EXCLUDED.time,metadata=EXCLUDED.metadata RETURNING definitionid \\gset d{index}_",
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
