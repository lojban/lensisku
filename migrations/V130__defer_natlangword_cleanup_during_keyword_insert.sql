-- Glosswords were not persisted because the trigger on natlangwords runs
-- delete_orphaned_natlangwords() after each INSERT and deletes new rows before
-- keywordmapping is inserted. Only clean up orphans created one day ago or earlier,
-- so recent inserts (same transaction or parallel users) are never deleted.

CREATE OR REPLACE FUNCTION public.delete_orphaned_natlangwords() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM natlangwords n
    WHERE NOT EXISTS (
        SELECT 1 FROM threads t WHERE t.natlangwordid = n.wordid
    )
    AND NOT EXISTS (
        SELECT 1 FROM natlangwordvotes v WHERE v.natlangwordid = n.wordid
    )
    AND NOT EXISTS (
        SELECT 1 FROM keywordmapping k WHERE k.natlangwordid = n.wordid
    )
    AND n.time < (EXTRACT(EPOCH FROM (NOW() - INTERVAL '1 day'))::bigint);
END;
$$;
