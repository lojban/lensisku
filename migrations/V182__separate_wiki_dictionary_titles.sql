-- Dictionary entries and wiki pages have independent title namespaces.
ALTER TABLE public.valsi DROP CONSTRAINT IF EXISTS valsi_word_source_langid_key;

CREATE UNIQUE INDEX IF NOT EXISTS idx_valsi_dictionary_unique_word_source_lang
ON public.valsi (word, source_langid)
WHERE typeid <> 16;

-- The wiki namespace already has idx_valsi_wiki_unique_word_source_lang (V150).
