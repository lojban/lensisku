-- Official gismu always allow a 4-letter rafsi = word minus final vowel
-- (CLL / vlazba get_candid), except the broda-series stem "brod".
-- Append that form to valsi.rafsi when missing so API, search cache, and
-- custom rafsi maps expose it (e.g. datni → datn).
--
-- AFTER UPDATE OF rafsi on valsi syncs definitions.cached_rafsi via
-- trigger_sync_definition_cache_from_valsi.

UPDATE valsi AS v
SET rafsi = CASE
        WHEN src.norm = '' THEN src.four
        WHEN NOT (src.four = ANY (string_to_array(src.norm, ' '))) THEN src.norm || ' ' || src.four
        ELSE src.norm
    END
FROM (
    SELECT
        valsiid,
        -- normalize runs of whitespace in existing rafsi
        trim(both FROM regexp_replace(COALESCE(rafsi, ''), '\s+', ' ', 'g')) AS norm,
        left(word, char_length(word) - 1) AS four
    FROM valsi
    WHERE typeid = 1
      AND source_langid = 1
      AND char_length(word) = 5
      AND right(word, 1) IN ('a', 'e', 'i', 'o', 'u')
      AND word NOT IN ('broda', 'brode', 'brodi', 'brodo', 'brodu')
) AS src
WHERE v.valsiid = src.valsiid
  AND (
      src.norm = ''
      OR NOT (src.four = ANY (string_to_array(src.norm, ' ')))
  );
