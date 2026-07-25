-- Update get_user_message_threads to expose is_active and max_participants columns
-- for compatibility with the ThreadResponse struct.
DROP FUNCTION IF EXISTS get_user_message_threads(INTEGER);

CREATE OR REPLACE FUNCTION get_user_message_threads(p_user_id INTEGER)
RETURNS TABLE (
    thread_id BIGINT,
    thread_name VARCHAR(255),
    thread_type VARCHAR(20),
    created_by INTEGER,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    is_active BOOLEAN,
    max_participants INTEGER,
    last_message_at TIMESTAMPTZ,
    last_message_preview TEXT,
    message_count BIGINT,
    unread_count BIGINT,
    participant_count BIGINT,
    is_admin BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        mt.thread_id,
        mt.thread_name,
        mt.thread_type,
        mt.created_by,
        mt.created_at,
        mt.updated_at,
        mt.is_active,
        mt.max_participants,
        mt.last_message_at,
        mt.last_message_preview,
        mt.message_count,
        COALESCE(tp.unread_count, 0) as unread_count,
        (SELECT COUNT(*) FROM thread_participants tp2 WHERE tp2.thread_id = mt.thread_id AND tp2.is_active = TRUE) as participant_count,
        (tp.role = 'admin') as is_admin
    FROM message_threads mt
    JOIN thread_participants tp ON mt.thread_id = tp.thread_id
    WHERE tp.user_id = p_user_id
    AND tp.is_active = TRUE
    AND mt.is_active = TRUE
    ORDER BY mt.last_message_at DESC NULLS LAST, mt.created_at DESC;
END;
$$ LANGUAGE plpgsql;
