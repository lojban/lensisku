-- Allow empty/NULL meaning in natlangwords so gloss keywords with no meaning
-- (e.g. "cmmevla-brivla merger" with meaning "") can be stored.
-- The constraint natlangwords_meaning_nonempty may come from an imported schema.
ALTER TABLE natlangwords
DROP CONSTRAINT IF EXISTS natlangwords_meaning_nonempty;
