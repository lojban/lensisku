-- V99 tried to drop valsi_unique_word_nospaces with DROP CONSTRAINT, but the
-- object is a UNIQUE INDEX (from the base dump), so it was never removed.

DROP INDEX IF EXISTS valsi_unique_word_nospaces;
