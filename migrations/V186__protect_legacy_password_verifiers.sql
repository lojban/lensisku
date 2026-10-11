-- Preserve existing passwords while replacing fast MD5 verifiers with salted,
-- work-factored bcrypt envelopes. Successful login upgrades to direct bcrypt.
-- This operation costs one bcrypt hash per legacy account and can take time.
UPDATE users
SET password = 'legacy-bcrypt:' || crypt(lower(password), gen_salt('bf', 12))
WHERE password ~ '^[0-9a-fA-F]{32}$';
