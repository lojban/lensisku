ALTER TABLE sent_emails
    ADD COLUMN initiated_by_user_id INTEGER;

CREATE INDEX idx_sent_emails_initiated_by_user_id
    ON sent_emails (initiated_by_user_id)
    WHERE initiated_by_user_id IS NOT NULL;
