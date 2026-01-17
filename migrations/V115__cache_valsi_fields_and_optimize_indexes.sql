-- Cache valsi fields in definitions table to eliminate valsi JOIN
-- This further optimizes fast search by removing the last remaining JOIN

-- Add cached valsi fields
ALTER TABLE definitions ADD COLUMN IF NOT EXISTS cached_valsiword TEXT;
ALTER TABLE definitions ADD COLUMN IF NOT EXISTS cached_rafsi TEXT;
ALTER TABLE definitions ADD COLUMN IF NOT EXISTS cached_source_langid INTEGER;
ALTER TABLE definitions ADD COLUMN IF NOT EXISTS cached_typeid SMALLINT;

-- Populate cached valsi fields
UPDATE definitions d
SET 
    cached_valsiword = v.word,
    cached_rafsi = v.rafsi,
    cached_source_langid = v.source_langid,
    cached_typeid = v.typeid
FROM valsi v
WHERE d.valsiid = v.valsiid;

-- Update sync function to include valsi fields
CREATE OR REPLACE FUNCTION sync_definition_cache_fields()
RETURNS TRIGGER AS $$
BEGIN
    -- Update cached fields for the affected definition(s)
    IF TG_TABLE_NAME = 'definitions' THEN
        UPDATE definitions d
        SET 
            cached_username = u.username,
            cached_langrealname = l.realname,
            cached_type_name = vt.descriptor,
            cached_valsiword = v.word,
            cached_rafsi = v.rafsi,
            cached_source_langid = v.source_langid,
            cached_typeid = v.typeid,
            cached_search_text = LOWER(
                COALESCE(v.word, '') || ' ' ||
                COALESCE(v.rafsi, '') || ' ' ||
                COALESCE(d.definition, '') || ' ' ||
                COALESCE(d.notes, '') || ' ' ||
                COALESCE(d.selmaho, '') || ' ' ||
                COALESCE((
                    SELECT string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' ')
                    FROM keywordmapping k
                    JOIN natlangwords n ON k.natlangwordid = n.wordid
                    WHERE k.definitionid = d.definitionid
                ), '')
            )
        FROM valsi v
        JOIN users u ON d.userid = u.userid
        JOIN languages l ON d.langid = l.langid
        JOIN valsitypes vt ON v.typeid = vt.typeid
        WHERE d.definitionid = COALESCE(NEW.definitionid, OLD.definitionid)
        AND d.valsiid = v.valsiid;
    ELSIF TG_TABLE_NAME = 'keywordmapping' THEN
        -- Update cached_search_text when keywords change
        UPDATE definitions d
        SET cached_search_text = LOWER(
            COALESCE(d.cached_valsiword, '') || ' ' ||
            COALESCE(d.cached_rafsi, '') || ' ' ||
            COALESCE(d.definition, '') || ' ' ||
            COALESCE(d.notes, '') || ' ' ||
            COALESCE(d.selmaho, '') || ' ' ||
            COALESCE((
                SELECT string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' ')
                FROM keywordmapping k
                JOIN natlangwords n ON k.natlangwordid = n.wordid
                WHERE k.definitionid = d.definitionid
            ), '')
        )
        WHERE d.definitionid = COALESCE(NEW.definitionid, OLD.definitionid);
    ELSIF TG_TABLE_NAME = 'valsi' THEN
        -- Update cached valsi fields and cached_search_text when valsi changes
        UPDATE definitions d
        SET 
            cached_valsiword = v.word,
            cached_rafsi = v.rafsi,
            cached_source_langid = v.source_langid,
            cached_typeid = v.typeid,
            cached_type_name = vt.descriptor,
            cached_search_text = LOWER(
                COALESCE(v.word, '') || ' ' ||
                COALESCE(v.rafsi, '') || ' ' ||
                COALESCE(d.definition, '') || ' ' ||
                COALESCE(d.notes, '') || ' ' ||
                COALESCE(d.selmaho, '') || ' ' ||
                COALESCE((
                    SELECT string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' ')
                    FROM keywordmapping k
                    JOIN natlangwords n ON k.natlangwordid = n.wordid
                    WHERE k.definitionid = d.definitionid
                ), '')
            )
        FROM valsi v
        JOIN valsitypes vt ON v.typeid = vt.typeid
        WHERE d.valsiid = COALESCE(NEW.valsiid, OLD.valsiid)
        AND d.valsiid = v.valsiid;
    ELSIF TG_TABLE_NAME = 'users' THEN
        -- Update cached_username when username changes
        UPDATE definitions d
        SET cached_username = NEW.username
        WHERE d.userid = NEW.userid;
    ELSIF TG_TABLE_NAME = 'languages' THEN
        -- Update cached_langrealname when language name changes
        UPDATE definitions d
        SET cached_langrealname = NEW.realname
        WHERE d.langid = NEW.langid;
    ELSIF TG_TABLE_NAME = 'valsitypes' THEN
        -- Update cached_type_name and cached_typeid when type descriptor changes
        UPDATE definitions d
        SET 
            cached_type_name = NEW.descriptor,
            cached_typeid = NEW.typeid
        WHERE d.cached_typeid = NEW.typeid;
    END IF;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Optimized indexes for fast search query pattern
-- The query filters by: cached_search_text ILIKE, langid, cached_source_langid
-- And sorts by: rank (computed), then sort_column (valsiword/type_name/created_at/score)

-- Composite index optimized for the WHERE clause pattern
-- This index supports: cached_search_text search + langid filter + source_langid filter
-- Note: Excluding large TEXT columns (cached_search_text, definition, notes) from INCLUDE
-- to avoid exceeding btree index row size limits. These are already indexed separately or
-- can be fetched from the table when needed.
CREATE INDEX IF NOT EXISTS idx_definitions_fast_search_where 
ON definitions(cached_source_langid, langid)
INCLUDE (definitionid, cached_valsiword, cached_rafsi, cached_username, cached_langrealname, cached_type_name, selmaho, created_at)
WHERE definition != '' AND cached_search_text IS NOT NULL;

-- Index for word_type filtering (cached_typeid)
CREATE INDEX IF NOT EXISTS idx_definitions_cached_typeid 
ON definitions(cached_typeid)
WHERE cached_typeid IS NOT NULL;

-- Index for username filtering
CREATE INDEX IF NOT EXISTS idx_definitions_cached_username 
ON definitions(cached_username)
WHERE cached_username IS NOT NULL;

-- Index for selmaho filtering (if not already exists)
CREATE INDEX IF NOT EXISTS idx_definitions_selmaho_cached 
ON definitions(selmaho)
WHERE selmaho IS NOT NULL;

-- The GIN index on cached_search_text (from V114) handles the text search
-- The composite index above handles the filtering efficiently
-- Together they enable very fast queries with no JOINs
