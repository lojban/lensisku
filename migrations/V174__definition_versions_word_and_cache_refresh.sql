-- Snapshot the headword on each definition version so renames/reattaches are visible in history.
ALTER TABLE definition_versions
    ADD COLUMN IF NOT EXISTS word TEXT;

-- Backfill from the valsi currently pointed at by each version row.
UPDATE definition_versions dv
SET word = v.word
FROM valsi v
WHERE dv.valsiid = v.valsiid
  AND (dv.word IS NULL OR dv.word = '');

-- Explicit refresh for definition-scoped valsi cache fields.
-- Triggers on valsi UPDATE already refresh dependents; definitions.valsiid changes also fire
-- trigger_sync_definition_cache (UPDATE OF … valsiid …), but reattach call sites can invoke
-- this helper for a clear, idempotent refresh of denormalized valsi fields + search text.
CREATE OR REPLACE FUNCTION refresh_definition_cached_valsi(p_definitionid INTEGER)
RETURNS VOID AS $$
DECLARE
    v_word TEXT;
    v_rafsi TEXT;
    v_source_langid INTEGER;
    v_typeid SMALLINT;
    v_type_name TEXT;
    v_cached_decomposition TEXT;
    v_canonical_word TEXT;
    d_definition TEXT;
    d_notes TEXT;
    d_selmaho TEXT;
    d_rafsi TEXT;
    u_username TEXT;
    l_realname TEXT;
    kw_gloss TEXT;
    kw_text TEXT;
BEGIN
    SELECT COALESCE(string_agg(LOWER(n.word), '|' ORDER BY n.word) FILTER (WHERE k.place = 0), ''),
           COALESCE(string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' '), '')
    INTO kw_gloss, kw_text
    FROM keywordmapping k
    JOIN natlangwords n ON k.natlangwordid = n.wordid
    WHERE k.definitionid = p_definitionid;

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
    WHERE d.definitionid = p_definitionid;

    IF NOT FOUND THEN
        RETURN;
    END IF;

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
    WHERE definitionid = p_definitionid;
END;
$$ LANGUAGE plpgsql;
