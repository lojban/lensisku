-- Add quiz-with-image-options directions: question is text, options are images (item_id:side).
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'quiz_image_direct';
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'quiz_image_reverse';
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'quiz_image_both';
