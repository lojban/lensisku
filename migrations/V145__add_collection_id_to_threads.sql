-- Add collection_id context to threads so collections can have discussion threads
-- Mirrors the pattern used for valsi / definition / definition_link / target_user contexts.

ALTER TABLE public.threads
ADD COLUMN collection_id INTEGER REFERENCES public.collections(collection_id) ON DELETE CASCADE DEFAULT NULL;

CREATE INDEX idx_threads_collection_id ON public.threads (collection_id) WHERE collection_id IS NOT NULL;

-- Recreate context check to include collection_id as a valid thread context
ALTER TABLE public.threads DROP CONSTRAINT IF EXISTS threads_context_check;
ALTER TABLE public.threads
ADD CONSTRAINT threads_context_check
CHECK (
    valsiid IS NOT NULL OR
    natlangwordid IS NOT NULL OR
    definitionid IS NOT NULL OR
    definition_link_id IS NOT NULL OR
    target_user_id IS NOT NULL OR
    collection_id IS NOT NULL
);

-- Expose collection_id through the convenient comments view used by listings
CREATE OR REPLACE VIEW convenientcomments AS
SELECT
    c.commentid,
    c.threadid,
    c.parentid,
    c.userid,
    u.username,
    u.realname,
    c.time,
    c.subject,
    c.content,
    c.commentnum,
    cc.total_reactions,
    cc.total_replies,
    t.valsiid,
    t.definitionid,
    t.definition_link_id,
    t.collection_id
FROM
    comments c
    JOIN users u ON c.userid = u.userid
    JOIN threads t ON c.threadid = t.threadid
    LEFT JOIN comment_counters cc ON c.commentid = cc.comment_id;
