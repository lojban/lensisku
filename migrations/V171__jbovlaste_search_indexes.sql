-- Indexes for jbovlaste lexical / fast / semantic search paths that use
-- cached_canonical_word and denormalized cached_valsiword equality + ILIKE.
--
-- QA: lexical + fast search and semantic exact-match anchors for non-canonical
-- lujvo should stay correct; no UI change. BitmapOr on the search WHERE can use
-- these indexes instead of sequential scan for exact/ILIKE arms.

-- Exact / ANY() match on canonical spelling aliases (sparse: mostly lujvo)
CREATE INDEX IF NOT EXISTS idx_definitions_cached_canonical_word
ON definitions (cached_canonical_word)
WHERE cached_canonical_word IS NOT NULL;

-- Exact / ANY() match on denormalized headword (alias reverse lookups)
CREATE INDEX IF NOT EXISTS idx_definitions_cached_valsiword
ON definitions (cached_valsiword)
WHERE cached_valsiword IS NOT NULL;

-- ILIKE / substring on cached_valsiword (lojban h/' variant arm of WHERE)
CREATE INDEX IF NOT EXISTS idx_definitions_cached_valsiword_gin
ON definitions USING gin (cached_valsiword gin_trgm_ops)
WHERE cached_valsiword IS NOT NULL;

-- Refresh covering index to INCLUDE cached_canonical_word so exact-canonical
-- hits can avoid heap fetches when this index is chosen for (source_langid, langid).
DROP INDEX IF EXISTS idx_definitions_fast_search_where;
CREATE INDEX IF NOT EXISTS idx_definitions_fast_search_where
ON definitions (cached_source_langid, langid)
INCLUDE (
    definitionid,
    cached_valsiword,
    cached_rafsi,
    cached_canonical_word,
    cached_username,
    cached_langrealname,
    cached_type_name,
    selmaho,
    created_at
)
WHERE definition != '' AND cached_search_text IS NOT NULL;
