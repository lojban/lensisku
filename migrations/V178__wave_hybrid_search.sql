-- Shared lexical index: immediately searchable, even while vectors are backfilling.
CREATE TABLE wave_search_documents (
    document_id BIGSERIAL PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('comment', 'mail', 'wiki', 'native_wiki')),
    source_id INTEGER NOT NULL,
    source TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    body TEXT NOT NULL DEFAULT '',
    author TEXT NOT NULL DEFAULT '',
    edited_at TIMESTAMPTZ,
    generation BIGINT NOT NULL DEFAULT 1,
    embedding VECTOR(384),
    search_simple TSVECTOR GENERATED ALWAYS AS (
        setweight(to_tsvector('simple', title), 'A') ||
        setweight(to_tsvector('simple', body), 'B') ||
        setweight(to_tsvector('simple', author), 'C')) STORED,
    search_english TSVECTOR GENERATED ALWAYS AS (
        setweight(to_tsvector('english', title), 'A') ||
        setweight(to_tsvector('english', body), 'B')) STORED,
    UNIQUE (kind, source_id)
);
CREATE INDEX wave_search_simple ON wave_search_documents USING gin (search_simple);
CREATE INDEX wave_search_english ON wave_search_documents USING gin (search_english);
CREATE INDEX wave_search_title ON wave_search_documents USING gin (title gin_trgm_ops);
CREATE INDEX wave_search_body ON wave_search_documents USING gin (body gin_trgm_ops);
CREATE INDEX wave_search_author ON wave_search_documents USING gin (author gin_trgm_ops);
CREATE INDEX wave_search_mirror_identity ON wiki_articles ((page_id::text));
CREATE INDEX wave_search_pending ON wave_search_documents (document_id) WHERE embedding IS NULL;
CREATE INDEX wave_search_embedding ON wave_search_documents USING hnsw (embedding vector_cosine_ops)
    WHERE embedding IS NOT NULL;

CREATE TABLE wave_search_chunks (
    document_id BIGINT NOT NULL REFERENCES wave_search_documents ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    embedding VECTOR(384) NOT NULL,
    PRIMARY KEY (document_id, chunk_index)
);
CREATE INDEX wave_chunk_embedding ON wave_search_chunks USING hnsw (embedding vector_cosine_ops);

-- Repeated imports and non-text edits retain vectors. Text changes atomically invalidate
-- document and paragraph embeddings. generation guards against edits during inference.
CREATE FUNCTION wave_search_upsert(k TEXT, sid INTEGER, src TEXT, heading TEXT,
                                    text_body TEXT, writer TEXT, edited TIMESTAMPTZ)
RETURNS VOID LANGUAGE plpgsql AS $$
DECLARE changed_id BIGINT;
BEGIN
    INSERT INTO wave_search_documents (kind, source_id, source, title, body, author, edited_at)
    VALUES (k, sid, src, coalesce(heading,''), coalesce(text_body,''), coalesce(writer,''), edited)
    ON CONFLICT (kind, source_id) DO UPDATE SET
        source = EXCLUDED.source, title = EXCLUDED.title, body = EXCLUDED.body,
        author = EXCLUDED.author, edited_at = EXCLUDED.edited_at,
        generation = wave_search_documents.generation +
            CASE WHEN (wave_search_documents.title, wave_search_documents.body)
                IS DISTINCT FROM (EXCLUDED.title, EXCLUDED.body) THEN 1 ELSE 0 END,
        embedding = CASE WHEN (wave_search_documents.title, wave_search_documents.body)
                IS DISTINCT FROM (EXCLUDED.title, EXCLUDED.body) THEN NULL
                ELSE wave_search_documents.embedding END
    RETURNING CASE WHEN embedding IS NULL THEN document_id END INTO changed_id;
    IF changed_id IS NOT NULL THEN
        DELETE FROM wave_search_chunks WHERE document_id = changed_id;
    END IF;
END $$;

CREATE FUNCTION wave_search_comment_sync() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        DELETE FROM wave_search_documents WHERE kind = 'comment' AND source_id = OLD.commentid;
    ELSE
        PERFORM wave_search_upsert('comment', NEW.commentid,
            CASE WHEN NEW.import_source IN ('jbotcan','freeforums') THEN NEW.import_source ELSE 'comments' END,
            NEW.subject, NEW.plain_content, (SELECT username FROM users WHERE userid = NEW.userid),
            to_timestamp(NEW.time));
    END IF;
    RETURN NULL;
END $$;
CREATE TRIGGER wave_search_comment_sync AFTER INSERT OR UPDATE OR DELETE ON comments
    FOR EACH ROW EXECUTE FUNCTION wave_search_comment_sync();

CREATE FUNCTION wave_search_mail_sync() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        DELETE FROM wave_search_documents WHERE kind = 'mail' AND source_id = OLD.id;
    ELSE
        PERFORM wave_search_upsert('mail', NEW.id, 'mail', NEW.subject,
                                   NEW.content, NEW.from_address, NEW.sent_at);
    END IF;
    RETURN NULL;
END $$;
CREATE TRIGGER wave_search_mail_sync AFTER INSERT OR UPDATE OR DELETE ON messages
    FOR EACH ROW EXECUTE FUNCTION wave_search_mail_sync();

CREATE FUNCTION wave_search_wiki_sync() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        DELETE FROM wave_search_documents WHERE kind = 'wiki' AND source_id = OLD.page_id;
    ELSIF NEW.is_redirect THEN
        DELETE FROM wave_search_documents WHERE kind = 'wiki' AND source_id = NEW.page_id;
    ELSE
        PERFORM wave_search_upsert('wiki', NEW.page_id, 'wiki', NEW.title, NEW.plain_text, '', NEW.last_edited);
    END IF;
    RETURN NULL;
END $$;
CREATE TRIGGER wave_search_wiki_sync AFTER INSERT OR UPDATE OR DELETE ON wiki_articles
    FOR EACH ROW EXECUTE FUNCTION wave_search_wiki_sync();

CREATE FUNCTION wave_search_native_sync(def_id INTEGER) RETURNS VOID LANGUAGE plpgsql AS $$
BEGIN
    PERFORM wave_search_upsert('native_wiki', d.definitionid, 'wiki', v.word, d.definition, '', d.created_at)
    FROM definitions d JOIN valsi v ON v.valsiid = d.valsiid
    WHERE d.definitionid = def_id AND v.typeid = 16
      AND coalesce(d.metadata->>'is_redirect','false') <> 'true';
    IF NOT FOUND THEN
        DELETE FROM wave_search_documents WHERE kind = 'native_wiki' AND source_id = def_id;
    END IF;
END $$;
CREATE FUNCTION wave_search_definition_sync() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        DELETE FROM wave_search_documents WHERE kind = 'native_wiki' AND source_id = OLD.definitionid;
    ELSE
        PERFORM wave_search_native_sync(NEW.definitionid);
    END IF;
    RETURN NULL;
END $$;
CREATE TRIGGER wave_search_definition_sync AFTER INSERT OR UPDATE OF definition, valsiid, metadata OR DELETE ON definitions
    FOR EACH ROW EXECUTE FUNCTION wave_search_definition_sync();
CREATE FUNCTION wave_search_valsi_sync() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE def_id INTEGER;
BEGIN
    FOR def_id IN SELECT definitionid FROM definitions WHERE valsiid = NEW.valsiid LOOP
        PERFORM wave_search_native_sync(def_id);
    END LOOP;
    RETURN NULL;
END $$;
CREATE TRIGGER wave_search_valsi_sync AFTER UPDATE OF word, typeid ON valsi
    FOR EACH ROW EXECUTE FUNCTION wave_search_valsi_sync();
CREATE FUNCTION wave_search_author_sync() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    UPDATE wave_search_documents SET author = coalesce(NEW.username,'')
    WHERE kind = 'comment' AND source_id IN (SELECT commentid FROM comments WHERE userid = NEW.userid);
    RETURN NULL;
END $$;
CREATE TRIGGER wave_search_author_sync AFTER UPDATE OF username ON users
    FOR EACH ROW EXECUTE FUNCTION wave_search_author_sync();

-- Bulk backfill avoids issuing one trigger query per historical document.
INSERT INTO wave_search_documents (kind, source_id, source, title, body, author, edited_at)
SELECT 'comment', c.commentid,
       CASE WHEN c.import_source IN ('jbotcan','freeforums') THEN c.import_source ELSE 'comments' END,
       coalesce(c.subject,''), coalesce(c.plain_content,''), coalesce(u.username,''), to_timestamp(c.time)
FROM comments c JOIN users u ON u.userid = c.userid
UNION ALL
SELECT 'mail', id, 'mail', coalesce(subject,''), coalesce(content,''), coalesce(from_address,''), sent_at FROM messages
UNION ALL
SELECT 'wiki', page_id, 'wiki', title, plain_text, '', last_edited FROM wiki_articles WHERE NOT is_redirect
UNION ALL
SELECT 'native_wiki', d.definitionid, 'wiki', v.word, d.definition, '', d.created_at
FROM definitions d JOIN valsi v ON v.valsiid = d.valsiid
WHERE v.typeid = 16 AND coalesce(d.metadata->>'is_redirect','false') <> 'true';
