-- Restore the final-syllable stress in the mojibake Berlin headword.
-- Keep the valsi ID so its definition, history, and references stay attached.
UPDATE valsi AS v
SET word = 'berLIN'
WHERE v.valsiid = 29863
  AND v.source_langid = 1
  AND v.typeid = 3
  AND v.word = convert_from(
      decode('6265726cc383c283c386c292c383c286c3a2c280c299c383c283c3a2c280c29ac383c282c382c2ac6e', 'hex'),
      'UTF8'
  )
  AND EXISTS (
      SELECT 1
      FROM definitions AS d
      WHERE d.definitionid = 67878 AND d.valsiid = v.valsiid
  );
