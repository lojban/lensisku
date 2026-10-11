CREATE TABLE oauth_login_transactions (
    nonce TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    pkce_verifier TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX oauth_login_transactions_expiry ON oauth_login_transactions(expires_at);
