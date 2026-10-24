-- Word-bank study directions: assemble the answer by tapping chips (fill-in grading).
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'wordbank';
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'wordbank_reverse';
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'wordbank_both';
