-- Non-canonical lujvo (score-suboptimal spelling) as a separate valsitype.
-- valsi.canonical_word: score-optimal form for typeid 17; for typeid 4, NULL means
-- unchecked (background batch pending) and equal-to-word means verified canonical.

INSERT INTO valsitypes (typeid, descriptor) VALUES (17, 'non-canonical lujvo')
ON CONFLICT (typeid) DO NOTHING;

INSERT INTO valsitypes (typeid, descriptor) VALUES (17, 'non-canonical lujvo')
ON CONFLICT (descriptor) DO NOTHING;

ALTER TABLE valsi ADD COLUMN IF NOT EXISTS canonical_word TEXT;

ALTER TABLE definitions ADD COLUMN IF NOT EXISTS cached_canonical_word TEXT;

-- Clear classification when the spelling changes so the background batch rechecks.
CREATE OR REPLACE FUNCTION invalidate_valsi_canonical_word()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.word IS DISTINCT FROM NEW.word THEN
        NEW.canonical_word := NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_invalidate_valsi_canonical_word ON valsi;
CREATE TRIGGER trigger_invalidate_valsi_canonical_word
BEFORE UPDATE ON valsi
FOR EACH ROW EXECUTE FUNCTION invalidate_valsi_canonical_word();

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
    v_canonical_word TEXT;
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
        SELECT COALESCE(string_agg(LOWER(n.word), '|' ORDER BY n.word) FILTER (WHERE k.place = 0), ''),
               COALESCE(string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' '), '')
        INTO kw_gloss, kw_text
        FROM keywordmapping k
        JOIN natlangwords n ON k.natlangwordid = n.wordid
        WHERE k.definitionid = target_definitionid;
    END IF;

    IF TG_TABLE_NAME = 'definitions' THEN
        SELECT d.definition, d.notes, d.selmaho, d.rafsi,
               v.word, v.rafsi, v.cached_decomposition, v.canonical_word, v.source_langid, v.typeid,
               vt.descriptor, u.username, l.realname
        INTO d_definition, d_notes, d_selmaho, d_rafsi,
             v_word, v_rafsi, v_cached_decomposition, v_canonical_word, v_source_langid, v_typeid,
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
            cached_canonical_word = v_canonical_word,
            cached_source_langid = v_source_langid,
            cached_typeid = v_typeid,
            cached_glosswords = kw_gloss,
            cached_search_text = LOWER(
                COALESCE(v_word, '') || ' ' ||
                COALESCE(d_rafsi, v_rafsi, '') || ' ' ||
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
            ),
            embedding = NULL
        WHERE d.definitionid = target_definitionid;

    ELSIF TG_TABLE_NAME = 'valsi' THEN
        WITH def_ids AS (
            SELECT definitionid FROM definitions WHERE valsiid = target_valsiid
        ),
        valsi_data AS (
            SELECT v.word, v.rafsi, v.cached_decomposition, v.canonical_word,
                   v.source_langid, v.typeid, vt.descriptor
            FROM valsi v
            JOIN valsitypes vt ON v.typeid = vt.typeid
            WHERE v.valsiid = target_valsiid
        ),
        keyword_agg AS (
            SELECT k.definitionid,
                   string_agg(LOWER(n.word), '|' ORDER BY n.word) FILTER (WHERE k.place = 0) AS glosswords,
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
            cached_canonical_word = vd.canonical_word,
            cached_source_langid = vd.source_langid,
            cached_typeid = vd.typeid,
            cached_type_name = vd.descriptor,
            cached_glosswords = COALESCE(ka.glosswords, ''),
            cached_search_text = LOWER(
                COALESCE(vd.word, '') || ' ' ||
                COALESCE(definitions.rafsi, vd.rafsi, '') || ' ' ||
                COALESCE(definitions.definition, '') || ' ' ||
                COALESCE(definitions.notes, '') || ' ' ||
                COALESCE(definitions.selmaho, '') || ' ' ||
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
                   string_agg(LOWER(n.word), '|' ORDER BY n.word) FILTER (WHERE k.place = 0) AS glosswords,
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
            ),
            embedding = NULL
        FROM affected
        LEFT JOIN keyword_agg ka ON ka.definitionid = affected.definitionid
        WHERE definitions.definitionid = affected.definitionid;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Index for the background classification drain.
CREATE INDEX IF NOT EXISTS idx_valsi_lujvo_unchecked_canonical
ON valsi (valsiid)
WHERE typeid = 4 AND source_langid = 1 AND canonical_word IS NULL;
