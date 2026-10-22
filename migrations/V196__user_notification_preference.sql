-- Per-user notification preference in users.metadata.
-- Missing key means notifications stay on (new rows and rows written before this migration).

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb;

UPDATE users
SET metadata = metadata || '{"notifications_enabled": true}'::jsonb
WHERE NOT (metadata ? 'notifications_enabled');

ALTER TABLE users
    ALTER COLUMN metadata SET DEFAULT '{"notifications_enabled": true}'::jsonb;

CREATE OR REPLACE FUNCTION user_accepts_notifications(p_user_id INTEGER)
RETURNS boolean
LANGUAGE sql
STABLE
AS $$
    SELECT COALESCE(
        (
            SELECT (metadata->>'notifications_enabled')::boolean
            FROM users
            WHERE userid = p_user_id
        ),
        true
    );
$$;

-- Dictionary edits, new definitions, and thread comments.
CREATE OR REPLACE FUNCTION notify_valsi_subscribers(
    p_valsi_id INTEGER,
    p_event_type TEXT,
    p_message TEXT,
    p_link TEXT,
    p_actor_id INTEGER,
    p_definition_id INTEGER DEFAULT NULL
) RETURNS void AS $$
BEGIN
    INSERT INTO user_notifications (
        user_id, notification_type, message, link,
        valsi_id, actor_id, definition_id, created_at
    )
    SELECT DISTINCT
        vs.user_id, p_event_type, p_message, p_link,
        p_valsi_id, p_actor_id, p_definition_id, CURRENT_TIMESTAMP
    FROM valsi_subscriptions vs
    JOIN users u ON vs.user_id = u.userid
    WHERE vs.valsi_id = p_valsi_id
      AND NOT vs.unsubscribed
      AND vs.user_id != p_actor_id
      AND u.email IS NOT NULL
      AND u.role::text <> 'blocked'
      AND user_accepts_notifications(u.userid);
END;
$$ LANGUAGE plpgsql;

-- Private-message unread badges. The message itself is still stored.
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

        UPDATE thread_participants tp
        SET unread_count = tp.unread_count + 1
        WHERE tp.thread_id = NEW.thread_id
          AND tp.user_id != NEW.sender_id
          AND tp.is_active = TRUE
          AND user_accepts_notifications(tp.user_id);
    ELSIF TG_OP = 'UPDATE' AND OLD.is_deleted = FALSE AND NEW.is_deleted = TRUE THEN
        UPDATE message_threads
        SET message_count = GREATEST(message_count - 1, 0)
        WHERE thread_id = NEW.thread_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
