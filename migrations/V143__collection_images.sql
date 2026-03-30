-- Optional logo/avatar per collection (same storage pattern as user_profile_images)
CREATE TABLE collection_images (
    collection_id INTEGER PRIMARY KEY REFERENCES collections (collection_id) ON DELETE CASCADE,
    image_data BYTEA NOT NULL,
    mime_type TEXT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);
