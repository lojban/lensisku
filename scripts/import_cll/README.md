# Import the CLL into native wiki

Imports the complete 21-chapter Markdown corpus as a separate table of contents page, 21 chapter landing pages, and 319 native wiki section pages named **Chapter N. Section M**. Each section page has previous/next and contents links. Existing section/example anchor IDs stay with their section; corpus cross-links are mapped to the page that owns the target anchor. Re-importing renames prior `CLL N` and `CLL N.M` imported pages in place, without leaving redirect pages.

Requires Python 3. To import into a remote instance through its API:

```sh
python3 scripts/import_cll/import_cll.py \
  --api-url https://lensisku-dev.lojban.org/api --username gleki
```

The script prompts for the password without echoing it. Automation can supply `CLL_USERNAME` and `CLL_PASSWORD`, or `CLL_ACCESS_TOKEN`; credentials are never written to files. It uses authenticated native wiki upserts, includes CLL metadata and version messages, and reads saved pages back to verify their titles and anchors. Existing pages without CLL metadata are rejected to avoid overwriting unrelated content. Re-running skips unchanged pages and resumes after a failure. API mode uses the server's normal permissions, validation, and notifications. `--dry-run` with `--api-url` lists planned titles without connecting.

For direct database imports, `psql` and a Lensisku database with current migrations are also required. For local Compose:

```sh
DATABASE_URL='postgres://lojban:password@localhost:5432/lojban_lens' python3 scripts/import_cll/import_cll.py
```

If `psql` is only available inside the local database container, pass the full command instead:

```sh
python3 scripts/import_cll/import_cll.py \
  --psql-command 'docker exec -i lenpostgres psql -U lojban -d lojban_lens'
```

`--dry-run` prints SQL, `--source PATH` selects another corpus checkout, and `--user-id ID` (or `CLL_IMPORT_USER_ID`) selects the importing Lensisku user. The default importer is seeded `officialdata` (ID 1). Re-running updates the imported pages in place.
