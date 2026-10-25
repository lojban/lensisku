-- Course setting: automatically apply the challenge mix (quiz / wordbank / listen
-- directions) to level cards whenever cards are added to a level.
ALTER TABLE public.collections
    ADD COLUMN IF NOT EXISTS auto_challenge_mix boolean NOT NULL DEFAULT false;
