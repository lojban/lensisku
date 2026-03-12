-- Fix: valsi trigger only fired on UPDATE OF word, rafsi, so when a valsi's typeid
-- (or source_langid) was changed, definitions.cached_type_name (and cached_source_langid)
-- stayed stale. Semantic search uses live vt.descriptor, so it showed the correct
-- type_name (e.g. "experimental gismu") while the definitions API showed cached "gismu".
-- Include typeid and source_langid so the cache is refreshed when valsi type changes.

-- One-time backfill: refresh cached_type_name, cached_typeid, cached_source_langid
-- for definitions where valsi type/source_langid changed and cache was not updated.
UPDATE definitions d
SET
    cached_type_name = vt.descriptor,
    cached_typeid = v.typeid,
    cached_source_langid = v.source_langid
FROM valsi v
JOIN valsitypes vt ON v.typeid = vt.typeid
WHERE d.valsiid = v.valsiid
  AND (d.cached_typeid IS DISTINCT FROM v.typeid
       OR d.cached_type_name IS DISTINCT FROM vt.descriptor
       OR d.cached_source_langid IS DISTINCT FROM v.source_langid);

DROP TRIGGER IF EXISTS trigger_sync_definition_cache_from_valsi ON valsi;
CREATE TRIGGER trigger_sync_definition_cache_from_valsi
AFTER UPDATE OF word, rafsi, typeid, source_langid
ON valsi
FOR EACH ROW EXECUTE FUNCTION sync_definition_cache_fields();
