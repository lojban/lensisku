-- A shared key groups attested lujvo spellings and trivial SE conversions.
-- Checked distinguishes an analyzed word with no key from pending work.
ALTER TABLE valsi ADD COLUMN IF NOT EXISTS related_expansion_key TEXT;
ALTER TABLE valsi ADD COLUMN IF NOT EXISTS trivial_se_checked BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX IF NOT EXISTS idx_valsi_related_expansion_key
ON valsi (related_expansion_key) WHERE related_expansion_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_valsi_trivial_se_unchecked
ON valsi (valsiid)
WHERE source_langid = 1 AND typeid IN (1, 4, 7, 17) AND NOT trivial_se_checked;

CREATE OR REPLACE FUNCTION invalidate_valsi_trivial_se()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.word IS DISTINCT FROM NEW.word
       OR OLD.source_langid IS DISTINCT FROM NEW.source_langid
       OR (OLD.typeid IS DISTINCT FROM NEW.typeid
           AND NOT (OLD.typeid IN (4, 17) AND NEW.typeid IN (4, 17))) THEN
        NEW.related_expansion_key := NULL;
        NEW.trivial_se_checked := FALSE;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_invalidate_valsi_trivial_se
BEFORE UPDATE ON valsi
FOR EACH ROW EXECUTE FUNCTION invalidate_valsi_trivial_se();
