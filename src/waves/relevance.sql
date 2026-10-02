-- $1 query, $2 source, $3 collection, $4 vector (nullable), $5 min similarity,
-- $6 page size, $7 offset. Lexical matches are never capped; semantic candidates
-- are bounded nearest-neighbour lists, not an assertion that every document matches.
WITH scoped AS NOT MATERIALIZED (
    SELECT d.* FROM wave_search_documents d
    WHERE /*CANONICAL*/ AND ($2 = 'all' OR d.source = $2)
      AND ($3::integer IS NULL OR (d.kind = 'comment' AND EXISTS (
          SELECT 1 FROM comments c JOIN threads t ON t.threadid = c.threadid
          WHERE c.commentid = d.source_id AND t.collection_id = $3)))
), q AS (
    SELECT websearch_to_tsquery('simple', $1) AS simple,
           websearch_to_tsquery('english', $1) AS english,
           '%' || replace(replace(replace($1, '\', '\\'), '%', '\%'), '_', '\_') || '%' AS pattern
), lexical AS (
    SELECT d.document_id,
        row_number() OVER (ORDER BY
            CASE WHEN lower(d.title) = lower($1) THEN 3
                 WHEN d.title ILIKE q.pattern ESCAPE '\' THEN 2
                 WHEN d.body ILIKE q.pattern ESCAPE '\' THEN 1 ELSE 0 END DESC,
            greatest(ts_rank_cd(d.search_simple, q.simple, 32),
                     ts_rank_cd(d.search_english, q.english, 32)) DESC,
            d.document_id ASC) AS rank
    FROM scoped d CROSS JOIN q
    WHERE d.search_simple @@ q.simple OR d.search_english @@ q.english
       OR d.title ILIKE q.pattern ESCAPE '\' OR d.body ILIKE q.pattern ESCAPE '\'
       OR d.author ILIKE q.pattern ESCAPE '\'
), document_nearest AS MATERIALIZED (
    SELECT d.document_id, d.embedding <=> $4::vector AS distance
    FROM wave_search_documents d
    WHERE $4::vector IS NOT NULL AND d.embedding IS NOT NULL
      AND EXISTS (SELECT 1 FROM scoped s WHERE s.document_id = d.document_id)
    ORDER BY d.embedding <=> $4::vector LIMIT 200
), document_ranked AS (
    SELECT document_id, row_number() OVER (ORDER BY distance, document_id) AS rank
    FROM document_nearest WHERE distance <= 1.0 - $5::double precision
), chunk_nearest AS MATERIALIZED (
    SELECT c.document_id, c.chunk_index, c.content, c.embedding <=> $4::vector AS distance
    FROM wave_search_chunks c
    WHERE $4::vector IS NOT NULL
      AND EXISTS (SELECT 1 FROM scoped s WHERE s.document_id = c.document_id)
    ORDER BY c.embedding <=> $4::vector LIMIT 2000
), best_chunk AS (
    -- Many matching paragraphs from one long mail count as one semantic vote.
    SELECT DISTINCT ON (document_id) document_id, content, distance
    FROM chunk_nearest WHERE distance <= 1.0 - $5::double precision
    ORDER BY document_id, distance, chunk_index
), chunk_ranked AS (
    SELECT document_id, row_number() OVER (ORDER BY distance, document_id) AS rank
    FROM best_chunk ORDER BY distance, document_id LIMIT 200
), votes AS (
    SELECT document_id, 2.0 / (60 + rank) AS score FROM lexical
    UNION ALL SELECT document_id, 0.7 / (60 + rank) FROM document_ranked
    UNION ALL SELECT document_id, 1.0 / (60 + rank) FROM chunk_ranked
), fused AS (
    SELECT document_id, sum(score)::double precision AS score FROM votes GROUP BY document_id
), matches AS (
    SELECT d.*, f.score, l.document_id IS NOT NULL AS lexical_match,
        (dr.document_id IS NOT NULL OR cr.document_id IS NOT NULL) AS semantic_match,
        b.content AS semantic_excerpt,
        coalesce(cc.total_reactions, 0) AS reactions, coalesce(cc.total_replies, 0) AS replies
    FROM fused f JOIN scoped d USING (document_id)
    LEFT JOIN lexical l USING (document_id)
    LEFT JOIN document_ranked dr USING (document_id)
    LEFT JOIN chunk_ranked cr USING (document_id)
    LEFT JOIN best_chunk b USING (document_id)
    LEFT JOIN comment_activity_counters cc ON d.kind = 'comment' AND cc.comment_id = d.source_id
)
-- Aggregate the page separately so empty/deep pages still return the correct total.
SELECT (SELECT count(*) FROM matches) AS total,
       coalesce((SELECT jsonb_agg(to_jsonb(page)) FROM (
           SELECT document_id, kind, source_id, source, title, edited_at, score,
                  lexical_match, semantic_match,
                  CASE WHEN strpos(lower(body), lower($1)) > 0
                       THEN substring(body FROM greatest(strpos(lower(body), lower($1)) - 120, 1) FOR 500)
                       WHEN to_tsvector('simple', body) @@ q.simple
                       THEN left(ts_headline('simple', body, q.simple,
                           'StartSel="",StopSel="",MaxWords=70,MinWords=20,MaxFragments=1'), 500)
                       WHEN to_tsvector('english', body) @@ q.english
                       THEN left(ts_headline('english', body, q.english,
                           'StartSel="",StopSel="",MaxWords=70,MinWords=20,MaxFragments=1'), 500)
                       ELSE coalesce(semantic_excerpt, substring(body FOR 500)) END AS excerpt
           FROM matches CROSS JOIN q
           ORDER BY /*ORDER*/ LIMIT $6 OFFSET $7
       ) page), '[]'::jsonb) AS hits
