ALTER TABLE threads
ADD COLUMN IF NOT EXISTS import_source TEXT,
ADD COLUMN IF NOT EXISTS import_ref JSONB;

ALTER TABLE comments
ADD COLUMN IF NOT EXISTS import_source TEXT,
ADD COLUMN IF NOT EXISTS import_ref JSONB;

CREATE INDEX IF NOT EXISTS idx_threads_import_source
ON threads (import_source)
WHERE import_source IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_comments_import_source
ON comments (import_source)
WHERE import_source IS NOT NULL;
