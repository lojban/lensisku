-- Add indexes for fast search functionality
-- These indexes optimize searches in glosswords (keywordmapping/natlangwords)

-- Index for keywordmapping to speed up glosswords search
-- This allows fast lookups when searching by definitionid and place
CREATE INDEX IF NOT EXISTS idx_keywordmapping_definitionid_place 
ON keywordmapping(definitionid, place);

-- Index for natlangwords to speed up text search in glosswords
-- Using GIN index with pg_trgm for fast ILIKE searches on word and meaning
CREATE INDEX IF NOT EXISTS idx_natlangwords_word_gin 
ON natlangwords USING gin (word gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_natlangwords_meaning_gin 
ON natlangwords USING gin (meaning gin_trgm_ops) WHERE meaning IS NOT NULL;

-- Composite index for keywordmapping joins with natlangwords
-- This optimizes the JOIN between keywordmapping and natlangwords
CREATE INDEX IF NOT EXISTS idx_keywordmapping_natlangwordid 
ON keywordmapping(natlangwordid);

-- Index for natlangwords wordid (primary key, but ensure it exists for JOINs)
CREATE INDEX IF NOT EXISTS idx_natlangwords_wordid 
ON natlangwords(wordid);
