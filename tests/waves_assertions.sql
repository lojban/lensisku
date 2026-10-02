DO $$
DECLARE result record; gen bigint; v vector;
BEGIN
    IF (SELECT count(*) FROM wave_search_documents) <> 6 THEN
        RAISE EXCEPTION 'Backfill must cover all four document kinds and exclude redirects/dictionary definitions';
    END IF;
    SELECT * INTO result FROM test_search('logical scope');
    IF result.total <> 2 OR result.hits->0->>'title' <> 'logical scope' THEN
        RAISE EXCEPTION 'Global exact-title/FTS ranking failed: %', result;
    END IF;
    UPDATE definitions SET metadata = '{"mw_page_id":1}' WHERE definitionid = 1;
    SELECT * INTO result FROM test_search('scope', 'wiki');
    IF result.total <> 1 OR result.hits->0->>'kind' <> 'wiki' THEN
        RAISE EXCEPTION 'Mirror and native history for the same imported wiki must be one result';
    END IF;
    UPDATE wiki_articles SET is_redirect = true WHERE page_id = 1;
    SELECT * INTO result FROM test_search('scope', 'wiki');
    IF result.total <> 0 THEN
        RAISE EXCEPTION 'Redirect mirrors must suppress their native history stubs too';
    END IF;
    DELETE FROM wiki_articles WHERE page_id = 1;
    SELECT * INTO result FROM test_search('scope', 'wiki');
    IF result.total <> 1 OR result.hits->0->>'kind' <> 'native_wiki' THEN
        RAISE EXCEPTION 'Native representation must remain discoverable when mirror is deleted';
    END IF;
    INSERT INTO wiki_articles VALUES (1,'logical scope','Scope and quantification.',false,now());
    UPDATE definitions SET metadata = '{}' WHERE definitionid = 1;
    SELECT * INTO result FROM test_search('discussions');
    IF result.total <> 1 THEN RAISE EXCEPTION 'English stemming must retrieve discussion/discussions: %', result; END IF;
    UPDATE comments SET plain_content = repeat('Unrelated garden text. ',100) || 'A discussion about logical scope.' WHERE commentid = 1;
    SELECT * INTO result FROM test_search('discussions');
    IF result.hits->0->>'excerpt' NOT LIKE '%discussion%' OR result.hits->0->>'excerpt' LIKE '%<b>%' THEN
        RAISE EXCEPTION 'Stemmed lexical matches must return the matching late passage as plain text';
    END IF;
    SELECT * INTO result FROM test_search('%_');
    IF result.total <> 1 OR (result.hits->0->>'source_id')::integer <> 3 THEN
        RAISE EXCEPTION 'Percent and underscore must be literal substrings: %', result;
    END IF;
    SELECT * INTO result FROM test_search('scope', 'mail');
    -- Source-restricted lexical search must not leak matching comments/wikis.
    IF result.total <> 1 OR result.hits->0->>'kind' <> 'mail' THEN RAISE EXCEPTION 'Source restriction failed: %', result; END IF;
    SELECT * INTO result FROM test_search('scope', 'all', 10);
    IF result.total <> 1 OR result.hits->0->>'kind' <> 'comment' THEN
        RAISE EXCEPTION 'Collection restriction must suppress mail and both wikis: %', result;
    END IF;
    SELECT * INTO result FROM test_search('scope', 'wiki', NULL, NULL, 0.4, 1, 100);
    IF result.total <> 2 OR jsonb_array_length(result.hits) <> 0 THEN
        RAISE EXCEPTION 'Deep page must retain total and have no phantom results: %', result;
    END IF;
    SELECT * INTO result FROM test_search('scope', 'all', NULL, NULL, 0.4, 1, 0);
    IF jsonb_array_length(result.hits) <> 1 THEN RAISE EXCEPTION 'Global pagination failed'; END IF;

    -- Deterministic orthogonal vectors exercise semantic-only matches without loading a model.
    v := ('[1,' || repeat('0,',382) || '0]')::vector;
    UPDATE wave_search_documents SET embedding = v WHERE kind = 'mail';
    INSERT INTO wave_search_chunks SELECT document_id, 0, 'A relevant late paragraph.', v
        FROM wave_search_documents WHERE kind = 'mail';
    INSERT INTO wave_search_chunks SELECT document_id, 1, 'Another relevant paragraph.', v
        FROM wave_search_documents WHERE kind = 'mail';
    SELECT * INTO result FROM test_search('meaning absent from text', 'all', NULL, v);
    IF result.total <> 1 OR jsonb_array_length(result.hits) <> 1
       OR result.hits->0->>'kind' <> 'mail'
       OR NOT (result.hits->0->>'semantic_match')::boolean
       OR (result.hits->0->>'lexical_match')::boolean
       OR result.hits->0->>'excerpt' <> 'A relevant late paragraph.' THEN
        RAISE EXCEPTION 'Semantic-only retrieval/paragraph deduplication failed: %', result;
    END IF;
    SELECT * INTO result FROM test_search('meaning absent from text', 'wiki', NULL, v);
    IF result.total <> 0 THEN RAISE EXCEPTION 'Semantic source filter failed'; END IF;
    SELECT * INTO result FROM test_search('meaning absent from text', 'all', 10, v);
    IF result.total <> 0 THEN RAISE EXCEPTION 'Semantic collection filter failed'; END IF;
    SELECT * INTO result FROM test_search('meaning absent from text', 'all', NULL, ('[-1,' || repeat('0,',382) || '0]')::vector);
    IF result.total <> 0 THEN RAISE EXCEPTION 'Semantic threshold failed'; END IF;

    SELECT generation INTO gen FROM wave_search_documents WHERE kind = 'mail';
    UPDATE messages SET sent_at = now() + interval '1 day' WHERE id = 1;
    IF (SELECT embedding IS NULL OR generation <> gen FROM wave_search_documents WHERE kind = 'mail') THEN
        RAISE EXCEPTION 'Metadata-only updates must retain vectors';
    END IF;
    UPDATE messages SET content = 'Changed message body' WHERE id = 1;
    IF (SELECT embedding IS NOT NULL OR generation <> gen + 1 FROM wave_search_documents WHERE kind = 'mail')
       OR EXISTS (SELECT 1 FROM wave_search_chunks) THEN
        RAISE EXCEPTION 'Edits must invalidate document and all paragraphs atomically';
    END IF;
    -- This is the same optimistic publication guard used by the worker.
    UPDATE wave_search_documents SET embedding = v WHERE kind = 'mail' AND generation = gen;
    IF FOUND THEN RAISE EXCEPTION 'Stale inference must not publish after a concurrent edit'; END IF;
    DELETE FROM messages WHERE id = 1;
    IF EXISTS (SELECT 1 FROM wave_search_documents WHERE kind = 'mail') THEN RAISE EXCEPTION 'Delete invalidation failed'; END IF;

    UPDATE valsi SET word = 'Renamed native article' WHERE valsiid = 1;
    IF NOT EXISTS (SELECT 1 FROM wave_search_documents WHERE kind = 'native_wiki' AND title = 'Renamed native article') THEN
        RAISE EXCEPTION 'Native wiki rename indexing failed';
    END IF;
    UPDATE definitions SET metadata = '{"is_redirect": true}' WHERE definitionid = 1;
    IF EXISTS (SELECT 1 FROM wave_search_documents WHERE kind = 'native_wiki') THEN RAISE EXCEPTION 'Native redirects must leave index'; END IF;
    UPDATE definitions SET metadata = '{}' WHERE definitionid = 1;
    UPDATE valsi SET typeid = 1 WHERE valsiid = 1;
    IF EXISTS (SELECT 1 FROM wave_search_documents WHERE kind = 'native_wiki') THEN RAISE EXCEPTION 'Wiki type change must leave index'; END IF;
    UPDATE wiki_articles SET is_redirect = true WHERE page_id = 1;
    IF EXISTS (SELECT 1 FROM wave_search_documents WHERE kind = 'wiki') THEN RAISE EXCEPTION 'Mirrored redirect invalidation failed'; END IF;
    UPDATE comments SET import_source = 'freeforums', plain_content = 'Updated forum topic' WHERE commentid = 2;
    IF NOT EXISTS (SELECT 1 FROM wave_search_documents WHERE kind = 'comment' AND source_id = 2 AND source = 'freeforums' AND body = 'Updated forum topic') THEN
        RAISE EXCEPTION 'Forum import updates failed';
    END IF;
    UPDATE users SET username = 'renamed_alice' WHERE userid = 1;
    SELECT * INTO result FROM test_search('renamed_alice', 'comments');
    IF result.total <> 1 THEN RAISE EXCEPTION 'Username updates/search failed'; END IF;
    UPDATE threads SET collection_id = 20 WHERE threadid = 1;
    SELECT * INTO result FROM test_search('scope', 'all', 10);
    IF result.total <> 0 THEN RAISE EXCEPTION 'Collection filter must reflect live thread reattachment'; END IF;
    DELETE FROM comments WHERE commentid = 1;
    IF EXISTS (SELECT 1 FROM wave_search_documents WHERE kind = 'comment' AND source_id = 1) THEN RAISE EXCEPTION 'Comment deletion invalidation failed'; END IF;
END $$;
