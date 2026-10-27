-- Dialogue ("story") cards: a script of speaker lines (front) plus questions, see src/dialogue.
-- Never Auto, never answer material; they count toward level strength and completion.
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'dialogue';

-- Stable speaker → Kokoro voice map, saved on first audio generation (e.g. {"Marian":"Victoria"}).
ALTER TABLE collection_items ADD COLUMN IF NOT EXISTS dialogue_voices JSONB;

-- One TTS clip per script line. `text_hash` = sha256(voice|speed|text): unchanged lines are reused.
CREATE TABLE IF NOT EXISTS dialogue_line_sounds (
    item_id INTEGER NOT NULL REFERENCES collection_items(item_id) ON DELETE CASCADE,
    line_index INTEGER NOT NULL,
    text_hash TEXT NOT NULL,
    voice TEXT NOT NULL,
    sound_data BYTEA NOT NULL,
    mime_type TEXT NOT NULL DEFAULT 'audio/ogg',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (item_id, line_index)
);

-- Each question of a dialogue is a separate answer with its own score. One attempt (one pass
-- through the dialogue) commits a single FSRS review for the card: the lowest question rating.
CREATE TABLE IF NOT EXISTS dialogue_question_answers (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    flashcard_id INTEGER NOT NULL REFERENCES flashcards(id) ON DELETE CASCADE,
    attempt_id UUID NOT NULL,
    question_index INTEGER NOT NULL,
    question_kind TEXT NOT NULL,
    answer TEXT NOT NULL,
    correct BOOLEAN NOT NULL,
    hint_used BOOLEAN NOT NULL DEFAULT false,
    rating INTEGER NOT NULL,
    answered_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (attempt_id, question_index)
);
CREATE INDEX IF NOT EXISTS idx_dialogue_answers_user_card
    ON dialogue_question_answers (user_id, flashcard_id, answered_at);
