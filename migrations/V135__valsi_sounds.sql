CREATE TABLE valsi_sounds (
    id SERIAL PRIMARY KEY,
    valsi_id INTEGER NOT NULL REFERENCES valsi(valsiid) ON DELETE CASCADE,
    sound_data BYTEA NOT NULL,
    mime_type TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(valsi_id)
);

CREATE INDEX idx_valsi_sounds_valsi ON valsi_sounds(valsi_id);

CREATE OR REPLACE FUNCTION cleanup_valsi_sounds() RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM valsi_sounds WHERE valsi_id = OLD.valsiid;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER cleanup_valsi_sounds_trigger
    BEFORE DELETE ON valsi
    FOR EACH ROW
    EXECUTE FUNCTION cleanup_valsi_sounds();
