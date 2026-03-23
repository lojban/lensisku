-- Per-user assistant chat threads (rich message JSON matches frontend AssistantChat.vue shape)
CREATE TABLE assistant_chats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    title TEXT NOT NULL DEFAULT '',
    messages JSONB NOT NULL DEFAULT '[]'::jsonb,
    primary_model_id TEXT,
    scroll_top DOUBLE PRECISION NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_assistant_chats_user_updated ON assistant_chats (user_id, updated_at DESC);
