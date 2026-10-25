-- Level completion was judged by check_level_completion() inside ON CONFLICT DO UPDATE,
-- which reads the *pre-update* user_level_progress row. The review that crossed the
-- min_cards / min_success_rate threshold therefore never completed the level; completion
-- only happened on a later review. When every card in the level had been reviewed and none
-- was due, no later review came and the learner stayed stuck with the next level locked.
-- Judge completion after the upsert instead, and backfill.
-- Also match every level containing the card (the old `= (subquery)` errored when a card
-- sat in more than one level).

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
        SUM(CASE WHEN fr.rating >= 3 THEN 1 ELSE 0 END),
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

-- Backfill learners who already met the requirements but were never marked complete.
UPDATE user_level_progress
SET completed_at = CURRENT_TIMESTAMP
WHERE completed_at IS NULL
  AND check_level_completion(user_id, level_id);

UPDATE user_level_progress
SET unlocked_at = CURRENT_TIMESTAMP
WHERE unlocked_at IS NULL
  AND check_level_prerequisites(user_id, level_id);
