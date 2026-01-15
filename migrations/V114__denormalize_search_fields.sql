-- Add denormalized cached fields for fast search performance
-- This eliminates the need for JOINs to users, languages, valsitypes, and keywordmapping/natlangwords

-- Add cached display fields
ALTER TABLE definitions ADD COLUMN IF NOT EXISTS cached_username TEXT;
ALTER TABLE definitions ADD COLUMN IF NOT EXISTS cached_langrealname TEXT;
ALTER TABLE definitions ADD COLUMN IF NOT EXISTS cached_type_name TEXT;

-- Add cached search text field containing all searchable content
ALTER TABLE definitions ADD COLUMN IF NOT EXISTS cached_search_text TEXT;

-- Populate cached_search_text with all searchable content
-- This includes: word, rafsi, definition, notes, selmaho, and all glosswords/place keywords
UPDATE definitions d
SET cached_search_text = LOWER(
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
WHERE d.valsiid = v.valsiid;

-- Populate cached display fields
UPDATE definitions d
SET 
    cached_username = u.username,
    cached_langrealname = l.realname,
    cached_type_name = vt.descriptor
FROM valsi v
JOIN users u ON d.userid = u.userid
JOIN languages l ON d.langid = l.langid
JOIN valsitypes vt ON v.typeid = vt.typeid
WHERE d.valsiid = v.valsiid;

-- Create GIN index for fast text search on cached_search_text
CREATE INDEX IF NOT EXISTS idx_definitions_cached_search_text_gin 
ON definitions USING gin(cached_search_text gin_trgm_ops);

-- Covering index for fast search main query
CREATE INDEX IF NOT EXISTS idx_definitions_fast_search_covering 
ON definitions(definitionid, valsiid, langid, definition, notes, selmaho, created_at, cached_username, cached_langrealname, cached_type_name)
WHERE definition != '';

-- Composite index for filtering (langid + valsiid)
CREATE INDEX IF NOT EXISTS idx_definitions_langid_valsiid 
ON definitions(langid, valsiid)
INCLUDE (definitionid, cached_search_text);

-- Index for valsi filtering (minimal join needed)
CREATE INDEX IF NOT EXISTS idx_valsi_valsiid_source_langid_word 
ON valsi(valsiid, source_langid, word)
INCLUDE (rafsi, typeid);

-- Function to sync all cached fields
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
        WHERE d.definitionid = COALESCE(NEW.definitionid, OLD.definitionid)
        AND d.valsiid = v.valsiid;
    ELSIF TG_TABLE_NAME = 'valsi' THEN
        -- Update cached_search_text when valsi word/rafsi changes
        UPDATE definitions d
        SET cached_search_text = LOWER(
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
        -- Update cached_type_name when type descriptor changes
        UPDATE definitions d
        SET cached_type_name = NEW.descriptor
        FROM valsi v
        WHERE d.valsiid = v.valsiid
        AND v.typeid = NEW.typeid;
    END IF;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Trigger on definitions changes
DROP TRIGGER IF EXISTS trigger_sync_definition_cache ON definitions;
CREATE TRIGGER trigger_sync_definition_cache
AFTER INSERT OR UPDATE OF definition, notes, selmaho, userid, langid, valsiid
ON definitions
FOR EACH ROW EXECUTE FUNCTION sync_definition_cache_fields();

-- Trigger on keywordmapping changes (affects cached_search_text)
DROP TRIGGER IF EXISTS trigger_sync_definition_cache_from_keywords ON keywordmapping;
CREATE TRIGGER trigger_sync_definition_cache_from_keywords
AFTER INSERT OR UPDATE OR DELETE
ON keywordmapping
FOR EACH ROW EXECUTE FUNCTION sync_definition_cache_fields();

-- Trigger on valsi changes (affects cached_search_text)
DROP TRIGGER IF EXISTS trigger_sync_definition_cache_from_valsi ON valsi;
CREATE TRIGGER trigger_sync_definition_cache_from_valsi
AFTER UPDATE OF word, rafsi
ON valsi
FOR EACH ROW EXECUTE FUNCTION sync_definition_cache_fields();

-- Trigger on users changes (affects cached_username)
DROP TRIGGER IF EXISTS trigger_sync_definition_cache_from_users ON users;
CREATE TRIGGER trigger_sync_definition_cache_from_users
AFTER UPDATE OF username
ON users
FOR EACH ROW EXECUTE FUNCTION sync_definition_cache_fields();

-- Trigger on languages changes (affects cached_langrealname)
DROP TRIGGER IF EXISTS trigger_sync_definition_cache_from_languages ON languages;
CREATE TRIGGER trigger_sync_definition_cache_from_languages
AFTER UPDATE OF realname
ON languages
FOR EACH ROW EXECUTE FUNCTION sync_definition_cache_fields();

-- Trigger on valsitypes changes (affects cached_type_name)
DROP TRIGGER IF EXISTS trigger_sync_definition_cache_from_valsitypes ON valsitypes;
CREATE TRIGGER trigger_sync_definition_cache_from_valsitypes
AFTER UPDATE OF descriptor
ON valsitypes
FOR EACH ROW EXECUTE FUNCTION sync_definition_cache_fields();
