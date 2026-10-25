-- Level progress counted a review as correct only when rating >= 3. Since the subtractive
-- rating policy (commit 383e3b79), a correct quiz / word-bank / listen answer submitted as 3
-- is stored as 2, so on courses made of those card types the success rate stayed 0 and no
-- level could ever be completed: learners were stuck on level 1 with the rest locked.
-- Count pass = rating >= 2 (FSRS: only 1 "Again" is a lapse), recount, and backfill.

CREATE OR REPLACE FUNCTION update_level_progress()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO user_level_progress AS ulp (
        user_id, level_id,
        cards_completed, correct_answers, total_answers,
        last_activity_at
    )
    SELECT
        NEW.user_id,
        fli.level_id,
        COUNT(DISTINCT fr.flashcard_id),
        -- Pass = rating >= 2 (FSRS: only 1 "Again" is a lapse). Since the subtractive rating
        -- policy (V-383e3b79) stores a correct quiz/word-bank answer as 2, counting >= 3 made
        -- the success rate 0 on those courses, so no level could ever be completed.
        SUM(CASE WHEN fr.rating >= 2 THEN 1 ELSE 0 END),
        COUNT(*),
        CURRENT_TIMESTAMP
    FROM flashcard_review_history fr
    JOIN flashcard_level_items fli ON fr.flashcard_id = fli.flashcard_id
    WHERE fr.user_id = NEW.user_id
    AND fli.level_id IN (
        SELECT level_id
        FROM flashcard_level_items
        WHERE flashcard_id = NEW.flashcard_id
    )
    GROUP BY fli.level_id
    ON CONFLICT (user_id, level_id) DO UPDATE
    SET
        cards_completed = EXCLUDED.cards_completed,
        correct_answers = EXCLUDED.correct_answers,
        total_answers = EXCLUDED.total_answers,
        last_activity_at = EXCLUDED.last_activity_at;

    -- Judge completion after the upsert so check_level_completion sees the new counts
    -- (covers both the first insert and the update branch).
    UPDATE user_level_progress
    SET completed_at = CURRENT_TIMESTAMP
    WHERE user_id = NEW.user_id
      AND completed_at IS NULL
      AND level_id IN (SELECT level_id FROM flashcard_level_items WHERE flashcard_id = NEW.flashcard_id)
      AND check_level_completion(user_id, level_id);

    UPDATE user_level_progress
    SET unlocked_at = CASE
        WHEN check_level_prerequisites(user_id, level_id)
            AND unlocked_at IS NULL
        THEN CURRENT_TIMESTAMP
        ELSE unlocked_at
    END
    WHERE user_id = NEW.user_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Recount correct answers with the pass threshold, then backfill learners who now meet
-- the requirements but were never marked complete.
UPDATE user_level_progress ulp
SET correct_answers = sub.correct
FROM (
    SELECT fr.user_id, fli.level_id, SUM(CASE WHEN fr.rating >= 2 THEN 1 ELSE 0 END)::int AS correct
    FROM flashcard_review_history fr
    JOIN flashcard_level_items fli ON fli.flashcard_id = fr.flashcard_id
    GROUP BY fr.user_id, fli.level_id
) sub
WHERE sub.user_id = ulp.user_id AND sub.level_id = ulp.level_id
  AND sub.correct <> ulp.correct_answers;

UPDATE user_level_progress
SET completed_at = CURRENT_TIMESTAMP
WHERE completed_at IS NULL
  AND check_level_completion(user_id, level_id);

UPDATE user_level_progress
SET unlocked_at = CURRENT_TIMESTAMP
WHERE unlocked_at IS NULL
  AND check_level_prerequisites(user_id, level_id);
