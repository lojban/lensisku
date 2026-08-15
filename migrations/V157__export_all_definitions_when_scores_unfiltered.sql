-- When positive_scores_only=false, export every definition for each word — not only
-- the single best-scoring one. DISTINCT ON meant non-positive (and alternate) definitions
-- were still omitted whenever any positive-scored definition existed for that word, so
-- disabling "only positive scores" still produced a positive-score-only looking dump.
--
-- positive_scores_only=true keeps the previous behaviour: one best definition per word
-- with score > 0.

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
filtered AS (
    SELECT ds.valsiid, ds.definitionid, ds.score
    FROM definition_scores ds
    WHERE ds.score > 0 OR p_positive_only = false
),
-- Positive-only mode: one deterministic best definition per word.
best_positive AS (
    SELECT DISTINCT ON (f.valsiid)
        f.valsiid,
        f.definitionid,
        f.score
    FROM filtered f
    WHERE p_positive_only = true
    ORDER BY f.valsiid, f.score DESC, f.definitionid ASC
)
SELECT b.valsiid, b.definitionid, b.score FROM best_positive b
UNION ALL
SELECT f.valsiid, f.definitionid, f.score FROM filtered f
WHERE p_positive_only = false;
$$ LANGUAGE sql STABLE;

-- Force regeneration of cached exports under the new ranking rules.
UPDATE cached_dictionary_exports
SET created_at = '1970-01-01T00:00:00+00:00';
