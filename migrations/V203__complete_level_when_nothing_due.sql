-- "Nothing due" in a level must not dead-end the learner. When every card of a level has
-- been reviewed by the learner at least once (no never-seen cards: those count as due) and
-- none of the learner's card sides in that level is due now, the level is marked complete
-- so check_level_prerequisites() unlocks the next level. Success-rate numbers are left as
-- they are; `completed_by_schedule` records that completion came from the schedule rather
-- than from meeting min_cards / min_success_rate.

ALTER TABLE public.user_level_progress
    ADD COLUMN IF NOT EXISTS completed_by_schedule boolean NOT NULL DEFAULT false;

CREATE OR REPLACE FUNCTION complete_level_when_nothing_due(
    p_user_id INTEGER,
    p_level_id INTEGER
) RETURNS BOOLEAN AS $$
DECLARE
    v_cards INTEGER;
    v_seen INTEGER;
    v_due INTEGER;
BEGIN
    IF EXISTS (
        SELECT 1 FROM user_level_progress
        WHERE user_id = p_user_id AND level_id = p_level_id AND completed_at IS NOT NULL
    ) THEN
        RETURN true;
    END IF;
    -- A locked level cannot be completed this way.
    IF NOT check_level_prerequisites(p_user_id, p_level_id) THEN
        RETURN false;
    END IF;

    SELECT COUNT(*) INTO v_cards FROM flashcard_level_items WHERE level_id = p_level_id;
    IF v_cards = 0 THEN
        RETURN false;
    END IF;

    SELECT COUNT(DISTINCT fli.flashcard_id) INTO v_seen
    FROM flashcard_level_items fli
    WHERE fli.level_id = p_level_id
      AND EXISTS (
          SELECT 1 FROM flashcard_review_history fr
          WHERE fr.user_id = p_user_id AND fr.flashcard_id = fli.flashcard_id
      );
    IF v_seen < v_cards THEN
        RETURN false;
    END IF;

    SELECT COUNT(*) INTO v_due
    FROM flashcard_level_items fli
    JOIN user_flashcard_progress p
      ON p.flashcard_id = fli.flashcard_id AND p.user_id = p_user_id
    WHERE fli.level_id = p_level_id
      AND NOT p.archived
      AND p.next_review_at <= CURRENT_TIMESTAMP;
    IF v_due > 0 THEN
        RETURN false;
    END IF;

    INSERT INTO user_level_progress AS ulp
        (user_id, level_id, cards_completed, correct_answers, total_answers,
         last_activity_at, completed_at, completed_by_schedule)
    VALUES (p_user_id, p_level_id, 0, 0, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, true)
    ON CONFLICT (user_id, level_id) DO UPDATE
    SET completed_at = COALESCE(ulp.completed_at, CURRENT_TIMESTAMP),
        completed_by_schedule = ulp.completed_at IS NULL OR ulp.completed_by_schedule;

    UPDATE user_level_progress
    SET unlocked_at = CURRENT_TIMESTAMP
    WHERE user_id = p_user_id
      AND unlocked_at IS NULL
      AND check_level_prerequisites(user_id, level_id);

    RETURN true;
END;
$$ LANGUAGE plpgsql;
