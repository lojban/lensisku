-- Track MediaWiki revision ids on definition_versions so mw.lojban.org
-- history can be imported without duplicating rows on each sync.
ALTER TABLE definition_versions
    ADD COLUMN IF NOT EXISTS mw_revid BIGINT;

CREATE UNIQUE INDEX IF NOT EXISTS definition_versions_mw_revid_uidx
    ON definition_versions (mw_revid)
    WHERE mw_revid IS NOT NULL;

ALTER TABLE wiki_articles
    ADD COLUMN IF NOT EXISTS history_imported_until BIGINT;
