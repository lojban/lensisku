-- Add cached glosswords field for fast full-word matching
-- This eliminates the need for JOINs or subqueries when checking for glossword matches

-- Add cached glosswords field (space-separated lowercase glosswords for place=0)
ALTER TABLE definitions ADD COLUMN IF NOT EXISTS cached_glosswords TEXT;

-- Populate cached_glosswords with all glosswords (place=0) as space-separated lowercase words
UPDATE definitions d
SET cached_glosswords = COALESCE((
    SELECT string_agg(LOWER(n.word), ' ' ORDER BY n.word)
    FROM keywordmapping k
    JOIN natlangwords n ON k.natlangwordid = n.wordid
    WHERE k.definitionid = d.definitionid
    AND k.place = 0
), '');

-- Create GIN index for fast full-word matching on cached_glosswords
CREATE INDEX IF NOT EXISTS idx_definitions_cached_glosswords_gin 
ON definitions USING gin(cached_glosswords gin_trgm_ops);

-- Update sync function to include cached_glosswords
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
            cached_glosswords = COALESCE((
                SELECT string_agg(LOWER(n.word), ' ' ORDER BY n.word)
                FROM keywordmapping k
                JOIN natlangwords n ON k.natlangwordid = n.wordid
                WHERE k.definitionid = d.definitionid
                AND k.place = 0
            ), ''),
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
        -- Update cached_glosswords and cached_search_text when keywords change
        UPDATE definitions d
        SET 
            cached_glosswords = COALESCE((
                SELECT string_agg(LOWER(n.word), ' ' ORDER BY n.word)
                FROM keywordmapping k
                JOIN natlangwords n ON k.natlangwordid = n.wordid
                WHERE k.definitionid = d.definitionid
                AND k.place = 0
            ), ''),
            cached_search_text = LOWER(
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
    ELSIF TG_TABLE_NAME = 'natlangwords' THEN
        -- Update cached_glosswords and cached_search_text when natlangwords change
        UPDATE definitions d
        SET 
            cached_glosswords = COALESCE((
                SELECT string_agg(LOWER(n.word), ' ' ORDER BY n.word)
                FROM keywordmapping k
                JOIN natlangwords n ON k.natlangwordid = n.wordid
                WHERE k.definitionid = d.definitionid
                AND k.place = 0
            ), ''),
            cached_search_text = LOWER(
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
        WHERE d.definitionid IN (
            SELECT k.definitionid
            FROM keywordmapping k
            WHERE k.natlangwordid = COALESCE(NEW.wordid, OLD.wordid)
        );
    END IF;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Trigger on natlangwords changes (affects cached_glosswords and cached_search_text)
DROP TRIGGER IF EXISTS trigger_sync_definition_cache_from_natlangwords ON natlangwords;
CREATE TRIGGER trigger_sync_definition_cache_from_natlangwords
AFTER UPDATE OF word, meaning OR DELETE
ON natlangwords
FOR EACH ROW EXECUTE FUNCTION sync_definition_cache_fields();
