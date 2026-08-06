-- Store rafsi on the definition row for experimental gismu (typeid 7) and
-- experimental cmavo (typeid 8) while keeping it on valsi for official gismu,
-- cmavo and other types. Also hide rafsi/gloss/place keywords for phrases.

-- 1. Add per-definition rafsi storage.
ALTER TABLE definitions ADD COLUMN IF NOT EXISTS rafsi TEXT;

-- 2. Version rafsi changes for definitions.
ALTER TABLE definition_versions ADD COLUMN IF NOT EXISTS rafsi TEXT;

-- 3. Update the cache-sync function so definition-level rafsi overrides valsi-level rafsi.
CREATE OR REPLACE FUNCTION sync_definition_cache_fields()
RETURNS TRIGGER AS $$
DECLARE
    target_definitionid INTEGER;
    target_valsiid INTEGER;
    target_natlangwordid INTEGER;
    kw_gloss TEXT := '';
    kw_text TEXT := '';
    v_word TEXT;
    v_rafsi TEXT;
    d_rafsi TEXT;
    v_cached_decomposition TEXT;
    v_source_langid INTEGER;
    v_typeid INTEGER;
    v_type_name TEXT;
    u_username TEXT;
    l_realname TEXT;
    d_definition TEXT;
    d_notes TEXT;
    d_selmaho TEXT;
BEGIN
    IF TG_TABLE_NAME = 'definitions' THEN
        target_definitionid := COALESCE(NEW.definitionid, OLD.definitionid);
        target_valsiid := COALESCE(NEW.valsiid, OLD.valsiid);
    ELSIF TG_TABLE_NAME = 'keywordmapping' THEN
        target_definitionid := COALESCE(NEW.definitionid, OLD.definitionid);
    ELSIF TG_TABLE_NAME = 'valsi' THEN
        target_valsiid := COALESCE(NEW.valsiid, OLD.valsiid);
    ELSIF TG_TABLE_NAME = 'natlangwords' THEN
        target_natlangwordid := COALESCE(NEW.wordid, OLD.wordid);
    END IF;

    IF target_definitionid IS NOT NULL THEN
        SELECT COALESCE(string_agg(LOWER(n.word), ' ' ORDER BY n.word), ''),
               COALESCE(string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' '), '')
        INTO kw_gloss, kw_text
        FROM keywordmapping k
        JOIN natlangwords n ON k.natlangwordid = n.wordid
        WHERE k.definitionid = target_definitionid;
    END IF;

    IF TG_TABLE_NAME = 'definitions' THEN
        SELECT d.definition, d.notes, d.selmaho, d.rafsi,
               v.word, v.rafsi, v.cached_decomposition, v.source_langid, v.typeid,
               vt.descriptor, u.username, l.realname
        INTO d_definition, d_notes, d_selmaho, d_rafsi,
             v_word, v_rafsi, v_cached_decomposition, v_source_langid, v_typeid,
             v_type_name, u_username, l_realname
        FROM definitions d
        JOIN valsi v ON d.valsiid = v.valsiid
        JOIN users u ON d.userid = u.userid
        JOIN languages l ON d.langid = l.langid
        JOIN valsitypes vt ON v.typeid = vt.typeid
        WHERE d.definitionid = target_definitionid;

        UPDATE definitions
        SET
            cached_username = u_username,
            cached_langrealname = l_realname,
            cached_type_name = v_type_name,
            cached_valsiword = v_word,
            cached_rafsi = COALESCE(d_rafsi, v_rafsi),
            cached_decomposition = v_cached_decomposition,
            cached_source_langid = v_source_langid,
            cached_typeid = v_typeid,
            cached_glosswords = kw_gloss,
            cached_search_text = LOWER(
                COALESCE(v_word, '') || ' ' ||
                COALESCE(d_rafsi, v_rafsi) || ' ' ||
                COALESCE(d_definition, '') || ' ' ||
                COALESCE(d_notes, '') || ' ' ||
                COALESCE(d_selmaho, '') || ' ' ||
                COALESCE(kw_text, '')
            )
        WHERE definitionid = target_definitionid;

    ELSIF TG_TABLE_NAME = 'keywordmapping' THEN
        UPDATE definitions d
        SET
            cached_glosswords = kw_gloss,
            cached_search_text = LOWER(
                COALESCE(d.cached_valsiword, '') || ' ' ||
                COALESCE(d.cached_rafsi, '') || ' ' ||
                COALESCE(d.definition, '') || ' ' ||
                COALESCE(d.notes, '') || ' ' ||
                COALESCE(d.selmaho, '') || ' ' ||
                COALESCE(kw_text, '')
            )
        WHERE d.definitionid = target_definitionid;

    ELSIF TG_TABLE_NAME = 'valsi' THEN
        WITH def_ids AS (
            SELECT definitionid FROM definitions WHERE valsiid = target_valsiid
        ),
        valsi_data AS (
            SELECT v.word, v.rafsi, v.cached_decomposition, v.source_langid, v.typeid, vt.descriptor
            FROM valsi v
            JOIN valsitypes vt ON v.typeid = vt.typeid
            WHERE v.valsiid = target_valsiid
        ),
        keyword_agg AS (
            SELECT k.definitionid,
                   MAX(d2.definition) AS def_text,
                   MAX(d2.notes) AS notes_text,
                   MAX(d2.selmaho) AS selmaho_text,
                   string_agg(LOWER(n.word), ' ' ORDER BY n.word) FILTER (WHERE k.place = 0) AS glosswords,
                   string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' ') AS keyword_text
            FROM keywordmapping k
            JOIN natlangwords n ON k.natlangwordid = n.wordid
            JOIN definitions d2 ON d2.definitionid = k.definitionid
            WHERE d2.valsiid = target_valsiid
            GROUP BY k.definitionid
        )
        UPDATE definitions
        SET
            cached_valsiword = vd.word,
            cached_rafsi = COALESCE(definitions.rafsi, vd.rafsi),
            cached_decomposition = vd.cached_decomposition,
            cached_source_langid = vd.source_langid,
            cached_typeid = vd.typeid,
            cached_type_name = vd.descriptor,
            cached_glosswords = COALESCE(ka.glosswords, ''),
            cached_search_text = LOWER(
                COALESCE(vd.word, '') || ' ' ||
                COALESCE(definitions.rafsi, vd.rafsi) || ' ' ||
                COALESCE(ka.def_text, '') || ' ' ||
                COALESCE(ka.notes_text, '') || ' ' ||
                COALESCE(ka.selmaho_text, '') || ' ' ||
                COALESCE(ka.keyword_text, '')
            )
        FROM def_ids
        CROSS JOIN valsi_data vd
        LEFT JOIN keyword_agg ka ON ka.definitionid = def_ids.definitionid
        WHERE definitions.definitionid = def_ids.definitionid;

    ELSIF TG_TABLE_NAME = 'users' THEN
        UPDATE definitions d
        SET cached_username = NEW.username
        WHERE d.userid = NEW.userid;

    ELSIF TG_TABLE_NAME = 'languages' THEN
        UPDATE definitions d
        SET cached_langrealname = NEW.realname
        WHERE d.langid = NEW.langid;

    ELSIF TG_TABLE_NAME = 'valsitypes' THEN
        UPDATE definitions d
        SET
            cached_type_name = NEW.descriptor,
            cached_typeid = NEW.typeid
        WHERE d.cached_typeid = NEW.typeid;

    ELSIF TG_TABLE_NAME = 'natlangwords' THEN
        WITH affected AS (
            SELECT DISTINCT k.definitionid
            FROM keywordmapping k
            WHERE k.natlangwordid = target_natlangwordid
        ),
        keyword_agg AS (
            SELECT k.definitionid,
                   MAX(d2.cached_valsiword) AS cached_valsiword,
                   MAX(d2.cached_rafsi) AS cached_rafsi,
                   MAX(d2.definition) AS def_text,
                   MAX(d2.notes) AS notes_text,
                   MAX(d2.selmaho) AS selmaho_text,
                   string_agg(LOWER(n.word), ' ' ORDER BY n.word) FILTER (WHERE k.place = 0) AS glosswords,
                   string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' ') AS keyword_text
            FROM keywordmapping k
            JOIN natlangwords n ON k.natlangwordid = n.wordid
            JOIN definitions d2 ON d2.definitionid = k.definitionid
            JOIN affected a ON a.definitionid = k.definitionid
            GROUP BY k.definitionid
        )
        UPDATE definitions
        SET
            cached_glosswords = COALESCE(ka.glosswords, ''),
            cached_search_text = LOWER(
                COALESCE(ka.cached_valsiword, '') || ' ' ||
                COALESCE(ka.cached_rafsi, '') || ' ' ||
                COALESCE(ka.def_text, '') || ' ' ||
                COALESCE(ka.notes_text, '') || ' ' ||
                COALESCE(ka.selmaho_text, '') || ' ' ||
                COALESCE(ka.keyword_text, '')
            )
        FROM affected
        LEFT JOIN keyword_agg ka ON ka.definitionid = affected.definitionid
        WHERE definitions.definitionid = affected.definitionid;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- 4. Make sure the definitions trigger also fires when definitions.rafsi changes.
DROP TRIGGER IF EXISTS trigger_sync_definition_cache ON definitions;
CREATE TRIGGER trigger_sync_definition_cache
AFTER INSERT OR UPDATE OF definition, notes, selmaho, userid, langid, valsiid, rafsi
ON definitions
FOR EACH ROW EXECUTE FUNCTION sync_definition_cache_fields();

-- 5. Update the convenientdefinitions view so exports/collections see per-definition rafsi.
CREATE OR REPLACE VIEW convenientdefinitions AS
SELECT nd.definitionid,
    l.realname AS langrealname,
    l.tag,
    l.langid,
    v.valsiid,
    v.word,
    nd.definition,
    nd.notes,
    u.username,
    u.userid,
    nd."time",
    nd.definitionnum,
    COALESCE(nd.rafsi, v.rafsi) AS rafsi,
    nd.selmaho,
    nd.jargon
FROM public.definitions nd
JOIN public.languages l ON nd.langid = l.langid
JOIN public.valsi v ON nd.valsiid = v.valsiid
JOIN public.users u ON nd.userid = u.userid;

-- 6. Migrate existing rafsi for experimental gismu/cmavo from valsi to each definition.
UPDATE definitions d
SET rafsi = v.rafsi
FROM valsi v
WHERE d.valsiid = v.valsiid
  AND v.typeid IN (7, 8)
  AND v.rafsi IS NOT NULL;

-- 7. Clear valsi-level rafsi for experimental gismu/cmavo now that it lives on definitions.
UPDATE valsi
SET rafsi = NULL
WHERE typeid IN (7, 8)
  AND rafsi IS NOT NULL;
