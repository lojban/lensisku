-- Invalidate embeddings to regenerate with raw text (no preprocessing)
-- This will trigger the background task to recalculate embeddings for all definitions
UPDATE definitions
SET embedding = NULL
WHERE langid != 1 AND definition != '' AND embedding IS NOT NULL;
