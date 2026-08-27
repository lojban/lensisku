-- Imported mw.lojban.org editors must not be treated as lensisku accounts.
-- Give them *@mw.lojban.org identities (same idea as *@jbotcan).
--
-- QA: recent changes / wiki history authors should show as name@mw.lojban.org
-- and open https://mw.lojban.org/papri/User:Name — not /user/Name.

INSERT INTO users (username, email, password, created_at, role, email_confirmed, votesize)
SELECT DISTINCT
    u.username || '@mw.lojban.org',
    u.username || '@mw.lojban.org',
    'DISABLED',
    NOW(),
    'blocked',
    false,
    0
FROM definition_versions dv
JOIN users u ON u.userid = dv.user_id
WHERE dv.mw_revid IS NOT NULL
  AND u.username IS DISTINCT FROM 'officialdata'
  AND u.username NOT LIKE '%@mw.lojban.org'
  AND char_length(u.username) + char_length('@mw.lojban.org') <= 64
  AND NOT EXISTS (
      SELECT 1 FROM users stub
      WHERE stub.username = u.username || '@mw.lojban.org'
  );

UPDATE definition_versions dv
SET user_id = stub.userid
FROM users u
JOIN users stub ON stub.username = u.username || '@mw.lojban.org'
WHERE dv.mw_revid IS NOT NULL
  AND dv.user_id = u.userid
  AND u.username IS DISTINCT FROM 'officialdata'
  AND u.username NOT LIKE '%@mw.lojban.org';
