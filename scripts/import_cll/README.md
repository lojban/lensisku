# Import the CLL into native wiki

Imports the complete 21-chapter Markdown corpus as a separate table of contents page, 21 chapter landing pages, and 319 native wiki section pages named **Chapter N. Section M**. Each section page has previous/next and contents links. Existing section/example anchor IDs stay with their section; corpus cross-links are mapped to the page that owns the target anchor. Re-importing renames prior `CLL N` and `CLL N.M` imported pages in place, without leaving redirect pages.

Requires Python 3, `psql`, and a Lensisku database with current migrations applied. For local Compose:

```sh
DATABASE_URL='postgres://lojban:password@localhost:5432/lojban_lens' python3 scripts/import_cll/import_cll.py
```

If `psql` is only available inside the local database container, pass the full command instead:

```sh
python3 scripts/import_cll/import_cll.py \
  --psql-command 'docker exec -i lenpostgres psql -U lojban -d lojban_lens'
```

`--dry-run` prints SQL, `--source PATH` selects another corpus checkout, and `--user-id ID` (or `CLL_IMPORT_USER_ID`) selects the importing Lensisku user. The default importer is seeded `officialdata` (ID 1). Re-running updates the imported pages in place.
