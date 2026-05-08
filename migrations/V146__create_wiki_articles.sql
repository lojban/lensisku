-- Mirror of mw.lojban.org articles for the /waves "wiki" source.
-- Populated/refreshed by the background importer in src/wiki/importer.rs.

CREATE TABLE wiki_articles (
    id          SERIAL PRIMARY KEY,
    page_id     INTEGER NOT NULL UNIQUE,
    namespace   INTEGER NOT NULL,
    title       TEXT    NOT NULL,
    revision_id BIGINT,
    wikitext    TEXT    NOT NULL,
    markdown    TEXT    NOT NULL,
    plain_text  TEXT    NOT NULL,
    is_redirect BOOLEAN NOT NULL DEFAULT FALSE,
    last_edited TIMESTAMPTZ,
    fetched_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (namespace, title)
);

CREATE INDEX wiki_articles_title_trgm
    ON wiki_articles USING gin (title gin_trgm_ops);
CREATE INDEX wiki_articles_plain_trgm
    ON wiki_articles USING gin (plain_text gin_trgm_ops);
CREATE INDEX wiki_articles_last_edited_idx
    ON wiki_articles (last_edited DESC NULLS LAST);

CREATE TABLE wiki_sync_state (
    id                     INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    last_full_sync         TIMESTAMPTZ,
    last_incremental_sync  TIMESTAMPTZ
);
INSERT INTO wiki_sync_state (id) VALUES (1) ON CONFLICT DO NOTHING;
