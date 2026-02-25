CREATE TABLE collection_item_sounds (
    id SERIAL PRIMARY KEY,
    item_id INTEGER NOT NULL REFERENCES collection_items(item_id) ON DELETE CASCADE,
    sound_data BYTEA NOT NULL,
    mime_type TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(item_id)
);

-- Add indexes for performance
CREATE INDEX idx_collection_item_sounds_item ON collection_item_sounds(item_id);

-- Trigger to clean up sound on item deletion (if not handled by ON DELETE CASCADE, but images had it)
-- Actually images had a custom trigger because it wasn't CASCADE? Wait.
-- Image migration V55:
-- item_id INTEGER NOT NULL REFERENCES collection_items(item_id) ON DELETE CASCADE,
-- It also had a trigger. Let's see why.
-- Maybe to handle storage cleanup if there was external files? But it's BYTEA.
-- A trigger mirroring images is safer for consistency.

CREATE OR REPLACE FUNCTION cleanup_item_sounds() RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM collection_item_sounds WHERE item_id = OLD.item_id;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER cleanup_collection_item_sounds
    BEFORE DELETE ON collection_items
    FOR EACH ROW
    EXECUTE FUNCTION cleanup_item_sounds();
