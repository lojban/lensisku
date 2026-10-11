-- All consumers resolve privacy from the stored thread, never request context.
CREATE FUNCTION can_read_comment_thread(thread_id integer, viewer_id integer)
RETURNS boolean LANGUAGE sql STABLE AS $$
    SELECT EXISTS (
        SELECT 1 FROM threads t WHERE t.threadid = thread_id
        AND (t.collection_id IS NULL OR EXISTS (
            SELECT 1 FROM collections c WHERE c.collection_id = t.collection_id
            AND (c.is_public OR c.user_id = viewer_id)
        ))
    );
$$;
CREATE FUNCTION can_read_comment(comment_id integer, viewer_id integer)
RETURNS boolean LANGUAGE sql STABLE AS $$
    SELECT EXISTS (SELECT 1 FROM comments c WHERE c.commentid = comment_id
                   AND can_read_comment_thread(c.threadid, viewer_id));
$$;
