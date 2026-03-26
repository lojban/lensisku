-- Imported jbotcan pseudo-users were created as username jbotcan_<name>_<number>
-- (see jbotcan-import). Rename to <name>_<number>@jbotcan for display and email-style identity.
--
-- QA: users with username matching ^jbotcan_ should show as *@jbotcan; comments/views
-- that join users.username should reflect the new values after migration.

UPDATE users
SET
    username = regexp_replace(username, '^jbotcan_', '') || '@jbotcan',
    email = regexp_replace(username, '^jbotcan_', '') || '@jbotcan'
WHERE username ~ '^jbotcan_'
  AND length(username) > length('jbotcan_');
