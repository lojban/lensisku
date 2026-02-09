-- Add definition_id to user_notifications for edit notifications (which definition was updated)
ALTER TABLE user_notifications
    ADD COLUMN definition_id INTEGER REFERENCES definitions(definitionid);

CREATE INDEX IF NOT EXISTS idx_user_notifications_definition_id
    ON user_notifications(definition_id);

-- Extend notify_valsi_subscribers to accept optional definition_id for edit events
CREATE OR REPLACE FUNCTION notify_valsi_subscribers(
    p_valsi_id INTEGER,
    p_event_type TEXT,
    p_message TEXT,
    p_link TEXT,
    p_actor_id INTEGER,
    p_definition_id INTEGER DEFAULT NULL
) RETURNS void AS $$
DECLARE
    v_user_id INTEGER;
    v_email TEXT;
    v_username TEXT;
BEGIN
    FOR v_user_id, v_email, v_username IN
        SELECT DISTINCT
            vs.user_id,
            u.email,
            u.username
        FROM valsi_subscriptions vs
        JOIN users u ON vs.user_id = u.userid
        WHERE vs.valsi_id = p_valsi_id
        AND NOT vs.unsubscribed
        AND vs.user_id != p_actor_id
        AND u.email IS NOT NULL
    LOOP
        INSERT INTO user_notifications (
            user_id,
            notification_type,
            message,
            link,
            valsi_id,
            actor_id,
            definition_id,
            created_at
        ) VALUES (
            v_user_id,
            p_event_type,
            p_message,
            p_link,
            p_valsi_id,
            p_actor_id,
            p_definition_id,
            CURRENT_TIMESTAMP
        );
    END LOOP;
END;
$$ LANGUAGE plpgsql;
