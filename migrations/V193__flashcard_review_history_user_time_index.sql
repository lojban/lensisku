-- Speeds up COUNT(*) / full history scans by user for optimal-retention refresh
-- and due-card ReviewHistory aggregation. Existing (user_id, flashcard_id, review_time)
-- is suboptimal for ORDER BY review_time without a flashcard_id predicate.
CREATE INDEX IF NOT EXISTS idx_flashcard_review_history_user_time
    ON flashcard_review_history (user_id, review_time);
