-- Information cards (`just_information`) are shown once: `new` until acknowledged, then
-- retired (graduated, next review far in the future — NULL would count as due).
UPDATE user_flashcard_progress p
SET status = 'graduated', next_review_at = CURRENT_TIMESTAMP + INTERVAL '100 years'
FROM flashcards f
WHERE f.id = p.flashcard_id AND f.direction = 'just_information' AND NOT p.archived
  AND EXISTS (SELECT 1 FROM flashcard_review_history h
              WHERE h.user_id = p.user_id AND h.flashcard_id = p.flashcard_id);

UPDATE user_flashcard_progress p
SET status = 'new', next_review_at = CURRENT_TIMESTAMP
FROM flashcards f
WHERE f.id = p.flashcard_id AND f.direction = 'just_information' AND NOT p.archived
  AND NOT EXISTS (SELECT 1 FROM flashcard_review_history h
                  WHERE h.user_id = p.user_id AND h.flashcard_id = p.flashcard_id);
