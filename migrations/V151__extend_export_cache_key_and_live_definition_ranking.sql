-- Extend cached_dictionary_exports so its key captures every option that affects output,
-- and provide a live, deterministic "best definition" ranking so the export can actually
-- honor the positive_scores_only option.

-- 1. Add the missing cache-key columns and mark all existing cached exports stale so they
--    are regenerated with the new logic on the next request.
ALTER TABLE cached_dictionary_exports
ADD COLUMN source_language_tag TEXT NOT NULL DEFAULT 'jbo';

ALTER TABLE cached_dictionary_exports
ADD COLUMN positive_scores_only BOOLEAN NOT NULL DEFAULT true;

-- Force regeneration of any existing rows (their content was produced under the old,
-- underspecified cache model).  This is cache data and can be rebuilt on demand.
UPDATE cached_dictionary_exports
SET created_at = '1970-01-01T00:00:00+00:00';

-- 2. Replace the old unique key/index with the full cache identity.
DROP INDEX IF EXISTS idx_cached_exports_lookup;

-- Robustly drop the old unique constraint no matter what Postgres named it.
DO $$
DECLARE
    con_name text;
BEGIN
    SELECT c.conname INTO con_name
    FROM pg_constraint c
    JOIN pg_class t ON t.oid = c.conrelid
    JOIN pg_attribute a1 ON a1.attrelid = c.conrelid AND a1.attnum = c.conkey[1]
    JOIN pg_attribute a2 ON a2.attrelid = c.conrelid AND a2.attnum = c.conkey[2]
    WHERE t.relname = 'cached_dictionary_exports'
      AND c.contype = 'u'
      AND array_length(c.conkey, 1) = 2
      AND a1.attname = 'language_tag'
      AND a2.attname = 'format';

    IF con_name IS NOT NULL THEN
        EXECUTE format('ALTER TABLE cached_dictionary_exports DROP CONSTRAINT %I', con_name);
    END IF;
END $$;

CREATE UNIQUE INDEX idx_cached_exports_full_key
ON cached_dictionary_exports (language_tag, source_language_tag, format, positive_scores_only);

CREATE INDEX IF NOT EXISTS idx_cached_exports_cleanup
ON cached_dictionary_exports (created_at);

-- 3. Live best-definition ranking used by all export generators.
--    Replaces valsibestguesses so positive_scores_only=false can include zero/negative
--    definitions while positive_scores_only=true keeps the old positive-only behaviour.
CREATE OR REPLACE FUNCTION export_best_definitions(p_langid integer, p_positive_only boolean)
RETURNS TABLE(valsiid integer, definitionid integer, score bigint) AS $$
WITH definition_scores AS (
    SELECT d.definitionid, d.valsiid, d.langid,
           COALESCE(SUM(dv.value), 0)::bigint AS score
    FROM definitions d
    LEFT JOIN definitionvotes dv ON dv.definitionid = d.definitionid
    WHERE d.langid = p_langid
    GROUP BY d.definitionid, d.valsiid, d.langid
),
ranked AS (
    SELECT DISTINCT ON (ds.valsiid)
        ds.valsiid,
        ds.definitionid,
        ds.score
    FROM definition_scores ds
    WHERE ds.score > 0 OR p_positive_only = false
    ORDER BY ds.valsiid, ds.score DESC, ds.definitionid ASC
)
SELECT r.valsiid, r.definitionid, r.score FROM ranked r;
$$ LANGUAGE sql STABLE;

-- 4. Invalidate affected cached exports when definitions or votes change.
CREATE OR REPLACE FUNCTION invalidate_cached_exports_for_valsi_lang(
    p_valsiid integer,
    p_langid integer
) RETURNS void AS $$
BEGIN
    DELETE FROM cached_dictionary_exports cde
    USING valsi v, languages target_lang, languages source_lang
    WHERE v.valsiid = p_valsiid
      AND target_lang.langid = p_langid
      AND cde.language_tag = target_lang.tag
      AND source_lang.langid = v.source_langid
      AND cde.source_language_tag = source_lang.tag;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION trg_invalidate_cached_exports_for_definition()
RETURNS TRIGGER AS $$
DECLARE
    _old_valsiid integer;
    _old_langid integer;
    _new_valsiid integer;
    _new_langid integer;
BEGIN
    IF TG_OP = 'DELETE' THEN
        PERFORM invalidate_cached_exports_for_valsi_lang(OLD.valsiid, OLD.langid);
        RETURN OLD;
    END IF;

    _new_valsiid := NEW.valsiid;
    _new_langid := NEW.langid;

    IF TG_OP = 'UPDATE' THEN
        _old_valsiid := OLD.valsiid;
        _old_langid := OLD.langid;
        IF _old_valsiid IS DISTINCT FROM _new_valsiid OR _old_langid IS DISTINCT FROM _new_langid THEN
            PERFORM invalidate_cached_exports_for_valsi_lang(_old_valsiid, _old_langid);
        END IF;
    END IF;

    PERFORM invalidate_cached_exports_for_valsi_lang(_new_valsiid, _new_langid);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION trg_invalidate_cached_exports_for_vote()
RETURNS TRIGGER AS $$
DECLARE
    _old_valsiid integer;
    _old_langid integer;
    _new_valsiid integer;
    _new_langid integer;
BEGIN
    IF TG_OP = 'DELETE' THEN
        PERFORM invalidate_cached_exports_for_valsi_lang(OLD.valsiid, OLD.langid);
        RETURN OLD;
    END IF;

    _new_valsiid := NEW.valsiid;
    _new_langid := NEW.langid;

    IF TG_OP = 'UPDATE' THEN
        _old_valsiid := OLD.valsiid;
        _old_langid := OLD.langid;
        IF _old_valsiid IS DISTINCT FROM _new_valsiid OR _old_langid IS DISTINCT FROM _new_langid THEN
            PERFORM invalidate_cached_exports_for_valsi_lang(_old_valsiid, _old_langid);
        END IF;
    END IF;

    PERFORM invalidate_cached_exports_for_valsi_lang(_new_valsiid, _new_langid);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS invalidate_cached_exports_on_definition_change ON definitions;
CREATE TRIGGER invalidate_cached_exports_on_definition_change
AFTER INSERT OR UPDATE OR DELETE ON definitions
FOR EACH ROW EXECUTE FUNCTION trg_invalidate_cached_exports_for_definition();

DROP TRIGGER IF EXISTS invalidate_cached_exports_on_vote_change ON definitionvotes;
CREATE TRIGGER invalidate_cached_exports_on_vote_change
AFTER INSERT OR UPDATE OR DELETE ON definitionvotes
FOR EACH ROW EXECUTE FUNCTION trg_invalidate_cached_exports_for_vote();
