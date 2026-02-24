-- Remove legacy "like" functionality; reactions cover this use case.
DROP TABLE IF EXISTS comment_likes;

-- comment_activity_counters had total_likes; drop it (total_reactions is used).
ALTER TABLE comment_activity_counters DROP COLUMN IF EXISTS total_likes;
