-- Update the message stats trigger so the thread preview shows decoded text
-- when the encrypted content is base64-encoded UTF-8 (the current opaque payload mode).
CREATE OR REPLACE FUNCTION update_thread_message_stats()
RETURNS TRIGGER AS $$
DECLARE
    preview_text TEXT;
BEGIN
    IF TG_OP = 'INSERT' AND NEW.is_deleted = FALSE THEN
        BEGIN
            preview_text := LEFT(convert_from(decode(NEW.encrypted_content, 'base64'), 'UTF8'), 100);
        EXCEPTION WHEN OTHERS THEN
            preview_text := LEFT(NEW.encrypted_content, 100);
        END;

        UPDATE message_threads
        SET message_count = message_count + 1,
            last_message_at = NEW.created_at,
            last_message_preview = preview_text,
            updated_at = CURRENT_TIMESTAMP
        WHERE thread_id = NEW.thread_id;

        UPDATE thread_participants
        SET unread_count = unread_count + 1
        WHERE thread_id = NEW.thread_id
        AND user_id != NEW.sender_id
        AND is_active = TRUE;
    ELSIF TG_OP = 'UPDATE' AND OLD.is_deleted = FALSE AND NEW.is_deleted = TRUE THEN
        UPDATE message_threads
        SET message_count = GREATEST(message_count - 1, 0)
        WHERE thread_id = NEW.thread_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
