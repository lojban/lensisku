-- unlocked_at was set while check_level_prerequisites incorrectly returned true
-- for levels with unmet prerequisites (fixed in V133). Clear stale stamps so
-- progress.is_unlocked cannot disagree with the live prerequisites check.
UPDATE user_level_progress ulp
SET unlocked_at = NULL
WHERE ulp.unlocked_at IS NOT NULL
  AND NOT check_level_prerequisites(ulp.user_id, ulp.level_id);
