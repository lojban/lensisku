-- Add definition_link_id to threads table to support discussions on definition links
ALTER TABLE threads
ADD COLUMN definition_link_id INTEGER REFERENCES definition_links(id) ON DELETE CASCADE;

CREATE INDEX idx_threads_definition_link_id ON threads(definition_link_id);
