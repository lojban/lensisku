-- Private Messaging System Migration
-- This migration creates tables for end-to-end encrypted private messaging
-- Uses private_messages table name to avoid conflicts with existing messages table

-- Message threads table - stores thread metadata
CREATE TABLE IF NOT EXISTS message_threads (
    thread_id BIGSERIAL PRIMARY KEY,
    thread_name VARCHAR(255),
    thread_type VARCHAR(20) NOT NULL CHECK (thread_type IN ('direct', 'group')),
    created_by INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    max_participants INTEGER NOT NULL DEFAULT 100,
    last_message_at TIMESTAMPTZ,
    last_message_preview TEXT,
    message_count BIGINT NOT NULL DEFAULT 0
);

-- Thread participants table - manages who can access each thread
CREATE TABLE IF NOT EXISTS thread_participants (
    participant_id BIGSERIAL PRIMARY KEY,
    thread_id BIGINT NOT NULL REFERENCES message_threads(thread_id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL DEFAULT 'member' CHECK (role IN ('admin', 'member')),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    left_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_read_at TIMESTAMPTZ,
    unread_count BIGINT NOT NULL DEFAULT 0,
    UNIQUE(thread_id, user_id)
);

-- Private messages table - stores encrypted messages (renamed to avoid conflicts)
CREATE TABLE IF NOT EXISTS private_messages (
    message_id BIGSERIAL PRIMARY KEY,
    thread_id BIGINT NOT NULL REFERENCES message_threads(thread_id) ON DELETE CASCADE,
    sender_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    message_type VARCHAR(20) NOT NULL DEFAULT 'text' CHECK (message_type IN ('text', 'image', 'file', 'system')),
    encrypted_content TEXT NOT NULL,
    content_nonce BYTEA NOT NULL,
    sender_key_signature BYTEA,
    reply_to_message_id BIGINT REFERENCES private_messages(message_id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at TIMESTAMPTZ,
    deleted_by INTEGER REFERENCES users(userid) ON DELETE SET NULL,
    edit_count INTEGER NOT NULL DEFAULT 0,
    last_edited_at TIMESTAMPTZ
);

-- Message encryption keys table - manages per-thread encryption keys
CREATE TABLE IF NOT EXISTS message_encryption_keys (
    key_id BIGSERIAL PRIMARY KEY,
    thread_id BIGINT NOT NULL REFERENCES message_threads(thread_id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    encrypted_key TEXT NOT NULL,
    key_algorithm VARCHAR(50) NOT NULL DEFAULT 'kyber1024',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    key_version INTEGER NOT NULL DEFAULT 1,
    UNIQUE(thread_id, user_id, key_version)
);

-- User message blocks table - manages blocking relationships
CREATE TABLE IF NOT EXISTS user_message_blocks (
    block_id BIGSERIAL PRIMARY KEY,
    blocker_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    blocked_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    blocked_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reason TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    UNIQUE(blocker_id, blocked_id),
    CHECK(blocker_id != blocked_id)
);

-- Message notifications table - queues in-app notifications
CREATE TABLE IF NOT EXISTS message_notifications (
    notification_id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    thread_id BIGINT NOT NULL REFERENCES message_threads(thread_id) ON DELETE CASCADE,
    message_id BIGINT NOT NULL REFERENCES private_messages(message_id) ON DELETE CASCADE,
    sender_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    notification_type VARCHAR(20) NOT NULL DEFAULT 'new_message' CHECK (notification_type IN ('new_message', 'thread_added', 'thread_removed')),
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    read_at TIMESTAMPTZ
);

-- WebRTC signaling table - temporary storage for P2P connection setup
CREATE TABLE IF NOT EXISTS webrtc_signaling (
    signal_id BIGSERIAL PRIMARY KEY,
    from_user_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    to_user_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    signal_type VARCHAR(20) NOT NULL CHECK (signal_type IN ('offer', 'answer', 'ice-candidate')),
    signal_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (CURRENT_TIMESTAMP + INTERVAL '5 minutes'),
    is_processed BOOLEAN NOT NULL DEFAULT FALSE
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_message_threads_created_by ON message_threads(created_by);
CREATE INDEX IF NOT EXISTS idx_message_threads_created_at ON message_threads(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_message_threads_last_message_at ON message_threads(last_message_at DESC);
CREATE INDEX IF NOT EXISTS idx_message_threads_active ON message_threads(is_active) WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_thread_participants_thread_id ON thread_participants(thread_id);
CREATE INDEX IF NOT EXISTS idx_thread_participants_user_id ON thread_participants(user_id);
CREATE INDEX IF NOT EXISTS idx_thread_participants_active ON thread_participants(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_thread_participants_unread ON thread_participants(user_id, unread_count DESC) WHERE unread_count > 0;

CREATE INDEX IF NOT EXISTS idx_private_messages_thread_id ON private_messages(thread_id);
CREATE INDEX IF NOT EXISTS idx_private_messages_sender_id ON private_messages(sender_id);
CREATE INDEX IF NOT EXISTS idx_private_messages_created_at ON private_messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_private_messages_thread_created ON private_messages(thread_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_private_messages_active ON private_messages(thread_id, is_deleted) WHERE is_deleted = FALSE;

CREATE INDEX IF NOT EXISTS idx_message_encryption_keys_thread_user ON message_encryption_keys(thread_id, user_id);
CREATE INDEX IF NOT EXISTS idx_message_encryption_keys_active ON message_encryption_keys(is_active) WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_user_message_blocks_blocker ON user_message_blocks(blocker_id);
CREATE INDEX IF NOT EXISTS idx_user_message_blocks_blocked ON user_message_blocks(blocked_id);
CREATE INDEX IF NOT EXISTS idx_user_message_blocks_active ON user_message_blocks(is_active) WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_message_notifications_user ON message_notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_message_notifications_unread ON message_notifications(user_id, is_read) WHERE is_read = FALSE;
CREATE INDEX IF NOT EXISTS idx_message_notifications_created ON message_notifications(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_webrtc_signaling_to_user ON webrtc_signaling(to_user_id, is_processed);
CREATE INDEX IF NOT EXISTS idx_webrtc_signaling_expires ON webrtc_signaling(expires_at);

-- Create triggers for automatic timestamp updates
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_message_threads_updated_at
    BEFORE UPDATE ON message_threads
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_private_messages_updated_at
    BEFORE UPDATE ON private_messages
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create trigger to update thread message count and last message info
CREATE OR REPLACE FUNCTION update_thread_message_stats()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' AND NEW.is_deleted = FALSE THEN
        UPDATE message_threads 
        SET 
            message_count = message_count + 1,
            last_message_at = NEW.created_at,
            last_message_preview = LEFT(NEW.encrypted_content, 100),
            updated_at = CURRENT_TIMESTAMP
        WHERE thread_id = NEW.thread_id;
        
        -- Update unread count for all participants except sender
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

CREATE TRIGGER maintain_thread_message_stats
    AFTER INSERT OR UPDATE ON private_messages
    FOR EACH ROW
    EXECUTE FUNCTION update_thread_message_stats();

-- Create trigger to mark messages as read for participants
CREATE OR REPLACE FUNCTION mark_messages_as_read()
RETURNS TRIGGER AS $$
BEGIN
    -- Mark all messages in thread as read for this user
    UPDATE thread_participants 
    SET 
        last_read_at = CURRENT_TIMESTAMP,
        unread_count = 0
    WHERE thread_id = NEW.thread_id 
    AND user_id = NEW.user_id 
    AND is_active = TRUE;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER mark_thread_messages_read
    AFTER UPDATE OF last_read_at ON thread_participants
    FOR EACH ROW
    WHEN (NEW.last_read_at IS DISTINCT FROM OLD.last_read_at)
    EXECUTE FUNCTION mark_messages_as_read();

-- Create function to get user's threads with unread counts
CREATE OR REPLACE FUNCTION get_user_message_threads(p_user_id INTEGER)
RETURNS TABLE (
    thread_id BIGINT,
    thread_name VARCHAR(255),
    thread_type VARCHAR(20),
    created_by INTEGER,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
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

-- Create function to check if user is blocked
CREATE OR REPLACE FUNCTION is_user_blocked(p_blocker_id INTEGER, p_blocked_id INTEGER)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM user_message_blocks 
        WHERE blocker_id = p_blocker_id 
        AND blocked_id = p_blocked_id 
        AND is_active = TRUE
    );
END;
$$ LANGUAGE plpgsql;

-- Create function to clean up expired WebRTC signals
CREATE OR REPLACE FUNCTION cleanup_expired_webrtc_signals()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM webrtc_signaling 
    WHERE expires_at < CURRENT_TIMESTAMP OR is_processed = TRUE;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;
