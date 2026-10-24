-- Listen-tap study directions: hear audio, assemble answer from chips (Duo listenTap).
-- Graded like wordbank but stimulus is audio-only (front sound → assemble front text).
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'listen_wordbank';
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'listen_wordbank_reverse';
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'listen_wordbank_both';
