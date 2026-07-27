-- Add native wiki as a new valsi type.
-- Wiki pages are stored as valsi rows with typeid = 16 and one definitions row
-- holding the markdown body. Existing definition_versions, can_edit_definition,
-- and owner_only provide history and edit permissions.

INSERT INTO valsitypes (typeid, descriptor) VALUES (16, 'wiki')
ON CONFLICT (typeid) DO NOTHING;

-- Ensure descriptor is unique if it was inserted with a different id.
INSERT INTO valsitypes (typeid, descriptor) VALUES (16, 'wiki')
ON CONFLICT (descriptor) DO NOTHING;

-- Enforce one native wiki page per word/source-language combination.
CREATE UNIQUE INDEX IF NOT EXISTS idx_valsi_wiki_unique_word_source_lang
ON valsi (word, source_langid)
WHERE typeid = 16;
