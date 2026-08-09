-- Speed up user contribution lookups (versions by author, newest first).
CREATE INDEX IF NOT EXISTS idx_definition_versions_user_id_created_at
ON definition_versions (user_id, created_at DESC);
