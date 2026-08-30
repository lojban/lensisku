-- MediaWiki revision ids are not chronological on a page (a later edit can
-- have a smaller revid). The importer used max(revid) as a watermark and
-- skipped revid <= that value, so some older-timestamp revisions were never
-- stored. Reset so every mirrored page is walked oldest-first again.
-- Existing definition_versions rows are kept (ON CONFLICT mw_revid DO NOTHING).
--
-- QA: open a wiki article with several mw.lojban.org edits (e.g. fi'i) and
-- confirm History lists every revision, not only the latest. After deploy,
-- background import backfills until each page is caught up.

UPDATE wiki_articles
SET history_imported_until = NULL
WHERE history_imported_until IS NOT NULL;
