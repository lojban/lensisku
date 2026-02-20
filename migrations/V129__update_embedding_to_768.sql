-- Update embedding column for onnx-community/embeddinggemma-300m-ONNX (768 dimensions).
-- All existing embeddings are cleared (NULL) as the old 384-dim vectors are
-- incompatible with the new model.  They will be regenerated on next indexing run.

DROP INDEX IF EXISTS idx_definitions_embedding_vector;

ALTER TABLE definitions
    ALTER COLUMN embedding TYPE vector(768) USING NULL;

-- Recreate ivfflat index for 768 dimensions.
CREATE INDEX IF NOT EXISTS idx_definitions_embedding_vector ON definitions
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100)
    WHERE embedding IS NOT NULL;
