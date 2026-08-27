-- Membership checks ("does this user's collection already include definition X?") look up
-- collection_items by definition_id. UNIQUE (collection_id, definition_id) does not help
-- that access path; a partial index on definition_id does.
CREATE INDEX IF NOT EXISTS idx_collection_items_definition_id
    ON collection_items (definition_id)
    WHERE definition_id IS NOT NULL;
