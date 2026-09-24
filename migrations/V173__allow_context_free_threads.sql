-- /comments/new-thread submits a comment without any thread context IDs.
-- Such a thread is a free wave. The existing check requires a context,
-- so it rejects the INSERT for this path.
ALTER TABLE public.threads DROP CONSTRAINT threads_context_check;
