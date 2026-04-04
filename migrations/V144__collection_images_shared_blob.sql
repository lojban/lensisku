-- Shared deduplicated blob store for collection covers and per-item card images.
-- Replaces per-collection cover table (collection_id PK) with global rows keyed by SHA-256.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Legacy: one cover row per collection
ALTER TABLE collection_images RENAME TO collection_cover_images_legacy;

CREATE TABLE collection_images (
    collection_image_id SERIAL PRIMARY KEY,
    content_sha256 BYTEA NOT NULL,
    image_data BYTEA NOT NULL,
    mime_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT collection_images_content_sha256_key UNIQUE (content_sha256)
);

CREATE INDEX idx_collection_images_created_at ON collection_images (created_at);

ALTER TABLE collections
    ADD COLUMN cover_collection_image_id INTEGER;

-- Distinct blobs from legacy covers
INSERT INTO collection_images (content_sha256, image_data, mime_type)
SELECT digest(image_data, 'sha256'), image_data, mime_type
FROM collection_cover_images_legacy
ON CONFLICT (content_sha256) DO NOTHING;

-- Distinct blobs from item images (still has BYTEA columns)
INSERT INTO collection_images (content_sha256, image_data, mime_type)
SELECT digest(image_data, 'sha256'), image_data, mime_type
FROM collection_item_images
ON CONFLICT (content_sha256) DO NOTHING;

ALTER TABLE collections
    ADD CONSTRAINT collections_cover_collection_image_id_fkey
    FOREIGN KEY (cover_collection_image_id) REFERENCES collection_images (collection_image_id);

UPDATE collections c
SET cover_collection_image_id = ci.collection_image_id
FROM collection_cover_images_legacy l
JOIN collection_images ci ON ci.content_sha256 = digest(l.image_data, 'sha256')
WHERE c.collection_id = l.collection_id;

ALTER TABLE collection_item_images ADD COLUMN collection_image_id INTEGER;

UPDATE collection_item_images cii
SET collection_image_id = ci.collection_image_id
FROM collection_images ci
WHERE ci.content_sha256 = digest(cii.image_data, 'sha256');

ALTER TABLE collection_item_images ALTER COLUMN collection_image_id SET NOT NULL;

ALTER TABLE collection_item_images DROP COLUMN image_data;
ALTER TABLE collection_item_images DROP COLUMN mime_type;

ALTER TABLE collection_item_images
    ADD CONSTRAINT collection_item_images_collection_image_id_fkey
    FOREIGN KEY (collection_image_id) REFERENCES collection_images (collection_image_id);

CREATE INDEX idx_collection_item_images_collection_image_id ON collection_item_images (collection_image_id);

DROP TABLE collection_cover_images_legacy;

-- Orphan cleanup: delete blob when no collection_items row and no collection cover references it.
CREATE OR REPLACE FUNCTION try_delete_collection_image_if_orphan(p_id INTEGER) RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
    IF p_id IS NULL THEN
        RETURN;
    END IF;
    IF EXISTS (SELECT 1 FROM collection_item_images WHERE collection_image_id = p_id) THEN
        RETURN;
    END IF;
    IF EXISTS (SELECT 1 FROM collections WHERE cover_collection_image_id = p_id) THEN
        RETURN;
    END IF;
    DELETE FROM collection_images WHERE collection_image_id = p_id;
END;
$$;

-- When deleting a collection, the row still holds cover_collection_image_id during BEFORE DELETE.
CREATE OR REPLACE FUNCTION try_delete_collection_image_if_orphan_for_deleted_collection(
    p_id INTEGER,
    p_excluded_collection_id INTEGER
) RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
    IF p_id IS NULL THEN
        RETURN;
    END IF;
    IF EXISTS (SELECT 1 FROM collection_item_images WHERE collection_image_id = p_id) THEN
        RETURN;
    END IF;
    IF EXISTS (
        SELECT 1 FROM collections
        WHERE cover_collection_image_id = p_id
          AND collection_id IS DISTINCT FROM p_excluded_collection_id
    ) THEN
        RETURN;
    END IF;
    DELETE FROM collection_images WHERE collection_image_id = p_id;
END;
$$;

CREATE OR REPLACE FUNCTION trg_collection_item_images_after_delete_cleanup() RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    PERFORM try_delete_collection_image_if_orphan(OLD.collection_image_id);
    RETURN OLD;
END;
$$;

CREATE TRIGGER collection_item_images_after_delete_cleanup
    AFTER DELETE ON collection_item_images
    FOR EACH ROW
    EXECUTE FUNCTION trg_collection_item_images_after_delete_cleanup();

CREATE OR REPLACE FUNCTION trg_collection_item_images_after_update_cleanup() RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    IF OLD.collection_image_id IS DISTINCT FROM NEW.collection_image_id THEN
        PERFORM try_delete_collection_image_if_orphan(OLD.collection_image_id);
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER collection_item_images_after_update_cleanup
    AFTER UPDATE OF collection_image_id ON collection_item_images
    FOR EACH ROW
    EXECUTE FUNCTION trg_collection_item_images_after_update_cleanup();

CREATE OR REPLACE FUNCTION trg_collections_cover_update_cleanup() RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    IF OLD.cover_collection_image_id IS DISTINCT FROM NEW.cover_collection_image_id
       AND OLD.cover_collection_image_id IS NOT NULL THEN
        PERFORM try_delete_collection_image_if_orphan(OLD.cover_collection_image_id);
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER collections_cover_update_cleanup
    AFTER UPDATE OF cover_collection_image_id ON collections
    FOR EACH ROW
    EXECUTE FUNCTION trg_collections_cover_update_cleanup();

CREATE OR REPLACE FUNCTION trg_collections_before_delete_cover_cleanup() RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    PERFORM try_delete_collection_image_if_orphan_for_deleted_collection(
        OLD.cover_collection_image_id,
        OLD.collection_id
    );
    RETURN OLD;
END;
$$;

CREATE TRIGGER collections_before_delete_cover_cleanup
    BEFORE DELETE ON collections
    FOR EACH ROW
    EXECUTE FUNCTION trg_collections_before_delete_cover_cleanup();
