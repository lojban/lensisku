-- PostgreSQL 18 + pgvector: use HNSW for semantic search instead of IVFFlat.
-- IVFFlat was kept for older Postgres (see V109). HNSW does not need a lists
-- training pass and ranks more stably for our embedding size.
--
-- QA: semantic search (/search with embeddings) still returns nearest definitions;
-- no UI change. Reindex may take a few minutes on a full dictionary.

DROP INDEX IF EXISTS idx_definitions_embedding_vector;

CREATE INDEX IF NOT EXISTS idx_definitions_embedding_vector ON definitions
USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64)
WHERE embedding IS NOT NULL;
