-- Bidirectional linking table between definitions
CREATE TABLE IF NOT EXISTS definition_links (
    id SERIAL PRIMARY KEY,
    definition_id INTEGER NOT NULL REFERENCES definitions(definitionid) ON DELETE CASCADE,
    translation_id INTEGER NOT NULL REFERENCES definitions(definitionid) ON DELETE CASCADE,
    created_by INTEGER REFERENCES users(userid),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(definition_id, translation_id),
    CHECK(definition_id != translation_id)
);

CREATE INDEX idx_def_links_definition ON definition_links(definition_id);
CREATE INDEX idx_def_links_translation ON definition_links(translation_id);
