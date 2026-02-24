-- Fix check_level_prerequisites: levels with prerequisites must be locked until
-- the user has completed all prerequisite levels. The previous version used
-- INNER JOIN so when the user had no progress on a prerequisite, the join
-- returned zero rows and COALESCE(bool_and(...), true) incorrectly returned true (unlocked).

CREATE OR REPLACE FUNCTION check_level_prerequisites(
    p_user_id INTEGER,
    p_level_id INTEGER
) RETURNS BOOLEAN AS $$
DECLARE
    v_has_prereqs BOOLEAN;
    v_all_completed BOOLEAN;
BEGIN
    -- Level with no prerequisites is always unlocked
    SELECT EXISTS(SELECT 1 FROM level_prerequisites WHERE level_id = p_level_id)
    INTO v_has_prereqs;
    IF NOT v_has_prereqs THEN
        RETURN true;
    END IF;

    -- Level has prerequisites: unlocked only if user has completed every prerequisite
    -- LEFT JOIN so we get one row per prerequisite; missing progress -> completed_at IS NULL -> false
    SELECT COALESCE(bool_and(ulp.completed_at IS NOT NULL), false)
    INTO v_all_completed
    FROM level_prerequisites lp
    LEFT JOIN user_level_progress ulp
        ON ulp.level_id = lp.prerequisite_id
        AND ulp.user_id = p_user_id
    WHERE lp.level_id = p_level_id;

    RETURN v_all_completed;
END;
$$ LANGUAGE plpgsql;
