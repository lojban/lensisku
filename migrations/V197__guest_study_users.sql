-- Guest study accounts: role='guest' dummy users that hold normal flashcard progress.
-- Not shown in user directories; deleted after claim-on-login or by TTL cleanup.
-- Role is free-text (V78); no enum migration required.

CREATE INDEX IF NOT EXISTS idx_users_guest_role_created
    ON users (created_at)
    WHERE LOWER(role::text) = 'guest';
