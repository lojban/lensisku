-- Relevance is a partial-order contract, not just an example of a plausible top hit.
-- Strong vector matches deliberately favor weaker lexical candidates below.
DO $$
DECLARE result record; v vector; ids integer[];
BEGIN
    v := ('[1,' || repeat('0,',382) || '0]')::vector;
    INSERT INTO messages VALUES
        (200,'Cat','Exact title',NULL,now()),
        (201,'An unrelated heading','A CAT, asleep beside the window.',NULL,now()),
        (202,'catalog','A catalogue of things.',NULL,now()),
        (203,'An animal','A feline companion',NULL,now());
    UPDATE wave_search_documents SET embedding=v WHERE kind='mail' AND source_id IN (202,203);
    INSERT INTO wave_search_chunks SELECT document_id,0,body,v FROM wave_search_documents WHERE kind='mail' AND source_id IN (202,203);
    SELECT * INTO result FROM test_search('cat','mail',NULL,v);
    SELECT array_agg((hit->>'source_id')::integer ORDER BY position) INTO ids
        FROM jsonb_array_elements(result.hits) WITH ORDINALITY AS h(hit,position);
    IF ids IS DISTINCT FROM ARRAY[200,201,202,203] THEN
        RAISE EXCEPTION 'Exact title > whole body word > title substring > semantic-only: %', result;
    END IF;
    IF (result.hits->2->>'score')::double precision <= (result.hits->1->>'score')::double precision THEN
        RAISE EXCEPTION 'Adversarial fixture must give substring a stronger fusion score';
    END IF;
    INSERT INTO wiki_articles VALUES
        (300,'Catalog entry','A cat lives here.',false,now()),
        (301,'CAT','Exact wiki heading',false,now());
    SELECT * INTO result FROM test_search('cat','all',NULL,v);
    SELECT array_agg((hit->>'source_id')::integer ORDER BY position) INTO ids
        FROM jsonb_array_elements(result.hits) WITH ORDINALITY AS h(hit,position);
    IF array_position(ids,201) >= array_position(ids,202)
       OR array_position(ids,300) >= array_position(ids,202)
       OR array_position(ids,301) >= array_position(ids,300)
       OR array_position(ids,202) >= array_position(ids,203) THEN
        RAISE EXCEPTION 'Precision ordering must apply globally across mail and wiki: %', result;
    END IF;
    DELETE FROM wiki_articles WHERE page_id IN (300,301);
    -- No embeddings and a deep page must retain the same relevance ordering.
    SELECT * INTO result FROM test_search('cat','mail',NULL,NULL,0.4,1,1);
    IF result.total <> 3 OR (result.hits->0->>'source_id')::integer <> 201 THEN
        RAISE EXCEPTION 'Lexical fallback and global pagination must preserve precision tiers';
    END IF;
    INSERT INTO messages VALUES (204,'Other heading','A ''cat'' is an animal.',NULL,now());
    SELECT * INTO result FROM test_search('cat','mail');
    IF NOT EXISTS (SELECT 1 FROM jsonb_array_elements(result.hits) hit
        WHERE (hit->>'source_id')::integer=204 AND (hit->>'match_priority')::integer=5) THEN
        RAISE EXCEPTION 'Surrounding quotation marks must not turn whole words into substrings';
    END IF;
    DELETE FROM messages WHERE id BETWEEN 200 AND 204;

    INSERT INTO messages VALUES
        (210,'Unrelated heading','scope changes quantifier interpretation',NULL,now()),
        (211,'Unrelated heading','quantifier     scope is subtle',NULL,now()),
        (212,'quantifier scope','Exact heading',NULL,now()),
        (213,'quantifier scopework','Only a substring phrase',NULL,now());
    UPDATE wave_search_documents SET embedding=v WHERE kind='mail' AND source_id=213;
    INSERT INTO wave_search_chunks SELECT document_id,0,body,v FROM wave_search_documents WHERE kind='mail' AND source_id=213;
    SELECT * INTO result FROM test_search('quantifier scope','mail',NULL,v);
    SELECT array_agg((hit->>'source_id')::integer ORDER BY position) INTO ids
        FROM jsonb_array_elements(result.hits) WITH ORDINALITY AS h(hit,position);
    IF ids IS DISTINCT FROM ARRAY[212,211,210,213] THEN
        RAISE EXCEPTION 'Exact title > whole phrase with flexible whitespace > all words reordered > substring: %', result;
    END IF;
    DELETE FROM messages WHERE id BETWEEN 210 AND 213;

    -- Apostrophes form part of a Lojban word. Query na must not give na'e the
    -- whole-word tier, while na'e itself must beat na'eperhaps.
    INSERT INTO messages VALUES
        (220,'Other heading', 'na means negation',NULL,now()),
        (221,'Other heading', 'na''e means scalar negation',NULL,now()),
        (222,'Other heading', 'na''eperhaps',NULL,now());
    SELECT * INTO result FROM test_search('na','mail');
    IF (result.hits->0->>'source_id')::integer <> 220
       OR (result.hits->1->>'match_priority')::integer >= 4 THEN
        RAISE EXCEPTION 'Lojban apostrophe must not create a false word boundary';
    END IF;
    SELECT * INTO result FROM test_search('na''e','mail');
    IF (result.hits->0->>'source_id')::integer <> 221 THEN
        RAISE EXCEPTION 'Whole Lojban token must beat its substring extension';
    END IF;
    DELETE FROM messages WHERE id BETWEEN 220 AND 222;

    -- Regex syntax in user text is always literal, never an executable pattern.
    INSERT INTO messages VALUES
        (230,'Other heading','Use c++ safely; reference broda[1] and a.b and a|b.',NULL,now()),
        (231,'Other heading','Use cxxxxxxxx safely; reference broda1 and axb.',NULL,now()),
        (232,'Other heading','日本語について話す',NULL,now()),
        (233,'Other heading','日本語 について話す',NULL,now()),
        (234,'Other heading','Literal \ycat appears here.',NULL,now());
    SELECT * INTO result FROM test_search('c++','mail');
    IF result.total <> 1 OR (result.hits->0->>'source_id')::integer <> 230 THEN
        RAISE EXCEPTION 'Regex quantifiers in queries must be literal';
    END IF;
    SELECT * INTO result FROM test_search('broda[1]','mail');
    IF (result.hits->0->>'source_id')::integer <> 230 OR (result.hits->0->>'match_priority')::integer <> 5 THEN
        RAISE EXCEPTION 'Regex brackets must be escaped for whole-word classification';
    END IF;
    SELECT * INTO result FROM test_search('a.b','mail');
    IF (result.hits->0->>'source_id')::integer <> 230 OR (result.hits->0->>'match_priority')::integer <> 5 THEN
        RAISE EXCEPTION 'Regex dot must be literal';
    END IF;
    SELECT * INTO result FROM test_search('a|b','mail');
    IF (result.hits->0->>'source_id')::integer <> 230 OR (result.hits->0->>'match_priority')::integer <> 5 THEN
        RAISE EXCEPTION 'Regex alternation must be literal';
    END IF;
    SELECT * INTO result FROM test_search('日本語','mail');
    IF (result.hits->0->>'source_id')::integer <> 233 THEN
        RAISE EXCEPTION 'Unicode whole words must outrank substring matches';
    END IF;
    SELECT * INTO result FROM test_search('\ycat','mail');
    IF result.total <> 1 OR (result.hits->0->>'source_id')::integer <> 234
       OR (result.hits->0->>'match_priority')::integer <> 5 THEN
        RAISE EXCEPTION 'Backslashes must remain literal instead of injecting word-boundary syntax';
    END IF;
    DELETE FROM messages WHERE id BETWEEN 230 AND 234;
END $$;
