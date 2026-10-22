-- Flashcard study discussions reuse threads.definition_link_id to store the
-- flashcard id (with collection_id). That value is not a definition_links.id,
-- so the original FK blocks creating those threads. Drop the FK; keep the column
-- and index for lookups.
ALTER TABLE public.threads
  DROP CONSTRAINT IF EXISTS threads_definition_link_id_fkey;
