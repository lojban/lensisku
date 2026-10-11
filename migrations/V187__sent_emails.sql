CREATE TABLE sent_emails (
    id BIGSERIAL PRIMARY KEY,
    to_address TEXT NOT NULL,
    from_address TEXT NOT NULL,
    subject TEXT NOT NULL,
    text_body TEXT NOT NULL,
    html_body TEXT NOT NULL,
    params JSONB NOT NULL DEFAULT '{}'::jsonb,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_sent_emails_sent_at ON sent_emails (sent_at DESC);
CREATE INDEX idx_sent_emails_to_address ON sent_emails (to_address);
