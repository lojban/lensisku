-- Experimental gismu (typeid 7): append the implicit 4-letter rafsi
-- (word minus final vowel) on definitions.rafsi when that form is not already
-- claimed by any official gismu rafsi token (short or 4-letter after V169).
-- Skip brod-stem. Does not invent rafsi that collide with the official set.

WITH official_rafsi AS (
    SELECT DISTINCT tok AS rafsi
    FROM valsi v
    CROSS JOIN LATERAL unnest(
        string_to_array(
            trim(both FROM regexp_replace(COALESCE(v.rafsi, ''), '\s+', ' ', 'g')),
            ' '
        )
    ) AS tok
    WHERE v.typeid = 1
      AND v.source_langid = 1
      AND tok <> ''
),
candidates AS (
    SELECT
        d.definitionid,
        trim(both FROM regexp_replace(COALESCE(d.rafsi, ''), '\s+', ' ', 'g')) AS norm,
        left(v.word, char_length(v.word) - 1) AS four
    FROM definitions d
    JOIN valsi v ON v.valsiid = d.valsiid
    WHERE v.typeid = 7
      AND v.source_langid = 1
      AND char_length(v.word) = 5
      AND right(v.word, 1) IN ('a', 'e', 'i', 'o', 'u')
      AND left(v.word, char_length(v.word) - 1) <> 'brod'
)
UPDATE definitions AS d
SET rafsi = CASE
        WHEN c.norm = '' THEN c.four
        ELSE c.norm || ' ' || c.four
    END
FROM candidates c
WHERE d.definitionid = c.definitionid
  AND (c.norm = '' OR NOT (c.four = ANY (string_to_array(c.norm, ' '))))
  AND NOT EXISTS (
      SELECT 1 FROM official_rafsi o WHERE o.rafsi = c.four
  );
