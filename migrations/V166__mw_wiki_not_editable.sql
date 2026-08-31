-- Imported mw.lojban.org wiki pages are read-only mirrors in Lensisku.
-- Same pattern as official dictionary data: owner_only + officialdata author.
-- QA: open an imported article (e.g. fi'i) — no Edit button; direct API edit/rename
-- returns permission error. Native wiki pages at the same title remain editable.
-- After deploy, the background importer re-tags rows on the next sync pass.

UPDATE definitions
SET metadata = COALESCE(metadata, '{}'::jsonb) || '{"imported": true}'::jsonb,
    owner_only = true
WHERE COALESCE(metadata->>'source', '') = 'mw.lojban.org'
   OR metadata ? 'mw_page_id';
