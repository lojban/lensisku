-- Personal News feed: thread participation lookup (comments by a user in a thread).
CREATE INDEX IF NOT EXISTS idx_comments_userid_threadid ON comments (userid, threadid);

-- Personal News feed: a user's active watches by trigger.
CREATE INDEX IF NOT EXISTS idx_valsi_subs_user_active
    ON valsi_subscriptions (user_id, trigger_type) INCLUDE (valsi_id)
    WHERE NOT unsubscribed;

ANALYZE comments;
ANALYZE valsi_subscriptions;
