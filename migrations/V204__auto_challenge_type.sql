-- "Auto" challenge type: the server picks the presented type (quiz / wordbank / listen /
-- reverse / typing) for every review of an `auto` card.
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'auto';

-- Which concrete type was presented for a review (NULL for fixed cards and old rows).
-- Nullable, no default: metadata-only on large tables.
ALTER TABLE public.flashcard_review_history
    ADD COLUMN IF NOT EXISTS presented_type text;

-- Ladder stage of an auto card side (0 = recognition quiz ... 4 = typing).
ALTER TABLE public.user_flashcard_progress
    ADD COLUMN IF NOT EXISTS auto_stage smallint NOT NULL DEFAULT 0;

-- Course challenge mode replaces the auto_challenge_mix toggle:
--   off   = leave card types alone
--   fixed = re-apply the positional challenge mix when cards are added (old toggle = true)
--   auto  = new / mixed cards become `auto`
ALTER TABLE public.collections
    ADD COLUMN IF NOT EXISTS challenge_mode text NOT NULL DEFAULT 'off'
    CHECK (challenge_mode IN ('off', 'fixed', 'auto'));
UPDATE public.collections SET challenge_mode = 'fixed'
WHERE auto_challenge_mix AND challenge_mode = 'off';
