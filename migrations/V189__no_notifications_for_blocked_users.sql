-- Do not create notifications for users whose role is 'blocked'.
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
      AND u.role::text <> 'blocked';
END;
$$ LANGUAGE plpgsql;
