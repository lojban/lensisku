-- Full lensisku schema (pg_dump -s of a migrated dev DB) plus the languages and
-- valsitypes reference rows. Used by the collections/flashcards DB tests in CI.
-- Regenerate after adding migrations: see the db-tests job in .github/workflows/rust.yml.
--
--
-- PostgreSQL database dump
--


-- Dumped from database version 17.11 (Debian 17.11-0+deb13u1)
-- Dumped by pg_dump version 17.11 (Debian 17.11-0+deb13u1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: pg_trgm; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;


--
-- Name: EXTENSION pg_trgm; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';


--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: EXTENSION pgcrypto; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';


--
-- Name: vector; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS vector WITH SCHEMA public;


--
-- Name: EXTENSION vector; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION vector IS 'vector data type and ivfflat and hnsw access methods';


--
-- Name: flashcard_direction; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.flashcard_direction AS ENUM (
    'direct',
    'reverse',
    'both',
    'fillin',
    'fillin_reverse',
    'fillin_both',
    'just_information',
    'quiz_direct',
    'quiz_reverse',
    'quiz_both',
    'quiz_image_direct',
    'quiz_image_reverse',
    'quiz_image_both',
    'wordbank',
    'wordbank_reverse',
    'wordbank_both',
    'listen_wordbank',
    'listen_wordbank_reverse',
    'listen_wordbank_both',
    'auto'
);


--
-- Name: flashcard_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.flashcard_status AS ENUM (
    'new',
    'learning',
    'review',
    'graduated'
);


--
-- Name: payment_provider; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.payment_provider AS ENUM (
    'stripe',
    'paypal',
    'binance',
    'wise'
);


--
-- Name: payment_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.payment_status AS ENUM (
    'pending',
    'succeeded',
    'failed',
    'faulty'
);


--
-- Name: subscription_trigger; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.subscription_trigger AS ENUM (
    'comment',
    'definition',
    'edit'
);


--
-- Name: user_subscription_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.user_subscription_status AS ENUM (
    'active',
    'inactive',
    'cancelled',
    'past_due',
    'pending_cancellation'
);


--
-- Name: auto_subscribe_on_content(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.auto_subscribe_on_content() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_TABLE_NAME = 'definitions' THEN
        INSERT INTO valsi_subscriptions (valsi_id, user_id, trigger_type, source_definition_id)
        VALUES (NEW.valsiid, NEW.userid, 'definition', NEW.definitionid)
        ON CONFLICT (valsi_id, user_id, trigger_type) DO NOTHING;

    ELSIF TG_TABLE_NAME = 'comments' THEN
        INSERT INTO valsi_subscriptions (valsi_id, user_id, trigger_type, source_comment_id)
        SELECT t.valsiid, NEW.userid, 'comment', NEW.commentid
        FROM threads t
        WHERE t.threadid = NEW.threadid AND t.valsiid IS NOT NULL -- Only subscribe if valsiid is present
        ON CONFLICT (valsi_id, user_id, trigger_type) DO NOTHING;
    END IF;

    RETURN NEW;
END;
$$;


--
-- Name: can_edit_definition(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.can_edit_definition(p_definition_id integer, p_user_id integer) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 
        FROM definitions d
        WHERE d.definitionid = p_definition_id
        AND (
            d.userid = p_user_id  -- User is the owner
            OR NOT d.owner_only   -- Definition is not owner-only
        )
    );
END;
$$;


--
-- Name: can_read_comment(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.can_read_comment(comment_id integer, viewer_id integer) RETURNS boolean
    LANGUAGE sql STABLE
    AS $$
    SELECT EXISTS (SELECT 1 FROM comments c WHERE c.commentid = comment_id
                   AND can_read_comment_thread(c.threadid, viewer_id));
$$;


--
-- Name: can_read_comment_thread(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.can_read_comment_thread(thread_id integer, viewer_id integer) RETURNS boolean
    LANGUAGE sql STABLE
    AS $$
    SELECT EXISTS (
        SELECT 1 FROM threads t WHERE t.threadid = thread_id
        AND (t.collection_id IS NULL OR EXISTS (
            SELECT 1 FROM collections c WHERE c.collection_id = t.collection_id
            AND (c.is_public OR c.user_id = viewer_id)
        ))
    );
$$;


--
-- Name: check_level_completion(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_level_completion(p_user_id integer, p_level_id integer) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_min_cards INTEGER;
    v_min_success_rate FLOAT;
    v_completion_rate FLOAT;
    v_cards_completed INTEGER;
BEGIN
    -- Get level requirements
    SELECT min_cards, min_success_rate
    INTO v_min_cards, v_min_success_rate
    FROM flashcard_levels
    WHERE level_id = p_level_id;
    
    -- Get user progress
    SELECT 
        cards_completed,
        CASE WHEN total_answers > 0 
            THEN correct_answers::FLOAT / total_answers::FLOAT
            ELSE 0 
        END
    INTO v_cards_completed, v_completion_rate
    FROM user_level_progress
    WHERE user_id = p_user_id AND level_id = p_level_id;
    
    RETURN v_cards_completed >= v_min_cards 
        AND v_completion_rate >= v_min_success_rate;
END;
$$;


--
-- Name: check_level_prerequisites(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_level_prerequisites(p_user_id integer, p_level_id integer) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_has_prereqs BOOLEAN;
    v_all_completed BOOLEAN;
BEGIN
    -- Level with no prerequisites is always unlocked
    SELECT EXISTS(SELECT 1 FROM level_prerequisites WHERE level_id = p_level_id)
    INTO v_has_prereqs;
    IF NOT v_has_prereqs THEN
        RETURN true;
    END IF;

    -- Level has prerequisites: unlocked only if user has completed every prerequisite
    -- LEFT JOIN so we get one row per prerequisite; missing progress -> completed_at IS NULL -> false
    SELECT COALESCE(bool_and(ulp.completed_at IS NOT NULL), false)
    INTO v_all_completed
    FROM level_prerequisites lp
    LEFT JOIN user_level_progress ulp
        ON ulp.level_id = lp.prerequisite_id
        AND ulp.user_id = p_user_id
    WHERE lp.level_id = p_level_id;

    RETURN v_all_completed;
END;
$$;


--
-- Name: check_reaction_limit(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_reaction_limit(p_user_id integer, p_comment_id integer) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN (
        SELECT COALESCE(reaction_count, 0) < 5
        FROM user_reaction_counts
        WHERE user_id = p_user_id AND comment_id = p_comment_id
        UNION ALL
        SELECT true 
        LIMIT 1
    );
END;
$$;


--
-- Name: clean_subject(text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.clean_subject(subject text) RETURNS text
    LANGUAGE plpgsql IMMUTABLE
    AS $$
DECLARE
    clean text;
    modified boolean;
    bracket_start int;
    bracket_end int;
BEGIN
    clean := subject;
    
    LOOP
        modified := false;
        clean := trim(clean);
        
        IF lower(substr(clean, 1, 3)) = 're:' THEN
            clean := trim(substr(clean, 4));
            modified := true;
            CONTINUE;
        END IF;
        
        bracket_start := position('[' in clean);
        IF bracket_start > 0 THEN
            bracket_end := position(']' in substr(clean, bracket_start));
            IF bracket_end > 0 THEN
                clean := trim(substr(clean, 1, bracket_start - 1) || 
                            substr(clean, bracket_start + bracket_end));
                modified := true;
                CONTINUE;
            END IF;
        END IF;
        
        EXIT WHEN NOT modified;
    END LOOP;

    -- Replace multiple spaces with single space
    clean := regexp_replace(clean, '\s+', ' ', 'g');

    RETURN trim(clean);
END;
$$;


--
-- Name: cleanup_expired_webrtc_signals(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.cleanup_expired_webrtc_signals() RETURNS integer
    LANGUAGE plpgsql
    AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM webrtc_signaling 
    WHERE expires_at < CURRENT_TIMESTAMP OR is_processed = TRUE;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$;


--
-- Name: cleanup_item_images(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.cleanup_item_images() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM collection_item_images WHERE item_id = OLD.item_id;
    RETURN OLD;
END;
$$;


--
-- Name: cleanup_item_sounds(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.cleanup_item_sounds() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM collection_item_sounds WHERE item_id = OLD.item_id;
    RETURN OLD;
END;
$$;


--
-- Name: cleanup_orphaned_images(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.cleanup_orphaned_images() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM definition_images WHERE definition_id = OLD.definitionid;
    RETURN OLD;
END;
$$;


--
-- Name: cleanup_valsi_sounds(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.cleanup_valsi_sounds() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM valsi_sounds WHERE valsi_id = OLD.valsiid;
    RETURN OLD;
END;
$$;


--
-- Name: combined_rafsi(text, text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.combined_rafsi(official text, experimental text) RETURNS text
    LANGUAGE sql IMMUTABLE
    AS $$
    SELECT string_agg(token, ' ' ORDER BY token)
    FROM (SELECT DISTINCT token FROM regexp_split_to_table(concat_ws(' ', official, experimental), '\s+') token
          WHERE token <> '') tokens;
$$;


--
-- Name: complete_level_when_nothing_due(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.complete_level_when_nothing_due(p_user_id integer, p_level_id integer) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_cards INTEGER;
    v_seen INTEGER;
    v_due INTEGER;
BEGIN
    IF EXISTS (
        SELECT 1 FROM user_level_progress
        WHERE user_id = p_user_id AND level_id = p_level_id AND completed_at IS NOT NULL
    ) THEN
        RETURN true;
    END IF;
    -- A locked level cannot be completed this way.
    IF NOT check_level_prerequisites(p_user_id, p_level_id) THEN
        RETURN false;
    END IF;

    SELECT COUNT(*) INTO v_cards FROM flashcard_level_items WHERE level_id = p_level_id;
    IF v_cards = 0 THEN
        RETURN false;
    END IF;

    SELECT COUNT(DISTINCT fli.flashcard_id) INTO v_seen
    FROM flashcard_level_items fli
    WHERE fli.level_id = p_level_id
      AND EXISTS (
          SELECT 1 FROM flashcard_review_history fr
          WHERE fr.user_id = p_user_id AND fr.flashcard_id = fli.flashcard_id
      );
    IF v_seen < v_cards THEN
        RETURN false;
    END IF;

    SELECT COUNT(*) INTO v_due
    FROM flashcard_level_items fli
    JOIN user_flashcard_progress p
      ON p.flashcard_id = fli.flashcard_id AND p.user_id = p_user_id
    WHERE fli.level_id = p_level_id
      AND NOT p.archived
      AND p.next_review_at <= CURRENT_TIMESTAMP;
    IF v_due > 0 THEN
        RETURN false;
    END IF;

    INSERT INTO user_level_progress AS ulp
        (user_id, level_id, cards_completed, correct_answers, total_answers,
         last_activity_at, completed_at, completed_by_schedule)
    VALUES (p_user_id, p_level_id, 0, 0, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, true)
    ON CONFLICT (user_id, level_id) DO UPDATE
    SET completed_at = COALESCE(ulp.completed_at, CURRENT_TIMESTAMP),
        completed_by_schedule = ulp.completed_at IS NULL OR ulp.completed_by_schedule;

    UPDATE user_level_progress
    SET unlocked_at = CURRENT_TIMESTAMP
    WHERE user_id = p_user_id
      AND unlocked_at IS NULL
      AND check_level_prerequisites(user_id, level_id);

    RETURN true;
END;
$$;


--
-- Name: create_user_balance(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.create_user_balance() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO user_balances (user_id, balance_cents, created_at, updated_at)
    VALUES (NEW.userid, 0, NOW(), NOW())
    ON CONFLICT (user_id) DO NOTHING;
    RETURN NEW;
END;
$$;


--
-- Name: delete_orphaned_natlangwords(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.delete_orphaned_natlangwords() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM natlangwords n
    WHERE NOT EXISTS (
        SELECT 1 FROM threads t WHERE t.natlangwordid = n.wordid
    )
    AND NOT EXISTS (
        SELECT 1 FROM natlangwordvotes v WHERE v.natlangwordid = n.wordid
    )
    AND NOT EXISTS (
        SELECT 1 FROM keywordmapping k WHERE k.natlangwordid = n.wordid
    )
    AND n.time < (EXTRACT(EPOCH FROM (NOW() - INTERVAL '1 day'))::bigint);
END;
$$;


--
-- Name: editable_definition_rafsi(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.editable_definition_rafsi(p_definitionid integer) RETURNS text
    LANGUAGE sql STABLE
    AS $$
    SELECT CASE WHEN u.username = 'officialdata' THEN v.rafsi ELSE d.rafsi END
    FROM definitions d JOIN users u ON u.userid = d.userid JOIN valsi v ON v.valsiid = d.valsiid
    WHERE d.definitionid = p_definitionid;
$$;


--
-- Name: export_best_definitions(integer, boolean); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.export_best_definitions(p_langid integer, p_positive_only boolean) RETURNS TABLE(valsiid integer, definitionid integer, score bigint)
    LANGUAGE sql STABLE
    AS $$
WITH definition_scores AS (
    SELECT d.definitionid, d.valsiid, d.langid,
           COALESCE(SUM(dv.value), 0)::bigint AS score
    FROM definitions d
    LEFT JOIN definitionvotes dv ON dv.definitionid = d.definitionid
    WHERE d.langid = p_langid
    GROUP BY d.definitionid, d.valsiid, d.langid
),
filtered AS (
    SELECT ds.valsiid, ds.definitionid, ds.score
    FROM definition_scores ds
    WHERE ds.score > 0 OR p_positive_only = false
),
-- Positive-only mode: one deterministic best definition per word.
best_positive AS (
    SELECT DISTINCT ON (f.valsiid)
        f.valsiid,
        f.definitionid,
        f.score
    FROM filtered f
    WHERE p_positive_only = true
    ORDER BY f.valsiid, f.score DESC, f.definitionid ASC
)
SELECT b.valsiid, b.definitionid, b.score FROM best_positive b
UNION ALL
SELECT f.valsiid, f.definitionid, f.score FROM filtered f
WHERE p_positive_only = false;
$$;


--
-- Name: extract_plain_content(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.extract_plain_content() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
 DECLARE
     element JSONB;
     result TEXT := '';
 BEGIN
     IF NEW.content IS NULL THEN
         NEW.plain_content := '';
         RETURN NEW;
     END IF;

     FOR element IN SELECT * FROM jsonb_array_elements(NEW.content)
    LOOP
        IF element->>'type' = 'text' THEN
            result := result || (element->>'data') || ' ';
        END IF;
     END LOOP;

     NEW.plain_content := TRIM(result);
     RETURN NEW;
END;
$$;


--
-- Name: get_user_message_threads(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.get_user_message_threads(p_user_id integer) RETURNS TABLE(thread_id bigint, thread_name character varying, thread_type character varying, created_by integer, created_at timestamp with time zone, updated_at timestamp with time zone, is_active boolean, max_participants integer, last_message_at timestamp with time zone, last_message_preview text, message_count bigint, unread_count bigint, participant_count bigint, is_admin boolean)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        mt.thread_id,
        mt.thread_name,
        mt.thread_type,
        mt.created_by,
        mt.created_at,
        mt.updated_at,
        mt.is_active,
        mt.max_participants,
        mt.last_message_at,
        mt.last_message_preview,
        mt.message_count,
        COALESCE(tp.unread_count, 0) as unread_count,
        (SELECT COUNT(*) FROM thread_participants tp2 WHERE tp2.thread_id = mt.thread_id AND tp2.is_active = TRUE) as participant_count,
        (tp.role = 'admin') as is_admin
    FROM message_threads mt
    JOIN thread_participants tp ON mt.thread_id = tp.thread_id
    WHERE tp.user_id = p_user_id
    AND tp.is_active = TRUE
    AND mt.is_active = TRUE
    ORDER BY mt.last_message_at DESC NULLS LAST, mt.created_at DESC;
END;
$$;


--
-- Name: has_profile_image(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.has_profile_image(userid integer) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM user_profile_images 
        WHERE user_id = userid
    );
END;
$$;


--
-- Name: invalidate_cached_exports_for_valsi_lang(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.invalidate_cached_exports_for_valsi_lang(p_valsiid integer, p_langid integer) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM cached_dictionary_exports cde
    USING valsi v, languages target_lang, languages source_lang
    WHERE v.valsiid = p_valsiid
      AND target_lang.langid = p_langid
      AND cde.language_tag = target_lang.tag
      AND source_lang.langid = v.source_langid
      AND cde.source_language_tag = source_lang.tag;
END;
$$;


--
-- Name: invalidate_rafsi_dependents(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.invalidate_rafsi_dependents() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'UPDATE' THEN
        IF TG_TABLE_NAME = 'definitions' THEN
            IF OLD.rafsi IS NOT DISTINCT FROM NEW.rafsi AND OLD.valsiid = NEW.valsiid THEN RETURN NEW; END IF;
        ELSE
            IF OLD.rafsi IS NOT DISTINCT FROM NEW.rafsi AND OLD.word = NEW.word AND OLD.typeid = NEW.typeid AND OLD.source_langid IS NOT DISTINCT FROM NEW.source_langid THEN RETURN NEW; END IF;
            IF OLD.rafsi IS NOT DISTINCT FROM NEW.rafsi AND OLD.typeid NOT IN (1, 2, 7, 8) AND NEW.typeid NOT IN (1, 2, 7, 8) THEN RETURN NEW; END IF;
        END IF;
    ELSIF TG_OP = 'INSERT' THEN
        IF NEW.rafsi IS NULL THEN RETURN NEW; END IF;
    ELSE
        IF OLD.rafsi IS NULL THEN RETURN OLD; END IF;
    END IF;
    DELETE FROM cached_dictionary_exports;
    UPDATE valsi SET cached_decomposition = NULL, canonical_word = NULL,
                     related_expansion_key = NULL, trivial_se_checked = FALSE,
                     typeid = CASE WHEN typeid = 17 THEN 4 ELSE typeid END
    WHERE source_langid = 1 AND typeid IN (4, 17)
      AND (cached_decomposition IS NOT NULL OR canonical_word IS NOT NULL OR trivial_se_checked OR typeid = 17);
    RETURN COALESCE(NEW, OLD);
END;
$$;


--
-- Name: invalidate_valsi_cached_decomposition(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.invalidate_valsi_cached_decomposition() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.word IS DISTINCT FROM NEW.word
       OR OLD.rafsi IS DISTINCT FROM NEW.rafsi
       OR OLD.typeid IS DISTINCT FROM NEW.typeid
       OR OLD.source_langid IS DISTINCT FROM NEW.source_langid THEN
        NEW.cached_decomposition := NULL;
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: invalidate_valsi_canonical_word(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.invalidate_valsi_canonical_word() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.word IS DISTINCT FROM NEW.word THEN
        NEW.canonical_word := NULL;
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: invalidate_valsi_trivial_se(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.invalidate_valsi_trivial_se() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.word IS DISTINCT FROM NEW.word
       OR OLD.source_langid IS DISTINCT FROM NEW.source_langid
       OR (OLD.typeid IS DISTINCT FROM NEW.typeid
           AND NOT (OLD.typeid IN (4, 17) AND NEW.typeid IN (4, 17))) THEN
        NEW.related_expansion_key := NULL;
        NEW.trivial_se_checked := FALSE;
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: is_user_blocked(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.is_user_blocked(p_blocker_id integer, p_blocked_id integer) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM user_message_blocks 
        WHERE blocker_id = p_blocker_id 
        AND blocked_id = p_blocked_id 
        AND is_active = TRUE
    );
END;
$$;


--
-- Name: mark_messages_as_read(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.mark_messages_as_read() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Mark all messages in thread as read for this user
    UPDATE thread_participants 
    SET 
        last_read_at = CURRENT_TIMESTAMP,
        unread_count = 0
    WHERE thread_id = NEW.thread_id 
    AND user_id = NEW.user_id 
    AND is_active = TRUE;
    RETURN NEW;
END;
$$;


--
-- Name: notify_valsi_subscribers(integer, text, text, text, integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.notify_valsi_subscribers(p_valsi_id integer, p_event_type text, p_message text, p_link text, p_actor_id integer, p_definition_id integer DEFAULT NULL::integer) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO user_notifications (
        user_id, notification_type, message, link,
        valsi_id, actor_id, definition_id, created_at
    )
    SELECT DISTINCT
        vs.user_id, p_event_type, p_message, p_link,
        p_valsi_id, p_actor_id, p_definition_id, CURRENT_TIMESTAMP
    FROM valsi_subscriptions vs
    JOIN users u ON vs.user_id = u.userid
    WHERE vs.valsi_id = p_valsi_id
      AND NOT vs.unsubscribed
      AND vs.user_id != p_actor_id
      AND u.email IS NOT NULL
      AND u.role::text <> 'blocked'
      AND user_accepts_notifications(u.userid);
END;
$$;


--
-- Name: parse_email_date(text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.parse_email_date(date_text text) RETURNS timestamp with time zone
    LANGUAGE plpgsql
    AS $_$
DECLARE
  cleaned_date text;
BEGIN
  IF date_text IS NULL OR date_text = '' THEN
      RETURN NULL;
  END IF;

  -- Remove timezone names in parentheses
  cleaned_date := regexp_replace(date_text, '\s*\([A-Z]+\)', '', 'g');
  
  -- Convert EST/EDT to -0500/-0400
  cleaned_date := regexp_replace(cleaned_date, '\sEST\s*$', ' -0500', 'g');
  cleaned_date := regexp_replace(cleaned_date, '\sEDT\s*$', ' -0400', 'g');
  
  -- Parse as timestamp with timezone
  RETURN cleaned_date::timestamptz;
EXCEPTION WHEN OTHERS THEN
  RETURN NULL;
END;
$_$;


--
-- Name: prevent_admin_role_deletion(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.prevent_admin_role_deletion() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.role = 'admin' THEN
        RAISE EXCEPTION 'Cannot delete admin role';
    END IF;
    RETURN OLD;
END;
$$;


--
-- Name: process_successful_payment(integer, bigint); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.process_successful_payment(p_payment_id integer, p_amount_cents bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    PERFORM pg_advisory_xact_lock(p_payment_id);
    
    WITH payment_info AS (
        UPDATE payments 
        SET status = 'succeeded',
            completed_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = p_payment_id
        AND status = 'pending'
        RETURNING user_id
    )
    INSERT INTO balance_transactions (
        user_id, 
        amount_cents,
        currency,
        transaction_type,
        reference_id
    )
    SELECT 
        user_id,
        p_amount_cents,
        'USD',
        'payment_credit',
        p_payment_id::TEXT
    FROM payment_info;

    UPDATE user_balances
    SET balance_cents = balance_cents + p_amount_cents,
        total_spent_cents = total_spent_cents + p_amount_cents,
        updated_at = CURRENT_TIMESTAMP
    FROM payments
    WHERE payments.id = p_payment_id
    AND user_balances.user_id = payments.user_id;
END;
$$;


--
-- Name: refresh_definition_cached_valsi(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.refresh_definition_cached_valsi(p_definitionid integer) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_word TEXT;
    v_rafsi TEXT;
    v_source_langid INTEGER;
    v_typeid SMALLINT;
    v_type_name TEXT;
    v_cached_decomposition TEXT;
    v_canonical_word TEXT;
    d_definition TEXT;
    d_notes TEXT;
    d_selmaho TEXT;
    d_rafsi TEXT;
    u_username TEXT;
    l_realname TEXT;
    kw_gloss TEXT;
    kw_text TEXT;
BEGIN
    SELECT COALESCE(string_agg(LOWER(n.word), '|' ORDER BY n.word) FILTER (WHERE k.place = 0), ''),
           COALESCE(string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' '), '')
    INTO kw_gloss, kw_text
    FROM keywordmapping k
    JOIN natlangwords n ON k.natlangwordid = n.wordid
    WHERE k.definitionid = p_definitionid;

    SELECT d.definition, d.notes, d.selmaho, d.rafsi,
           v.word, v.rafsi, v.cached_decomposition, v.canonical_word, v.source_langid, v.typeid,
           vt.descriptor, u.username, l.realname
    INTO d_definition, d_notes, d_selmaho, d_rafsi,
         v_word, v_rafsi, v_cached_decomposition, v_canonical_word, v_source_langid, v_typeid,
         v_type_name, u_username, l_realname
    FROM definitions d
    JOIN valsi v ON d.valsiid = v.valsiid
    JOIN users u ON d.userid = u.userid
    JOIN languages l ON d.langid = l.langid
    JOIN valsitypes vt ON v.typeid = vt.typeid
    WHERE d.definitionid = p_definitionid;

    IF NOT FOUND THEN
        RETURN;
    END IF;

    UPDATE definitions
    SET
        cached_username = u_username,
        cached_langrealname = l_realname,
        cached_type_name = v_type_name,
        cached_valsiword = v_word,
        cached_rafsi = combined_rafsi(v_rafsi, d_rafsi),
        cached_official_rafsi = v_rafsi,
        cached_decomposition = v_cached_decomposition,
        cached_canonical_word = v_canonical_word,
        cached_source_langid = v_source_langid,
        cached_typeid = v_typeid,
        cached_glosswords = kw_gloss,
        cached_search_text = LOWER(
            COALESCE(v_word, '') || ' ' ||
            COALESCE(combined_rafsi(v_rafsi, d_rafsi), '') || ' ' ||
            COALESCE(d_definition, '') || ' ' ||
            COALESCE(d_notes, '') || ' ' ||
            COALESCE(d_selmaho, '') || ' ' ||
            COALESCE(kw_text, '')
        )
    WHERE definitionid = p_definitionid;
END;
$$;


--
-- Name: refresh_natlangwordbestplaces_for_delete(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.refresh_natlangwordbestplaces_for_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
  BEGIN
    PERFORM reset_natlangwordbestplace(OLD.natlangwordid);
    RETURN NULL;
  END;
$$;


--
-- Name: refresh_natlangwordbestplaces_for_upsert(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.refresh_natlangwordbestplaces_for_upsert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
  BEGIN
    PERFORM reset_natlangwordbestplace(NEW.natlangwordid);
    RETURN NULL;
  END;
$$;


--
-- Name: refresh_reaction_counts(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.refresh_reaction_counts() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_reaction_counts;
    RETURN NULL;
END;
$$;


--
-- Name: reload_natlangwordbestplaces(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.reload_natlangwordbestplaces() RETURNS void
    LANGUAGE plpgsql
    AS $$
  DECLARE
    _wordid integer;
  BEGIN

    TRUNCATE natlangwordbestplaces;

    FOR _wordid IN
      SELECT DISTINCT natlangwordid AS wordid
        FROM natlangwordvotes
    LOOP
      PERFORM reset_natlangwordbestplace(_wordid);
    END LOOP;

  END;
$$;


--
-- Name: reset_natlangwordbestplace(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.reset_natlangwordbestplace(_wordid integer) RETURNS void
    LANGUAGE plpgsql
    AS $$
  DECLARE
    _new RECORD;
  BEGIN

    SELECT natlangwordid AS wordid, definitionid, place, min(time) AS time, sum(value) AS score
      INTO _new
      FROM natlangwordvotes
      WHERE natlangwordid = _wordid
      GROUP BY wordid, definitionid, place
      ORDER BY score DESC, time
      LIMIT 1;

    DELETE
      FROM natlangwordbestplaces
      WHERE wordid = _wordid;

    IF _new IS NOT NULL THEN
      INSERT
        INTO natlangwordbestplaces (wordid, definitionid, place, score)
        VALUES (_new.wordid, _new.definitionid, _new.place, _new.score);
    END IF;

  END;
$$;


--
-- Name: set_message_sent_at(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.set_message_sent_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
  IF NEW.date IS NOT NULL AND NEW.sent_at IS NULL THEN
      NEW.sent_at := parse_email_date(NEW.date);
  END IF;
  IF NEW.sent_at IS NULL THEN
      NEW.sent_at := NOW();
  END IF;
  RETURN NEW;
END;
$$;


--
-- Name: sync_admin_permissions(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.sync_admin_permissions() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO role_permissions (role, permission_id)
    SELECT 'admin', id
    FROM permissions
    WHERE NOT EXISTS (
        SELECT 1 
        FROM role_permissions rp 
        WHERE rp.role = 'admin' 
        AND rp.permission_id = permissions.id
    );
    RETURN NULL;
END;
$$;


--
-- Name: sync_definition_cache_fields(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.sync_definition_cache_fields() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    target_definitionid INTEGER;
    target_valsiid INTEGER;
    target_natlangwordid INTEGER;
    kw_gloss TEXT := '';
    kw_text TEXT := '';
    v_word TEXT;
    v_rafsi TEXT;
    d_rafsi TEXT;
    v_cached_decomposition TEXT;
    v_canonical_word TEXT;
    v_source_langid INTEGER;
    v_typeid INTEGER;
    v_type_name TEXT;
    u_username TEXT;
    l_realname TEXT;
    d_definition TEXT;
    d_notes TEXT;
    d_selmaho TEXT;
BEGIN
    IF TG_TABLE_NAME = 'definitions' THEN
        target_definitionid := COALESCE(NEW.definitionid, OLD.definitionid);
        target_valsiid := COALESCE(NEW.valsiid, OLD.valsiid);
    ELSIF TG_TABLE_NAME = 'keywordmapping' THEN
        target_definitionid := COALESCE(NEW.definitionid, OLD.definitionid);
    ELSIF TG_TABLE_NAME = 'valsi' THEN
        target_valsiid := COALESCE(NEW.valsiid, OLD.valsiid);
    ELSIF TG_TABLE_NAME = 'natlangwords' THEN
        target_natlangwordid := COALESCE(NEW.wordid, OLD.wordid);
    END IF;

    IF target_definitionid IS NOT NULL THEN
        SELECT COALESCE(string_agg(LOWER(n.word), '|' ORDER BY n.word) FILTER (WHERE k.place = 0), ''),
               COALESCE(string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' '), '')
        INTO kw_gloss, kw_text
        FROM keywordmapping k
        JOIN natlangwords n ON k.natlangwordid = n.wordid
        WHERE k.definitionid = target_definitionid;
    END IF;

    IF TG_TABLE_NAME = 'definitions' THEN
        SELECT d.definition, d.notes, d.selmaho, d.rafsi,
               v.word, v.rafsi, v.cached_decomposition, v.canonical_word, v.source_langid, v.typeid,
               vt.descriptor, u.username, l.realname
        INTO d_definition, d_notes, d_selmaho, d_rafsi,
             v_word, v_rafsi, v_cached_decomposition, v_canonical_word, v_source_langid, v_typeid,
             v_type_name, u_username, l_realname
        FROM definitions d
        JOIN valsi v ON d.valsiid = v.valsiid
        JOIN users u ON d.userid = u.userid
        JOIN languages l ON d.langid = l.langid
        JOIN valsitypes vt ON v.typeid = vt.typeid
        WHERE d.definitionid = target_definitionid;

        UPDATE definitions
        SET
            cached_username = u_username,
            cached_langrealname = l_realname,
            cached_type_name = v_type_name,
            cached_valsiword = v_word,
            cached_rafsi = combined_rafsi(v_rafsi, d_rafsi),
            cached_official_rafsi = v_rafsi,
            cached_decomposition = v_cached_decomposition,
            cached_canonical_word = v_canonical_word,
            cached_source_langid = v_source_langid,
            cached_typeid = v_typeid,
            cached_glosswords = kw_gloss,
            cached_search_text = LOWER(
                COALESCE(v_word, '') || ' ' ||
                COALESCE(combined_rafsi(v_rafsi, d_rafsi), '') || ' ' ||
                COALESCE(d_definition, '') || ' ' ||
                COALESCE(d_notes, '') || ' ' ||
                COALESCE(d_selmaho, '') || ' ' ||
                COALESCE(kw_text, '')
            )
        WHERE definitionid = target_definitionid;

    ELSIF TG_TABLE_NAME = 'keywordmapping' THEN
        UPDATE definitions d
        SET
            cached_glosswords = kw_gloss,
            cached_search_text = LOWER(
                COALESCE(d.cached_valsiword, '') || ' ' ||
                COALESCE(d.cached_rafsi, '') || ' ' ||
                COALESCE(d.definition, '') || ' ' ||
                COALESCE(d.notes, '') || ' ' ||
                COALESCE(d.selmaho, '') || ' ' ||
                COALESCE(kw_text, '')
            ),
            embedding = NULL
        WHERE d.definitionid = target_definitionid;

    ELSIF TG_TABLE_NAME = 'valsi' THEN
        WITH def_ids AS (
            SELECT definitionid FROM definitions WHERE valsiid = target_valsiid
        ),
        valsi_data AS (
            SELECT v.word, v.rafsi, v.cached_decomposition, v.canonical_word,
                   v.source_langid, v.typeid, vt.descriptor
            FROM valsi v
            JOIN valsitypes vt ON v.typeid = vt.typeid
            WHERE v.valsiid = target_valsiid
        ),
        keyword_agg AS (
            SELECT k.definitionid,
                   string_agg(LOWER(n.word), '|' ORDER BY n.word) FILTER (WHERE k.place = 0) AS glosswords,
                   string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' ') AS keyword_text
            FROM keywordmapping k
            JOIN natlangwords n ON k.natlangwordid = n.wordid
            JOIN definitions d2 ON d2.definitionid = k.definitionid
            WHERE d2.valsiid = target_valsiid
            GROUP BY k.definitionid
        )
        UPDATE definitions
        SET
            cached_valsiword = vd.word,
            cached_rafsi = combined_rafsi(vd.rafsi, definitions.rafsi),
            cached_official_rafsi = vd.rafsi,
            cached_decomposition = vd.cached_decomposition,
            cached_canonical_word = vd.canonical_word,
            cached_source_langid = vd.source_langid,
            cached_typeid = vd.typeid,
            cached_type_name = vd.descriptor,
            cached_glosswords = COALESCE(ka.glosswords, ''),
            cached_search_text = LOWER(
                COALESCE(vd.word, '') || ' ' ||
                COALESCE(combined_rafsi(vd.rafsi, definitions.rafsi), '') || ' ' ||
                COALESCE(definitions.definition, '') || ' ' ||
                COALESCE(definitions.notes, '') || ' ' ||
                COALESCE(definitions.selmaho, '') || ' ' ||
                COALESCE(ka.keyword_text, '')
            )
        FROM def_ids
        CROSS JOIN valsi_data vd
        LEFT JOIN keyword_agg ka ON ka.definitionid = def_ids.definitionid
        WHERE definitions.definitionid = def_ids.definitionid;

    ELSIF TG_TABLE_NAME = 'users' THEN
        UPDATE definitions d
        SET cached_username = NEW.username
        WHERE d.userid = NEW.userid;

    ELSIF TG_TABLE_NAME = 'languages' THEN
        UPDATE definitions d
        SET cached_langrealname = NEW.realname
        WHERE d.langid = NEW.langid;

    ELSIF TG_TABLE_NAME = 'valsitypes' THEN
        UPDATE definitions d
        SET
            cached_type_name = NEW.descriptor,
            cached_typeid = NEW.typeid
        WHERE d.cached_typeid = NEW.typeid;

    ELSIF TG_TABLE_NAME = 'natlangwords' THEN
        WITH affected AS (
            SELECT DISTINCT k.definitionid
            FROM keywordmapping k
            WHERE k.natlangwordid = target_natlangwordid
        ),
        keyword_agg AS (
            SELECT k.definitionid,
                   MAX(d2.cached_valsiword) AS cached_valsiword,
                   MAX(d2.cached_rafsi) AS cached_rafsi,
                   MAX(d2.definition) AS def_text,
                   MAX(d2.notes) AS notes_text,
                   MAX(d2.selmaho) AS selmaho_text,
                   string_agg(LOWER(n.word), '|' ORDER BY n.word) FILTER (WHERE k.place = 0) AS glosswords,
                   string_agg(LOWER(n.word || ' ' || COALESCE(n.meaning, '')), ' ') AS keyword_text
            FROM keywordmapping k
            JOIN natlangwords n ON k.natlangwordid = n.wordid
            JOIN definitions d2 ON d2.definitionid = k.definitionid
            JOIN affected a ON a.definitionid = k.definitionid
            GROUP BY k.definitionid
        )
        UPDATE definitions
        SET
            cached_glosswords = COALESCE(ka.glosswords, ''),
            cached_search_text = LOWER(
                COALESCE(ka.cached_valsiword, '') || ' ' ||
                COALESCE(ka.cached_rafsi, '') || ' ' ||
                COALESCE(ka.def_text, '') || ' ' ||
                COALESCE(ka.notes_text, '') || ' ' ||
                COALESCE(ka.selmaho_text, '') || ' ' ||
                COALESCE(ka.keyword_text, '')
            ),
            embedding = NULL
        FROM affected
        LEFT JOIN keyword_agg ka ON ka.definitionid = affected.definitionid
        WHERE definitions.definitionid = affected.definitionid;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$;


--
-- Name: trg_collection_item_images_after_delete_cleanup(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.trg_collection_item_images_after_delete_cleanup() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    PERFORM try_delete_collection_image_if_orphan(OLD.collection_image_id);
    RETURN OLD;
END;
$$;


--
-- Name: trg_collection_item_images_after_update_cleanup(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.trg_collection_item_images_after_update_cleanup() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.collection_image_id IS DISTINCT FROM NEW.collection_image_id THEN
        PERFORM try_delete_collection_image_if_orphan(OLD.collection_image_id);
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: trg_collections_before_delete_cover_cleanup(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.trg_collections_before_delete_cover_cleanup() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    PERFORM try_delete_collection_image_if_orphan_for_deleted_collection(
        OLD.cover_collection_image_id,
        OLD.collection_id
    );
    RETURN OLD;
END;
$$;


--
-- Name: trg_collections_cover_update_cleanup(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.trg_collections_cover_update_cleanup() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.cover_collection_image_id IS DISTINCT FROM NEW.cover_collection_image_id
       AND OLD.cover_collection_image_id IS NOT NULL THEN
        PERFORM try_delete_collection_image_if_orphan(OLD.cover_collection_image_id);
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: trg_invalidate_cached_exports_for_definition(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.trg_invalidate_cached_exports_for_definition() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    _old_valsiid integer;
    _old_langid integer;
    _new_valsiid integer;
    _new_langid integer;
BEGIN
    IF TG_OP = 'DELETE' THEN
        PERFORM invalidate_cached_exports_for_valsi_lang(OLD.valsiid, OLD.langid);
        RETURN OLD;
    END IF;

    _new_valsiid := NEW.valsiid;
    _new_langid := NEW.langid;

    IF TG_OP = 'UPDATE' THEN
        _old_valsiid := OLD.valsiid;
        _old_langid := OLD.langid;
        IF _old_valsiid IS DISTINCT FROM _new_valsiid OR _old_langid IS DISTINCT FROM _new_langid THEN
            PERFORM invalidate_cached_exports_for_valsi_lang(_old_valsiid, _old_langid);
        END IF;
    END IF;

    PERFORM invalidate_cached_exports_for_valsi_lang(_new_valsiid, _new_langid);
    RETURN NEW;
END;
$$;


--
-- Name: trg_invalidate_cached_exports_for_vote(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.trg_invalidate_cached_exports_for_vote() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    _old_valsiid integer;
    _old_langid integer;
    _new_valsiid integer;
    _new_langid integer;
BEGIN
    IF TG_OP = 'DELETE' THEN
        PERFORM invalidate_cached_exports_for_valsi_lang(OLD.valsiid, OLD.langid);
        RETURN OLD;
    END IF;

    _new_valsiid := NEW.valsiid;
    _new_langid := NEW.langid;

    IF TG_OP = 'UPDATE' THEN
        _old_valsiid := OLD.valsiid;
        _old_langid := OLD.langid;
        IF _old_valsiid IS DISTINCT FROM _new_valsiid OR _old_langid IS DISTINCT FROM _new_langid THEN
            PERFORM invalidate_cached_exports_for_valsi_lang(_old_valsiid, _old_langid);
        END IF;
    END IF;

    PERFORM invalidate_cached_exports_for_valsi_lang(_new_valsiid, _new_langid);
    RETURN NEW;
END;
$$;


--
-- Name: trigger_cleanup_natlangwords(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.trigger_cleanup_natlangwords() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    PERFORM public.delete_orphaned_natlangwords();
    PERFORM public.reload_natlangwordbestplaces();
    RETURN NULL;
END;
$$;


--
-- Name: trigger_set_collection_item_timestamp(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.trigger_set_collection_item_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$;


--
-- Name: trigger_set_timestamp(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.trigger_set_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$;


--
-- Name: try_delete_collection_image_if_orphan(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.try_delete_collection_image_if_orphan(p_id integer) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF p_id IS NULL THEN
        RETURN;
    END IF;
    IF EXISTS (SELECT 1 FROM collection_item_images WHERE collection_image_id = p_id) THEN
        RETURN;
    END IF;
    IF EXISTS (SELECT 1 FROM collections WHERE cover_collection_image_id = p_id) THEN
        RETURN;
    END IF;
    DELETE FROM collection_images WHERE collection_image_id = p_id;
END;
$$;


--
-- Name: try_delete_collection_image_if_orphan_for_deleted_collection(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.try_delete_collection_image_if_orphan_for_deleted_collection(p_id integer, p_excluded_collection_id integer) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF p_id IS NULL THEN
        RETURN;
    END IF;
    IF EXISTS (SELECT 1 FROM collection_item_images WHERE collection_image_id = p_id) THEN
        RETURN;
    END IF;
    IF EXISTS (
        SELECT 1 FROM collections
        WHERE cover_collection_image_id = p_id
          AND collection_id IS DISTINCT FROM p_excluded_collection_id
    ) THEN
        RETURN;
    END IF;
    DELETE FROM collection_images WHERE collection_image_id = p_id;
END;
$$;


--
-- Name: update_cleaned_subject(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_cleaned_subject() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.cleaned_subject := clean_subject(NEW.subject);
    RETURN NEW;
END;
$$;


--
-- Name: update_comment_activity_timestamp(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_comment_activity_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.last_activity_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;


--
-- Name: update_comment_counter(integer, text, boolean); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_comment_counter(p_comment_id integer, p_counter_type text, p_increment boolean) RETURNS void
    LANGUAGE plpgsql
    AS $_$
DECLARE
    v_amount INTEGER;
BEGIN
    v_amount := CASE WHEN p_increment THEN 1 ELSE -1 END;
    
    EXECUTE format('
        INSERT INTO comment_activity_counters (comment_id, total_%I)
        VALUES ($1, $2)
        ON CONFLICT (comment_id) 
        DO UPDATE SET total_%I = comment_activity_counters.total_%I + $2',
        p_counter_type, p_counter_type, p_counter_type
    ) USING p_comment_id, v_amount;
END;
$_$;


--
-- Name: update_comment_counter_timestamp(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_comment_counter_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;


--
-- Name: update_comment_reply_count(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_comment_reply_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.parentid IS NOT NULL THEN
            UPDATE comment_activity_counters
            SET total_replies = total_replies + 1
            WHERE comment_id = NEW.parentid;
        END IF;
    ELSIF TG_OP = 'DELETE' THEN
        IF OLD.parentid IS NOT NULL THEN
            UPDATE comment_activity_counters
            SET total_replies = total_replies - 1
            WHERE comment_id = OLD.parentid;
        END IF;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: update_followers_count(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_followers_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE users SET followers = followers + 1 
        WHERE userid = NEW.followee_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE users SET followers = followers - 1 
        WHERE userid = OLD.followee_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: update_level_progress(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_level_progress() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO user_level_progress AS ulp (
        user_id, level_id,
        cards_completed, correct_answers, total_answers,
        last_activity_at
    )
    SELECT
        NEW.user_id,
        fli.level_id,
        COUNT(DISTINCT fr.flashcard_id),
        -- Pass = rating >= 2 (FSRS: only 1 "Again" is a lapse). Since the subtractive rating
        -- policy (V-383e3b79) stores a correct quiz/word-bank answer as 2, counting >= 3 made
        -- the success rate 0 on those courses, so no level could ever be completed.
        SUM(CASE WHEN fr.rating >= 2 THEN 1 ELSE 0 END),
        COUNT(*),
        CURRENT_TIMESTAMP
    FROM flashcard_review_history fr
    JOIN flashcard_level_items fli ON fr.flashcard_id = fli.flashcard_id
    WHERE fr.user_id = NEW.user_id
    AND fli.level_id IN (
        SELECT level_id
        FROM flashcard_level_items
        WHERE flashcard_id = NEW.flashcard_id
    )
    GROUP BY fli.level_id
    ON CONFLICT (user_id, level_id) DO UPDATE
    SET
        cards_completed = EXCLUDED.cards_completed,
        correct_answers = EXCLUDED.correct_answers,
        total_answers = EXCLUDED.total_answers,
        last_activity_at = EXCLUDED.last_activity_at;

    -- Judge completion after the upsert so check_level_completion sees the new counts
    -- (covers both the first insert and the update branch).
    UPDATE user_level_progress
    SET completed_at = CURRENT_TIMESTAMP
    WHERE user_id = NEW.user_id
      AND completed_at IS NULL
      AND level_id IN (SELECT level_id FROM flashcard_level_items WHERE flashcard_id = NEW.flashcard_id)
      AND check_level_completion(user_id, level_id);

    UPDATE user_level_progress
    SET unlocked_at = CASE
        WHEN check_level_prerequisites(user_id, level_id)
            AND unlocked_at IS NULL
        THEN CURRENT_TIMESTAMP
        ELSE unlocked_at
    END
    WHERE user_id = NEW.user_id;

    RETURN NEW;
END;
$$;


--
-- Name: update_premium_status(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_premium_status() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- When balance increases, extend premium status
    IF NEW.balance_cents > OLD.balance_cents THEN
        NEW.premium_expires_at = GREATEST(
            COALESCE(NEW.premium_expires_at, NOW()),
            NOW() + INTERVAL '1 month'
        );
    END IF;
    
    -- When premium expires, reset balance
    IF NEW.premium_expires_at IS NOT NULL AND NEW.premium_expires_at < NOW() THEN
        NEW.balance_cents = 0;
        NEW.premium_expires_at = NULL;
    END IF;
    
    RETURN NEW;
END;
$$;


--
-- Name: update_reaction_count(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_reaction_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Only increment if this is the user's first reaction for this comment
        IF NOT EXISTS (
            SELECT 1 
            FROM comment_reactions 
            WHERE comment_id = NEW.comment_id 
            AND user_id = NEW.user_id 
            AND id != NEW.id
        ) THEN
            UPDATE comment_activity_counters
            SET total_reactions = total_reactions + 1,
                last_activity_at = CURRENT_TIMESTAMP
            WHERE comment_id = NEW.comment_id;
        END IF;
    ELSIF TG_OP = 'DELETE' THEN
        -- Only decrement if this was user's last reaction
        -- Note: We don't need to exclude OLD record as it's already deleted
        IF NOT EXISTS (
            SELECT 1 
            FROM comment_reactions 
            WHERE comment_id = OLD.comment_id 
            AND user_id = OLD.user_id
        ) THEN
            UPDATE comment_activity_counters
            SET total_reactions = total_reactions - 1,
                last_activity_at = CURRENT_TIMESTAMP
            WHERE comment_id = OLD.comment_id;
        END IF;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: update_thread_message_stats(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_thread_message_stats() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    preview_text TEXT;
BEGIN
    IF TG_OP = 'INSERT' AND NEW.is_deleted = FALSE THEN
        BEGIN
            preview_text := LEFT(convert_from(decode(NEW.encrypted_content, 'base64'), 'UTF8'), 100);
        EXCEPTION WHEN OTHERS THEN
            preview_text := LEFT(NEW.encrypted_content, 100);
        END;

        UPDATE message_threads
        SET message_count = message_count + 1,
            last_message_at = NEW.created_at,
            last_message_preview = preview_text,
            updated_at = CURRENT_TIMESTAMP
        WHERE thread_id = NEW.thread_id;

        UPDATE thread_participants tp
        SET unread_count = tp.unread_count + 1
        WHERE tp.thread_id = NEW.thread_id
          AND tp.user_id != NEW.sender_id
          AND tp.is_active = TRUE
          AND user_accepts_notifications(tp.user_id);
    ELSIF TG_OP = 'UPDATE' AND OLD.is_deleted = FALSE AND NEW.is_deleted = TRUE THEN
        UPDATE message_threads
        SET message_count = GREATEST(message_count - 1, 0)
        WHERE thread_id = NEW.thread_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: update_thread_stats(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_thread_stats() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Update stats for both old and new thread IDs (for comment moves)
    IF TG_OP = 'DELETE' OR TG_OP = 'UPDATE' THEN
        UPDATE threads SET
            last_comment_id = (SELECT commentid FROM comments WHERE threadid = OLD.threadid ORDER BY time DESC, commentid DESC LIMIT 1),
            last_comment_user_id = (SELECT userid FROM comments WHERE threadid = OLD.threadid ORDER BY time DESC, commentid DESC LIMIT 1),
            last_comment_time = (SELECT time FROM comments WHERE threadid = OLD.threadid ORDER BY time DESC, commentid DESC LIMIT 1),
            last_comment_subject = (SELECT subject FROM comments WHERE threadid = OLD.threadid ORDER BY time DESC, commentid DESC LIMIT 1),
            last_comment_content = (SELECT content FROM comments WHERE threadid = OLD.threadid ORDER BY time DESC, commentid DESC LIMIT 1),
            total_comments = (SELECT COUNT(*) FROM comments WHERE threadid = OLD.threadid),
            first_comment_subject = (SELECT subject FROM comments WHERE threadid = OLD.threadid ORDER BY time ASC, commentid ASC LIMIT 1),
            first_comment_content = (SELECT content FROM comments WHERE threadid = OLD.threadid ORDER BY time ASC, commentid ASC LIMIT 1)
        WHERE threadid = OLD.threadid;
    END IF;
    
    IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
        UPDATE threads SET
            last_comment_id = (SELECT commentid FROM comments WHERE threadid = NEW.threadid ORDER BY time DESC, commentid DESC LIMIT 1),
            last_comment_user_id = (SELECT userid FROM comments WHERE threadid = NEW.threadid ORDER BY time DESC, commentid DESC LIMIT 1),
            last_comment_time = (SELECT time FROM comments WHERE threadid = NEW.threadid ORDER BY time DESC, commentid DESC LIMIT 1),
            last_comment_subject = (SELECT subject FROM comments WHERE threadid = NEW.threadid ORDER BY time DESC, commentid DESC LIMIT 1),
            last_comment_content = (SELECT content FROM comments WHERE threadid = NEW.threadid ORDER BY time DESC, commentid DESC LIMIT 1),
            total_comments = (SELECT COUNT(*) FROM comments WHERE threadid = NEW.threadid),
            first_comment_subject = (SELECT subject FROM comments WHERE threadid = NEW.threadid ORDER BY time ASC, commentid ASC LIMIT 1),
            first_comment_content = (SELECT content FROM comments WHERE threadid = NEW.threadid ORDER BY time ASC, commentid ASC LIMIT 1),
            creator_user_id = (SELECT userid FROM comments WHERE threadid = NEW.threadid ORDER BY time ASC, commentid ASC LIMIT 1),
            creator_username = (SELECT username FROM users WHERE userid = (SELECT userid FROM comments WHERE threadid = NEW.threadid ORDER BY time ASC, commentid ASC LIMIT 1))
        WHERE threadid = NEW.threadid;
    END IF;

    RETURN NULL;
END;
$$;


--
-- Name: update_updated_at_column(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_updated_at_column() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;


--
-- Name: user_accepts_notifications(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.user_accepts_notifications(p_user_id integer) RETURNS boolean
    LANGUAGE sql STABLE
    AS $$
    SELECT COALESCE(
        (
            SELECT (metadata->>'notifications_enabled')::boolean
            FROM users
            WHERE userid = p_user_id
        ),
        true
    );
$$;


--
-- Name: wave_search_author_sync(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.wave_search_author_sync() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE wave_search_documents SET author = coalesce(NEW.username,'')
    WHERE kind = 'comment' AND source_id IN (SELECT commentid FROM comments WHERE userid = NEW.userid);
    RETURN NULL;
END $$;


--
-- Name: wave_search_comment_sync(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.wave_search_comment_sync() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        DELETE FROM wave_search_documents WHERE kind = 'comment' AND source_id = OLD.commentid;
    ELSE
        PERFORM wave_search_upsert('comment', NEW.commentid,
            CASE WHEN NEW.import_source IN ('jbotcan','freeforums') THEN NEW.import_source ELSE 'comments' END,
            NEW.subject, NEW.plain_content, (SELECT username FROM users WHERE userid = NEW.userid),
            to_timestamp(NEW.time));
    END IF;
    RETURN NULL;
END $$;


--
-- Name: wave_search_definition_sync(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.wave_search_definition_sync() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        DELETE FROM wave_search_documents WHERE kind = 'native_wiki' AND source_id = OLD.definitionid;
    ELSE
        PERFORM wave_search_native_sync(NEW.definitionid);
    END IF;
    RETURN NULL;
END $$;


--
-- Name: wave_search_mail_sync(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.wave_search_mail_sync() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        DELETE FROM wave_search_documents WHERE kind = 'mail' AND source_id = OLD.id;
    ELSE
        PERFORM wave_search_upsert('mail', NEW.id, 'mail', NEW.subject,
                                   NEW.content, NEW.from_address, NEW.sent_at);
    END IF;
    RETURN NULL;
END $$;


--
-- Name: wave_search_native_sync(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.wave_search_native_sync(def_id integer) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    PERFORM wave_search_upsert('native_wiki', d.definitionid, 'wiki', v.word, d.definition, '', d.created_at)
    FROM definitions d JOIN valsi v ON v.valsiid = d.valsiid
    WHERE d.definitionid = def_id AND v.typeid = 16
      AND coalesce(d.metadata->>'is_redirect','false') <> 'true';
    IF NOT FOUND THEN
        DELETE FROM wave_search_documents WHERE kind = 'native_wiki' AND source_id = def_id;
    END IF;
END $$;


--
-- Name: wave_search_upsert(text, integer, text, text, text, text, timestamp with time zone); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.wave_search_upsert(k text, sid integer, src text, heading text, text_body text, writer text, edited timestamp with time zone) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE changed_id BIGINT;
BEGIN
    INSERT INTO wave_search_documents (kind, source_id, source, title, body, author, edited_at)
    VALUES (k, sid, src, coalesce(heading,''), coalesce(text_body,''), coalesce(writer,''), edited)
    ON CONFLICT (kind, source_id) DO UPDATE SET
        source = EXCLUDED.source, title = EXCLUDED.title, body = EXCLUDED.body,
        author = EXCLUDED.author, edited_at = EXCLUDED.edited_at,
        generation = wave_search_documents.generation +
            CASE WHEN (wave_search_documents.title, wave_search_documents.body)
                IS DISTINCT FROM (EXCLUDED.title, EXCLUDED.body) THEN 1 ELSE 0 END,
        embedding = CASE WHEN (wave_search_documents.title, wave_search_documents.body)
                IS DISTINCT FROM (EXCLUDED.title, EXCLUDED.body) THEN NULL
                ELSE wave_search_documents.embedding END
    RETURNING CASE WHEN embedding IS NULL THEN document_id END INTO changed_id;
    IF changed_id IS NOT NULL THEN
        DELETE FROM wave_search_chunks WHERE document_id = changed_id;
    END IF;
END $$;


--
-- Name: wave_search_valsi_sync(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.wave_search_valsi_sync() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE def_id INTEGER;
BEGIN
    FOR def_id IN SELECT definitionid FROM definitions WHERE valsiid = NEW.valsiid LOOP
        PERFORM wave_search_native_sync(def_id);
    END LOOP;
    RETURN NULL;
END $$;


--
-- Name: wave_search_wiki_sync(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.wave_search_wiki_sync() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        DELETE FROM wave_search_documents WHERE kind = 'wiki' AND source_id = OLD.page_id;
    ELSIF NEW.is_redirect THEN
        DELETE FROM wave_search_documents WHERE kind = 'wiki' AND source_id = NEW.page_id;
    ELSE
        PERFORM wave_search_upsert('wiki', NEW.page_id, 'wiki', NEW.title, NEW.plain_text, '', NEW.last_edited);
    END IF;
    RETURN NULL;
END $$;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: assistant_chats; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.assistant_chats (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id integer NOT NULL,
    title text DEFAULT ''::text NOT NULL,
    messages jsonb DEFAULT '[]'::jsonb NOT NULL,
    primary_model_id text,
    scroll_top double precision DEFAULT 0 NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: balance_transactions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.balance_transactions (
    id integer NOT NULL,
    user_id integer NOT NULL,
    amount_cents bigint NOT NULL,
    currency text NOT NULL,
    transaction_type text NOT NULL,
    reference_id text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: balance_transactions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.balance_transactions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: balance_transactions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.balance_transactions_id_seq OWNED BY public.balance_transactions.id;


--
-- Name: cached_dictionary_exports; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.cached_dictionary_exports (
    id integer NOT NULL,
    language_tag text NOT NULL,
    format text NOT NULL,
    content bytea NOT NULL,
    content_type text NOT NULL,
    filename text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    source_language_tag text DEFAULT 'jbo'::text NOT NULL,
    positive_scores_only boolean DEFAULT true NOT NULL
);


--
-- Name: cached_dictionary_exports_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.cached_dictionary_exports_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: cached_dictionary_exports_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.cached_dictionary_exports_id_seq OWNED BY public.cached_dictionary_exports.id;


--
-- Name: collection_images; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.collection_images (
    collection_image_id integer NOT NULL,
    content_sha256 bytea NOT NULL,
    image_data bytea NOT NULL,
    mime_type text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: collection_images_collection_image_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.collection_images_collection_image_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: collection_images_collection_image_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.collection_images_collection_image_id_seq OWNED BY public.collection_images.collection_image_id;


--
-- Name: collection_item_images; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.collection_item_images (
    id integer NOT NULL,
    item_id integer NOT NULL,
    side text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    collection_image_id integer NOT NULL,
    CONSTRAINT collection_item_images_side_check CHECK ((side = ANY (ARRAY['front'::text, 'back'::text])))
);


--
-- Name: collection_item_images_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.collection_item_images_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: collection_item_images_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.collection_item_images_id_seq OWNED BY public.collection_item_images.id;


--
-- Name: collection_item_sounds; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.collection_item_sounds (
    id integer NOT NULL,
    item_id integer NOT NULL,
    sound_data bytea NOT NULL,
    mime_type text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: collection_item_sounds_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.collection_item_sounds_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: collection_item_sounds_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.collection_item_sounds_id_seq OWNED BY public.collection_item_sounds.id;


--
-- Name: collection_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.collection_items (
    item_id integer NOT NULL,
    collection_id integer NOT NULL,
    definition_id integer,
    notes text,
    added_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "position" integer DEFAULT 0 NOT NULL,
    free_content_front text,
    free_content_back text,
    auto_progress boolean DEFAULT true NOT NULL,
    langid integer,
    owner_user_id integer,
    license character varying(50),
    script character varying(4),
    is_original boolean DEFAULT true NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    canonical_form text,
    CONSTRAINT valid_content_check CHECK ((((definition_id IS NOT NULL) AND (free_content_front IS NULL) AND (free_content_back IS NULL)) OR ((definition_id IS NULL) AND (free_content_front IS NOT NULL) AND (free_content_back IS NOT NULL))))
);


--
-- Name: COLUMN collection_items.langid; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.collection_items.langid IS 'Language ID of the item content, references languages table.';


--
-- Name: COLUMN collection_items.owner_user_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.collection_items.owner_user_id IS 'User ID of the sentence owner, references users table.';


--
-- Name: COLUMN collection_items.license; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.collection_items.license IS 'License of the item content (e.g., CC BY 2.0 FR).';


--
-- Name: COLUMN collection_items.script; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.collection_items.script IS 'ISO 15924 script code (e.g., Latn, Cyrl).';


--
-- Name: COLUMN collection_items.is_original; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.collection_items.is_original IS 'TRUE if the sentence was not created as a direct translation of another.';


--
-- Name: COLUMN collection_items.updated_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.collection_items.updated_at IS 'Timestamp of the last update to the item.';


--
-- Name: collection_items_item_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.collection_items_item_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: collection_items_item_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.collection_items_item_id_seq OWNED BY public.collection_items.item_id;


--
-- Name: collections; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.collections (
    collection_id integer NOT NULL,
    user_id integer NOT NULL,
    name text NOT NULL,
    description text,
    is_public boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    cover_collection_image_id integer,
    auto_challenge_mix boolean DEFAULT false NOT NULL,
    challenge_mode text DEFAULT 'off'::text NOT NULL,
    CONSTRAINT collections_challenge_mode_check CHECK ((challenge_mode = ANY (ARRAY['off'::text, 'fixed'::text, 'auto'::text])))
);


--
-- Name: collections_collection_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.collections_collection_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: collections_collection_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.collections_collection_id_seq OWNED BY public.collections.collection_id;


--
-- Name: comment_activity_counters; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comment_activity_counters (
    comment_id integer NOT NULL,
    total_bookmarks bigint DEFAULT 0 NOT NULL,
    total_replies bigint DEFAULT 0 NOT NULL,
    total_opinions bigint DEFAULT 0 NOT NULL,
    last_activity_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    total_reactions bigint DEFAULT 0 NOT NULL
);


--
-- Name: comment_bookmarks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comment_bookmarks (
    comment_id integer NOT NULL,
    user_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: comment_counters; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comment_counters (
    comment_id integer NOT NULL,
    total_reactions bigint DEFAULT 0 NOT NULL,
    total_replies bigint DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: comment_media; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comment_media (
    media_id integer NOT NULL,
    comment_id integer,
    media_type text NOT NULL,
    media_data bytea,
    text_content text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT valid_media_check CHECK ((((media_type = 'image'::text) AND (media_data IS NOT NULL)) OR ((media_type = 'text'::text) AND (text_content IS NOT NULL)) OR ((media_type = 'header'::text) AND (text_content IS NOT NULL))))
);


--
-- Name: comment_media_media_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.comment_media_media_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: comment_media_media_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.comment_media_media_id_seq OWNED BY public.comment_media.media_id;


--
-- Name: comment_opinion_votes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comment_opinion_votes (
    opinion_id bigint NOT NULL,
    user_id integer NOT NULL,
    comment_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: comment_opinions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comment_opinions (
    id bigint NOT NULL,
    comment_id integer NOT NULL,
    user_id integer NOT NULL,
    opinion character varying(12) NOT NULL,
    votes integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: comment_opinions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.comment_opinions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: comment_opinions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.comment_opinions_id_seq OWNED BY public.comment_opinions.id;


--
-- Name: comment_reactions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comment_reactions (
    id integer NOT NULL,
    comment_id integer NOT NULL,
    user_id integer NOT NULL,
    reaction character varying(32) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: comment_reactions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.comment_reactions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: comment_reactions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.comment_reactions_id_seq OWNED BY public.comment_reactions.id;


--
-- Name: comments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comments (
    commentid integer NOT NULL,
    threadid integer NOT NULL,
    parentid integer,
    userid integer NOT NULL,
    commentnum integer NOT NULL,
    "time" integer NOT NULL,
    subject text,
    content jsonb,
    plain_content text,
    import_source text,
    import_ref jsonb
);


--
-- Name: comments_commentid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.comments_commentid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: comments_commentid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.comments_commentid_seq OWNED BY public.comments.commentid;


--
-- Name: threads; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.threads (
    threadid integer NOT NULL,
    valsiid integer,
    natlangwordid integer,
    definitionid integer,
    last_comment_id integer,
    last_comment_user_id integer,
    last_comment_time integer,
    last_comment_subject text,
    last_comment_content jsonb,
    total_comments integer DEFAULT 0,
    creator_user_id integer,
    creator_username character varying(64),
    first_comment_subject text,
    first_comment_content jsonb,
    target_user_id integer,
    definition_link_id integer,
    import_source text,
    import_ref jsonb,
    collection_id integer
);


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    userid integer NOT NULL,
    username character varying(64) NOT NULL,
    password text NOT NULL,
    email text NOT NULL,
    realname text,
    url text,
    personal text,
    votesize real DEFAULT 0.0,
    created_at timestamp with time zone NOT NULL,
    followers integer DEFAULT 0 NOT NULL,
    role text NOT NULL,
    email_confirmed boolean DEFAULT false NOT NULL,
    email_confirmation_token text,
    email_confirmation_sent_at timestamp with time zone,
    disabled boolean DEFAULT false NOT NULL,
    disabled_at timestamp with time zone,
    disabled_by integer,
    subscription_status public.user_subscription_status DEFAULT 'inactive'::public.user_subscription_status NOT NULL,
    paypal_customer_id text,
    oauth_signup boolean DEFAULT false,
    metadata jsonb DEFAULT '{"notifications_enabled": true}'::jsonb NOT NULL
);


--
-- Name: convenientcomments; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.convenientcomments AS
 SELECT c.commentid,
    c.threadid,
    c.parentid,
    c.userid,
    u.username,
    u.realname,
    c."time",
    c.subject,
    c.content,
    c.commentnum,
    cc.total_reactions,
    cc.total_replies,
    t.valsiid,
    t.definitionid,
    t.definition_link_id,
    t.collection_id
   FROM (((public.comments c
     JOIN public.users u ON ((c.userid = u.userid)))
     JOIN public.threads t ON ((c.threadid = t.threadid)))
     LEFT JOIN public.comment_counters cc ON ((c.commentid = cc.comment_id)));


--
-- Name: definitions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.definitions (
    langid integer NOT NULL,
    valsiid integer NOT NULL,
    definitionnum integer NOT NULL,
    definitionid integer NOT NULL,
    definition text NOT NULL,
    notes text,
    userid integer NOT NULL,
    "time" integer NOT NULL,
    selmaho text,
    jargon text,
    owner_only boolean DEFAULT false NOT NULL,
    etymology text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    embedding public.vector(384),
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    cached_username text,
    cached_langrealname text,
    cached_type_name text,
    cached_search_text text,
    cached_valsiword text,
    cached_rafsi text,
    cached_source_langid integer,
    cached_typeid smallint,
    cached_glosswords text,
    cached_decomposition text,
    rafsi text,
    cached_canonical_word text,
    cached_official_rafsi text
);


--
-- Name: languages; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.languages (
    langid integer NOT NULL,
    tag character varying(128) NOT NULL,
    englishname text NOT NULL,
    lojbanname text NOT NULL,
    realname text NOT NULL,
    forlojban text,
    url text
);


--
-- Name: valsi; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.valsi (
    valsiid integer NOT NULL,
    word text NOT NULL,
    typeid smallint NOT NULL,
    userid integer NOT NULL,
    "time" integer NOT NULL,
    rafsi text,
    source_langid integer DEFAULT 1 NOT NULL,
    cached_decomposition text,
    canonical_word text,
    related_expansion_key text,
    trivial_se_checked boolean DEFAULT false NOT NULL
);


--
-- Name: convenientdefinitions; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.convenientdefinitions AS
 SELECT nd.definitionid,
    l.realname AS langrealname,
    l.tag,
    l.langid,
    v.valsiid,
    v.word,
    nd.definition,
    nd.notes,
    u.username,
    u.userid,
    nd."time",
    nd.definitionnum,
    v.rafsi,
    nd.selmaho,
    nd.jargon,
    v.rafsi AS official_rafsi,
    nd.rafsi AS experimental_rafsi
   FROM (((public.definitions nd
     JOIN public.languages l ON ((nd.langid = l.langid)))
     JOIN public.valsi v ON ((nd.valsiid = v.valsiid)))
     JOIN public.users u ON ((nd.userid = u.userid)));


--
-- Name: etymology; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.etymology (
    etymologyid integer NOT NULL,
    valsiid integer NOT NULL,
    langid integer NOT NULL,
    content text,
    "time" integer NOT NULL,
    userid integer NOT NULL
);


--
-- Name: convenientetymology; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.convenientetymology AS
 SELECT e.etymologyid,
    v.word,
    e.valsiid,
    l.tag,
    l.realname,
    l.langid,
    e.content,
    e."time",
    u.username,
    u.userid
   FROM public.etymology e,
    public.valsi v,
    public.languages l,
    public.users u
  WHERE ((e.userid = u.userid) AND (e.langid = l.langid) AND (e.valsiid = v.valsiid));


--
-- Name: example; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.example (
    exampleid integer NOT NULL,
    valsiid integer NOT NULL,
    definitionid integer NOT NULL,
    examplenum integer NOT NULL,
    content text,
    "time" integer NOT NULL,
    userid integer NOT NULL
);


--
-- Name: convenientexamples; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.convenientexamples AS
 SELECT e.exampleid,
    v.word,
    e.valsiid,
    e.content,
    e."time",
    u.username,
    u.userid,
    e.examplenum,
    e.definitionid
   FROM public.valsi v,
    public.example e,
    public.users u
  WHERE ((v.valsiid = e.valsiid) AND (u.userid = e.userid));


--
-- Name: natlangwords; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.natlangwords (
    wordid integer NOT NULL,
    langid integer NOT NULL,
    word text NOT NULL,
    meaning text,
    meaningnum integer NOT NULL,
    userid integer NOT NULL,
    "time" integer NOT NULL,
    notes text
);


--
-- Name: convenientthreads; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.convenientthreads AS
 SELECT t.threadid,
    t.valsiid,
    v.word AS valsi,
    t.natlangwordid,
    nlw.word AS natlangword,
    l.tag,
    t.definitionid
   FROM public.threads t,
    public.valsi v,
    public.natlangwords nlw,
    public.languages l
  WHERE ((t.valsiid = v.valsiid) AND (t.natlangwordid = nlw.wordid) AND (nlw.langid = l.langid));


--
-- Name: valsitypes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.valsitypes (
    typeid smallint NOT NULL,
    descriptor character varying(128)
);


--
-- Name: convenientvalsi; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.convenientvalsi AS
 SELECT v.valsiid,
    v.word,
    t.descriptor AS type,
    v.typeid,
    u.username,
    v.userid,
    v."time",
    v.rafsi
   FROM public.valsi v,
    public.valsitypes t,
    public.users u
  WHERE ((v.typeid = t.typeid) AND (v.userid = u.userid));


--
-- Name: definition_images; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.definition_images (
    id integer NOT NULL,
    definition_id integer NOT NULL,
    image_data bytea NOT NULL,
    mime_type character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    description text,
    display_order integer DEFAULT 0 NOT NULL,
    created_by integer NOT NULL
);


--
-- Name: definition_images_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.definition_images_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: definition_images_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.definition_images_id_seq OWNED BY public.definition_images.id;


--
-- Name: definition_links; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.definition_links (
    id integer NOT NULL,
    definition_id integer NOT NULL,
    translation_id integer NOT NULL,
    created_by integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT definition_links_check CHECK ((definition_id <> translation_id))
);


--
-- Name: definition_links_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.definition_links_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: definition_links_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.definition_links_id_seq OWNED BY public.definition_links.id;


--
-- Name: definition_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.definition_versions (
    version_id integer NOT NULL,
    definition_id integer NOT NULL,
    langid integer NOT NULL,
    valsiid integer NOT NULL,
    definition text NOT NULL,
    notes text,
    selmaho text,
    jargon text,
    gloss_keywords jsonb DEFAULT '[]'::jsonb,
    place_keywords jsonb DEFAULT '[]'::jsonb,
    user_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    message text NOT NULL,
    owner_only boolean DEFAULT false NOT NULL,
    etymology text,
    rafsi text,
    mw_revid bigint,
    word text
);


--
-- Name: definition_versions_version_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.definition_versions_version_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: definition_versions_version_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.definition_versions_version_id_seq OWNED BY public.definition_versions.version_id;


--
-- Name: definitions_definitionid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.definitions_definitionid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: definitions_definitionid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.definitions_definitionid_seq OWNED BY public.definitions.definitionid;


--
-- Name: definitionvotes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.definitionvotes (
    valsiid integer NOT NULL,
    langid integer NOT NULL,
    definitionid integer NOT NULL,
    value real NOT NULL,
    userid integer NOT NULL,
    "time" integer NOT NULL
);


--
-- Name: dictionary; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.dictionary (
    id integer NOT NULL,
    w text NOT NULL,
    d text,
    n text,
    t text,
    s text,
    g text,
    q text,
    v text,
    r text,
    b text
);


--
-- Name: dictionary_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.dictionary_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: dictionary_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.dictionary_id_seq OWNED BY public.dictionary.id;


--
-- Name: etymology_backup; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.etymology_backup (
    etymologyid integer,
    valsiid integer,
    langid integer,
    content text,
    "time" integer,
    userid integer
);


--
-- Name: TABLE etymology_backup; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.etymology_backup IS 'Backup of etymology table created during migration on 2026-10-26 17:32:24.03294+03';


--
-- Name: etymology_etymologyid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.etymology_etymologyid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: etymology_etymologyid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.etymology_etymologyid_seq OWNED BY public.etymology.etymologyid;


--
-- Name: example_exampleid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.example_exampleid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: example_exampleid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.example_exampleid_seq OWNED BY public.example.exampleid;


--
-- Name: flashcards; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.flashcards (
    id integer NOT NULL,
    collection_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "position" integer DEFAULT 0 NOT NULL,
    item_id integer NOT NULL,
    direction public.flashcard_direction DEFAULT 'direct'::public.flashcard_direction NOT NULL,
    use_canonical_comparison boolean DEFAULT true
);


--
-- Name: flashcard_details; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.flashcard_details AS
 SELECT f.id,
    f.collection_id,
    f.created_at,
    f."position",
    f.item_id,
    f.direction,
    ci.notes,
    ci.auto_progress,
    ci.definition_id,
    v.word,
    d.definition,
    d.langid AS definition_language_id
   FROM (((public.flashcards f
     JOIN public.collection_items ci ON ((f.item_id = ci.item_id)))
     LEFT JOIN public.definitions d ON ((ci.definition_id = d.definitionid)))
     LEFT JOIN public.valsi v ON ((d.valsiid = v.valsiid)));


--
-- Name: flashcard_level_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.flashcard_level_items (
    level_id integer NOT NULL,
    flashcard_id integer NOT NULL,
    "position" integer DEFAULT 0 NOT NULL
);


--
-- Name: flashcard_levels; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.flashcard_levels (
    level_id integer NOT NULL,
    collection_id integer NOT NULL,
    name text NOT NULL,
    description text,
    min_cards integer DEFAULT 5 NOT NULL,
    min_success_rate double precision DEFAULT 0.8 NOT NULL,
    "position" integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: flashcard_levels_level_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.flashcard_levels_level_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: flashcard_levels_level_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.flashcard_levels_level_id_seq OWNED BY public.flashcard_levels.level_id;


--
-- Name: flashcard_quiz_options; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.flashcard_quiz_options (
    quiz_option_id integer NOT NULL,
    flashcard_id integer NOT NULL,
    correct_answer_text text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: flashcard_quiz_options_quiz_option_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.flashcard_quiz_options_quiz_option_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: flashcard_quiz_options_quiz_option_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.flashcard_quiz_options_quiz_option_id_seq OWNED BY public.flashcard_quiz_options.quiz_option_id;


--
-- Name: flashcard_review_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.flashcard_review_history (
    id integer NOT NULL,
    user_id integer NOT NULL,
    flashcard_id integer NOT NULL,
    rating integer NOT NULL,
    elapsed_days integer NOT NULL,
    scheduled_days integer NOT NULL,
    state jsonb NOT NULL,
    review_time timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    card_side text DEFAULT 'direct'::text NOT NULL,
    presented_type text
);


--
-- Name: flashcard_review_history_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.flashcard_review_history_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: flashcard_review_history_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.flashcard_review_history_id_seq OWNED BY public.flashcard_review_history.id;


--
-- Name: flashcards_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.flashcards_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: flashcards_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.flashcards_id_seq OWNED BY public.flashcards.id;


--
-- Name: follows; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.follows (
    follower_id integer NOT NULL,
    followee_id integer NOT NULL,
    follow_date timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT follows_check CHECK ((follower_id <> followee_id))
);


--
-- Name: hashtags; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.hashtags (
    id integer NOT NULL,
    tag character varying(255) NOT NULL
);


--
-- Name: hashtags_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.hashtags_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: hashtags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.hashtags_id_seq OWNED BY public.hashtags.id;


--
-- Name: keywordmapping; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.keywordmapping (
    natlangwordid integer NOT NULL,
    definitionid integer NOT NULL,
    place integer NOT NULL
);


--
-- Name: languages_langid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.languages_langid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: languages_langid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.languages_langid_seq OWNED BY public.languages.langid;


--
-- Name: level_prerequisites; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.level_prerequisites (
    level_id integer NOT NULL,
    prerequisite_id integer NOT NULL,
    CONSTRAINT level_prerequisites_check CHECK ((level_id <> prerequisite_id))
);


--
-- Name: mail_imported_files; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mail_imported_files (
    file_path text NOT NULL,
    message_id integer
);


--
-- Name: message_encryption_keys; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.message_encryption_keys (
    key_id bigint NOT NULL,
    thread_id bigint NOT NULL,
    user_id integer NOT NULL,
    encrypted_key text NOT NULL,
    key_algorithm character varying(50) DEFAULT 'kyber1024'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    key_version integer DEFAULT 1 NOT NULL
);


--
-- Name: message_encryption_keys_key_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.message_encryption_keys_key_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: message_encryption_keys_key_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.message_encryption_keys_key_id_seq OWNED BY public.message_encryption_keys.key_id;


--
-- Name: message_notifications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.message_notifications (
    notification_id bigint NOT NULL,
    user_id integer NOT NULL,
    thread_id bigint NOT NULL,
    message_id bigint NOT NULL,
    sender_id integer NOT NULL,
    notification_type character varying(20) DEFAULT 'new_message'::character varying NOT NULL,
    is_read boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    read_at timestamp with time zone,
    CONSTRAINT message_notifications_notification_type_check CHECK (((notification_type)::text = ANY ((ARRAY['new_message'::character varying, 'thread_added'::character varying, 'thread_removed'::character varying])::text[])))
);


--
-- Name: message_notifications_notification_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.message_notifications_notification_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: message_notifications_notification_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.message_notifications_notification_id_seq OWNED BY public.message_notifications.notification_id;


--
-- Name: message_spam_votes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.message_spam_votes (
    message_id integer NOT NULL,
    user_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: message_threads; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.message_threads (
    thread_id bigint NOT NULL,
    thread_name character varying(255),
    thread_type character varying(20) NOT NULL,
    created_by integer NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    max_participants integer DEFAULT 100 NOT NULL,
    last_message_at timestamp with time zone,
    last_message_preview text,
    message_count bigint DEFAULT 0 NOT NULL,
    CONSTRAINT message_threads_thread_type_check CHECK (((thread_type)::text = ANY ((ARRAY['direct'::character varying, 'group'::character varying])::text[])))
);


--
-- Name: message_threads_thread_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.message_threads_thread_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: message_threads_thread_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.message_threads_thread_id_seq OWNED BY public.message_threads.thread_id;


--
-- Name: messages; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.messages (
    id integer NOT NULL,
    message_id text,
    date text,
    subject text,
    from_address text,
    to_address text,
    content text,
    file_path text,
    cleaned_subject text,
    sent_at timestamp with time zone NOT NULL,
    parts_json jsonb
);


--
-- Name: messages_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.messages_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: messages_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.messages_id_seq OWNED BY public.messages.id;


--
-- Name: muplis; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.muplis (
    id integer NOT NULL,
    lojban text,
    english text
);


--
-- Name: muplis_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.muplis_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: muplis_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.muplis_id_seq OWNED BY public.muplis.id;


--
-- Name: muplis_update; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.muplis_update (
    id integer NOT NULL,
    last_update bigint
);


--
-- Name: muplis_update_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.muplis_update_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: muplis_update_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.muplis_update_id_seq OWNED BY public.muplis_update.id;


--
-- Name: natlangwordbestplaces; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.natlangwordbestplaces (
    wordid integer NOT NULL,
    definitionid integer NOT NULL,
    place integer NOT NULL,
    score integer NOT NULL
);


--
-- Name: natlangwords_wordid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.natlangwords_wordid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: natlangwords_wordid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.natlangwords_wordid_seq OWNED BY public.natlangwords.wordid;


--
-- Name: natlangwordvotes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.natlangwordvotes (
    natlangwordid integer NOT NULL,
    definitionid integer,
    place integer NOT NULL,
    userid integer NOT NULL,
    value integer NOT NULL,
    "time" integer NOT NULL
);


--
-- Name: oauth_accounts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_accounts (
    id integer NOT NULL,
    user_id integer NOT NULL,
    provider character varying(255) NOT NULL,
    provider_id character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);


--
-- Name: oauth_accounts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.oauth_accounts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: oauth_accounts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.oauth_accounts_id_seq OWNED BY public.oauth_accounts.id;


--
-- Name: oauth_login_transactions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_login_transactions (
    nonce text NOT NULL,
    provider text NOT NULL,
    pkce_verifier text NOT NULL,
    expires_at timestamp with time zone NOT NULL
);


--
-- Name: pages; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.pages (
    pagename text NOT NULL,
    version integer NOT NULL,
    "time" integer NOT NULL,
    userid integer NOT NULL,
    langid integer NOT NULL,
    content text,
    compressed boolean DEFAULT false,
    latest boolean DEFAULT true
);


--
-- Name: password_change_verifications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.password_change_verifications (
    id integer NOT NULL,
    user_id integer NOT NULL,
    verification_id text NOT NULL,
    verification_code text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    completed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: password_change_verifications_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.password_change_verifications_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: password_change_verifications_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.password_change_verifications_id_seq OWNED BY public.password_change_verifications.id;


--
-- Name: password_reset_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.password_reset_requests (
    id integer NOT NULL,
    email text NOT NULL,
    session_id text NOT NULL,
    token text NOT NULL,
    token_expiry bigint NOT NULL,
    created_at bigint NOT NULL,
    used boolean DEFAULT false,
    used_at bigint
);


--
-- Name: password_reset_requests_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.password_reset_requests_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: password_reset_requests_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.password_reset_requests_id_seq OWNED BY public.password_reset_requests.id;


--
-- Name: payment_audit_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.payment_audit_log (
    id integer NOT NULL,
    payment_id integer NOT NULL,
    user_id integer NOT NULL,
    event_type text NOT NULL,
    details text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: payment_audit_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.payment_audit_log_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: payment_audit_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.payment_audit_log_id_seq OWNED BY public.payment_audit_log.id;


--
-- Name: payments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.payments (
    id integer NOT NULL,
    user_id integer NOT NULL,
    provider public.payment_provider NOT NULL,
    provider_payment_id text NOT NULL,
    amount_cents bigint NOT NULL,
    currency text NOT NULL,
    status public.payment_status DEFAULT 'pending'::public.payment_status NOT NULL,
    metadata jsonb,
    idempotency_key text,
    error_message text,
    completed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payments_amount_cents_check CHECK ((amount_cents > 0))
);


--
-- Name: payments_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.payments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: payments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.payments_id_seq OWNED BY public.payments.id;


--
-- Name: paypal_subscriptions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.paypal_subscriptions (
    id integer NOT NULL,
    user_id integer NOT NULL,
    paypal_plan_id text NOT NULL,
    paypal_subscription_id text NOT NULL,
    status text NOT NULL,
    start_time timestamp with time zone NOT NULL,
    next_billing_time timestamp with time zone,
    last_payment_time timestamp with time zone,
    last_payment_amount_cents bigint,
    last_payment_currency text,
    cancel_reason text,
    cancelled_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: paypal_subscriptions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.paypal_subscriptions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: paypal_subscriptions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.paypal_subscriptions_id_seq OWNED BY public.paypal_subscriptions.id;


--
-- Name: permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.permissions (
    id integer NOT NULL,
    name character varying(50) NOT NULL,
    description text
);


--
-- Name: permissions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.permissions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: permissions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.permissions_id_seq OWNED BY public.permissions.id;


--
-- Name: post_hashtags; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.post_hashtags (
    post_id integer NOT NULL,
    hashtag_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: private_messages; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.private_messages (
    message_id bigint NOT NULL,
    thread_id bigint NOT NULL,
    sender_id integer NOT NULL,
    message_type character varying(20) DEFAULT 'text'::character varying NOT NULL,
    encrypted_content text NOT NULL,
    content_nonce bytea NOT NULL,
    sender_key_signature bytea,
    reply_to_message_id bigint,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    is_deleted boolean DEFAULT false NOT NULL,
    deleted_at timestamp with time zone,
    deleted_by integer,
    edit_count integer DEFAULT 0 NOT NULL,
    last_edited_at timestamp with time zone,
    CONSTRAINT private_messages_message_type_check CHECK (((message_type)::text = ANY ((ARRAY['text'::character varying, 'image'::character varying, 'file'::character varying, 'system'::character varying])::text[])))
);


--
-- Name: private_messages_message_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.private_messages_message_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: private_messages_message_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.private_messages_message_id_seq OWNED BY public.private_messages.message_id;


--
-- Name: role_permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.role_permissions (
    role text NOT NULL,
    permission_id integer NOT NULL
);


--
-- Name: sent_emails; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sent_emails (
    id bigint NOT NULL,
    to_address text NOT NULL,
    from_address text NOT NULL,
    subject text NOT NULL,
    text_body text NOT NULL,
    html_body text NOT NULL,
    params jsonb DEFAULT '{}'::jsonb NOT NULL,
    sent_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    initiated_by_user_id integer
);


--
-- Name: sent_emails_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.sent_emails_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: sent_emails_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.sent_emails_id_seq OWNED BY public.sent_emails.id;


--
-- Name: thread_participants; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.thread_participants (
    participant_id bigint NOT NULL,
    thread_id bigint NOT NULL,
    user_id integer NOT NULL,
    role character varying(20) DEFAULT 'member'::character varying NOT NULL,
    joined_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    left_at timestamp with time zone,
    is_active boolean DEFAULT true NOT NULL,
    last_read_at timestamp with time zone,
    unread_count bigint DEFAULT 0 NOT NULL,
    CONSTRAINT thread_participants_role_check CHECK (((role)::text = ANY ((ARRAY['admin'::character varying, 'member'::character varying])::text[])))
);


--
-- Name: thread_participants_participant_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.thread_participants_participant_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: thread_participants_participant_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.thread_participants_participant_id_seq OWNED BY public.thread_participants.participant_id;


--
-- Name: threads_threadid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.threads_threadid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: threads_threadid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.threads_threadid_seq OWNED BY public.threads.threadid;


--
-- Name: user_balances; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_balances (
    user_id integer NOT NULL,
    balance_cents bigint DEFAULT 0 NOT NULL,
    total_spent_cents bigint DEFAULT 0 NOT NULL,
    premium_expires_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT user_balances_balance_cents_check CHECK ((balance_cents >= 0))
);


--
-- Name: user_flashcard_progress; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_flashcard_progress (
    id integer NOT NULL,
    user_id integer NOT NULL,
    flashcard_id integer NOT NULL,
    ease_factor double precision DEFAULT 2.5 NOT NULL,
    "interval" integer DEFAULT 0 NOT NULL,
    review_count integer DEFAULT 0 NOT NULL,
    last_reviewed_at timestamp with time zone,
    next_review_at timestamp with time zone,
    status public.flashcard_status DEFAULT 'new'::public.flashcard_status NOT NULL,
    stability double precision DEFAULT 0.0 NOT NULL,
    difficulty double precision DEFAULT 0.0 NOT NULL,
    card_side text DEFAULT 'direct'::text NOT NULL,
    archived boolean DEFAULT false NOT NULL,
    auto_stage smallint DEFAULT 0 NOT NULL
);


--
-- Name: user_flashcard_progress_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_flashcard_progress_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_flashcard_progress_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_flashcard_progress_id_seq OWNED BY public.user_flashcard_progress.id;


--
-- Name: user_level_progress; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_level_progress (
    user_id integer NOT NULL,
    level_id integer NOT NULL,
    cards_completed integer DEFAULT 0 NOT NULL,
    correct_answers integer DEFAULT 0 NOT NULL,
    total_answers integer DEFAULT 0 NOT NULL,
    unlocked_at timestamp with time zone,
    completed_at timestamp with time zone,
    last_activity_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    completed_by_schedule boolean DEFAULT false NOT NULL
);


--
-- Name: user_message_blocks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_message_blocks (
    block_id bigint NOT NULL,
    blocker_id integer NOT NULL,
    blocked_id integer NOT NULL,
    blocked_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    reason text,
    is_active boolean DEFAULT true NOT NULL,
    CONSTRAINT user_message_blocks_check CHECK ((blocker_id <> blocked_id))
);


--
-- Name: user_message_blocks_block_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_message_blocks_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_message_blocks_block_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_message_blocks_block_id_seq OWNED BY public.user_message_blocks.block_id;


--
-- Name: user_notifications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_notifications (
    notification_id integer NOT NULL,
    user_id integer NOT NULL,
    notification_type text NOT NULL,
    message text NOT NULL,
    link text,
    valsi_id integer,
    actor_id integer,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    read_at timestamp with time zone,
    email_sent timestamp with time zone,
    definition_id integer
);


--
-- Name: user_notifications_notification_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_notifications_notification_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_notifications_notification_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_notifications_notification_id_seq OWNED BY public.user_notifications.notification_id;


--
-- Name: user_profile_images; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_profile_images (
    user_id integer NOT NULL,
    image_data bytea NOT NULL,
    mime_type text NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: user_quiz_answer_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_quiz_answer_history (
    history_id integer NOT NULL,
    user_id integer NOT NULL,
    flashcard_id integer NOT NULL,
    selected_option_text text NOT NULL,
    is_correct_selection boolean NOT NULL,
    presented_options jsonb,
    answered_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: user_quiz_answer_history_history_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_quiz_answer_history_history_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_quiz_answer_history_history_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_quiz_answer_history_history_id_seq OWNED BY public.user_quiz_answer_history.history_id;


--
-- Name: user_reaction_counts; Type: MATERIALIZED VIEW; Schema: public; Owner: -
--

CREATE MATERIALIZED VIEW public.user_reaction_counts AS
 SELECT user_id,
    comment_id,
    count(*) AS reaction_count
   FROM public.comment_reactions
  GROUP BY user_id, comment_id
  WITH NO DATA;


--
-- Name: user_search_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_search_history (
    id integer NOT NULL,
    user_id integer,
    search_query text NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    search_params jsonb
);


--
-- Name: user_search_history_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_search_history_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_search_history_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_search_history_id_seq OWNED BY public.user_search_history.id;


--
-- Name: user_session_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_session_events (
    id bigint NOT NULL,
    session_id bigint NOT NULL,
    event_type character varying(255) NOT NULL,
    event_timestamp timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    event_details jsonb
);


--
-- Name: user_session_events_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_session_events_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_session_events_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_session_events_id_seq OWNED BY public.user_session_events.id;


--
-- Name: user_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_sessions (
    id bigint NOT NULL,
    session_uuid uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id integer NOT NULL,
    started_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    ended_at timestamp with time zone,
    ip_address inet,
    user_agent text,
    last_active_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: user_sessions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_sessions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_sessions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_sessions_id_seq OWNED BY public.user_sessions.id;


--
-- Name: user_settings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_settings (
    user_id integer NOT NULL,
    optimal_retention double precision DEFAULT 0.9 NOT NULL,
    last_calculated timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: users_userid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.users_userid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: users_userid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.users_userid_seq OWNED BY public.users.userid;


--
-- Name: users_view; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users_view (
    userid integer,
    username character varying(64),
    votesize real
);


--
-- Name: valsi_sounds; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.valsi_sounds (
    id integer NOT NULL,
    valsi_id integer NOT NULL,
    sound_data bytea NOT NULL,
    mime_type text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: valsi_sounds_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.valsi_sounds_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: valsi_sounds_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.valsi_sounds_id_seq OWNED BY public.valsi_sounds.id;


--
-- Name: valsi_subscriptions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.valsi_subscriptions (
    subscription_id integer NOT NULL,
    valsi_id integer NOT NULL,
    user_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    unsubscribed boolean DEFAULT false NOT NULL,
    unsubscribed_at timestamp with time zone,
    trigger_type public.subscription_trigger NOT NULL,
    source_definition_id integer,
    source_comment_id integer
);


--
-- Name: valsi_subscriptions_subscription_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.valsi_subscriptions_subscription_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: valsi_subscriptions_subscription_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.valsi_subscriptions_subscription_id_seq OWNED BY public.valsi_subscriptions.subscription_id;


--
-- Name: valsi_valsiid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.valsi_valsiid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: valsi_valsiid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.valsi_valsiid_seq OWNED BY public.valsi.valsiid;


--
-- Name: wave_search_chunks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.wave_search_chunks (
    document_id bigint NOT NULL,
    chunk_index integer NOT NULL,
    content text NOT NULL,
    embedding public.vector(384) NOT NULL
);


--
-- Name: wave_search_documents; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.wave_search_documents (
    document_id bigint NOT NULL,
    kind text NOT NULL,
    source_id integer NOT NULL,
    source text NOT NULL,
    title text DEFAULT ''::text NOT NULL,
    body text DEFAULT ''::text NOT NULL,
    author text DEFAULT ''::text NOT NULL,
    edited_at timestamp with time zone,
    generation bigint DEFAULT 1 NOT NULL,
    embedding public.vector(384),
    search_simple tsvector GENERATED ALWAYS AS (((setweight(to_tsvector('simple'::regconfig, title), 'A'::"char") || setweight(to_tsvector('simple'::regconfig, body), 'B'::"char")) || setweight(to_tsvector('simple'::regconfig, author), 'C'::"char"))) STORED,
    search_english tsvector GENERATED ALWAYS AS ((setweight(to_tsvector('english'::regconfig, title), 'A'::"char") || setweight(to_tsvector('english'::regconfig, body), 'B'::"char"))) STORED,
    CONSTRAINT wave_search_documents_kind_check CHECK ((kind = ANY (ARRAY['comment'::text, 'mail'::text, 'wiki'::text, 'native_wiki'::text])))
);


--
-- Name: wave_search_documents_document_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.wave_search_documents_document_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: wave_search_documents_document_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.wave_search_documents_document_id_seq OWNED BY public.wave_search_documents.document_id;


--
-- Name: webrtc_signaling; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.webrtc_signaling (
    signal_id bigint NOT NULL,
    from_user_id integer NOT NULL,
    to_user_id integer NOT NULL,
    signal_type character varying(20) NOT NULL,
    signal_data jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at timestamp with time zone DEFAULT (CURRENT_TIMESTAMP + '00:05:00'::interval) NOT NULL,
    is_processed boolean DEFAULT false NOT NULL,
    CONSTRAINT webrtc_signaling_signal_type_check CHECK (((signal_type)::text = ANY ((ARRAY['offer'::character varying, 'answer'::character varying, 'ice-candidate'::character varying])::text[])))
);


--
-- Name: webrtc_signaling_signal_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.webrtc_signaling_signal_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: webrtc_signaling_signal_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.webrtc_signaling_signal_id_seq OWNED BY public.webrtc_signaling.signal_id;


--
-- Name: wiki_articles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.wiki_articles (
    id integer NOT NULL,
    page_id integer NOT NULL,
    namespace integer NOT NULL,
    title text NOT NULL,
    revision_id bigint,
    wikitext text NOT NULL,
    markdown text NOT NULL,
    plain_text text NOT NULL,
    is_redirect boolean DEFAULT false NOT NULL,
    last_edited timestamp with time zone,
    fetched_at timestamp with time zone DEFAULT now() NOT NULL,
    history_imported_until bigint
);


--
-- Name: wiki_articles_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.wiki_articles_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: wiki_articles_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.wiki_articles_id_seq OWNED BY public.wiki_articles.id;


--
-- Name: wiki_sync_state; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.wiki_sync_state (
    id integer DEFAULT 1 NOT NULL,
    last_full_sync timestamp with time zone,
    last_incremental_sync timestamp with time zone,
    CONSTRAINT wiki_sync_state_id_check CHECK ((id = 1))
);


--
-- Name: xrefs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.xrefs (
    srctype smallint NOT NULL,
    srcid integer NOT NULL,
    desttype smallint NOT NULL,
    destid integer NOT NULL
);


--
-- Name: balance_transactions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.balance_transactions ALTER COLUMN id SET DEFAULT nextval('public.balance_transactions_id_seq'::regclass);


--
-- Name: cached_dictionary_exports id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cached_dictionary_exports ALTER COLUMN id SET DEFAULT nextval('public.cached_dictionary_exports_id_seq'::regclass);


--
-- Name: collection_images collection_image_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_images ALTER COLUMN collection_image_id SET DEFAULT nextval('public.collection_images_collection_image_id_seq'::regclass);


--
-- Name: collection_item_images id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_images ALTER COLUMN id SET DEFAULT nextval('public.collection_item_images_id_seq'::regclass);


--
-- Name: collection_item_sounds id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_sounds ALTER COLUMN id SET DEFAULT nextval('public.collection_item_sounds_id_seq'::regclass);


--
-- Name: collection_items item_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_items ALTER COLUMN item_id SET DEFAULT nextval('public.collection_items_item_id_seq'::regclass);


--
-- Name: collections collection_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collections ALTER COLUMN collection_id SET DEFAULT nextval('public.collections_collection_id_seq'::regclass);


--
-- Name: comment_media media_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_media ALTER COLUMN media_id SET DEFAULT nextval('public.comment_media_media_id_seq'::regclass);


--
-- Name: comment_opinions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_opinions ALTER COLUMN id SET DEFAULT nextval('public.comment_opinions_id_seq'::regclass);


--
-- Name: comment_reactions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_reactions ALTER COLUMN id SET DEFAULT nextval('public.comment_reactions_id_seq'::regclass);


--
-- Name: comments commentid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments ALTER COLUMN commentid SET DEFAULT nextval('public.comments_commentid_seq'::regclass);


--
-- Name: definition_images id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_images ALTER COLUMN id SET DEFAULT nextval('public.definition_images_id_seq'::regclass);


--
-- Name: definition_links id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_links ALTER COLUMN id SET DEFAULT nextval('public.definition_links_id_seq'::regclass);


--
-- Name: definition_versions version_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_versions ALTER COLUMN version_id SET DEFAULT nextval('public.definition_versions_version_id_seq'::regclass);


--
-- Name: definitions definitionid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definitions ALTER COLUMN definitionid SET DEFAULT nextval('public.definitions_definitionid_seq'::regclass);


--
-- Name: dictionary id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dictionary ALTER COLUMN id SET DEFAULT nextval('public.dictionary_id_seq'::regclass);


--
-- Name: etymology etymologyid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.etymology ALTER COLUMN etymologyid SET DEFAULT nextval('public.etymology_etymologyid_seq'::regclass);


--
-- Name: example exampleid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.example ALTER COLUMN exampleid SET DEFAULT nextval('public.example_exampleid_seq'::regclass);


--
-- Name: flashcard_levels level_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_levels ALTER COLUMN level_id SET DEFAULT nextval('public.flashcard_levels_level_id_seq'::regclass);


--
-- Name: flashcard_quiz_options quiz_option_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_quiz_options ALTER COLUMN quiz_option_id SET DEFAULT nextval('public.flashcard_quiz_options_quiz_option_id_seq'::regclass);


--
-- Name: flashcard_review_history id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_review_history ALTER COLUMN id SET DEFAULT nextval('public.flashcard_review_history_id_seq'::regclass);


--
-- Name: flashcards id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcards ALTER COLUMN id SET DEFAULT nextval('public.flashcards_id_seq'::regclass);


--
-- Name: hashtags id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.hashtags ALTER COLUMN id SET DEFAULT nextval('public.hashtags_id_seq'::regclass);


--
-- Name: languages langid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.languages ALTER COLUMN langid SET DEFAULT nextval('public.languages_langid_seq'::regclass);


--
-- Name: message_encryption_keys key_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_encryption_keys ALTER COLUMN key_id SET DEFAULT nextval('public.message_encryption_keys_key_id_seq'::regclass);


--
-- Name: message_notifications notification_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_notifications ALTER COLUMN notification_id SET DEFAULT nextval('public.message_notifications_notification_id_seq'::regclass);


--
-- Name: message_threads thread_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_threads ALTER COLUMN thread_id SET DEFAULT nextval('public.message_threads_thread_id_seq'::regclass);


--
-- Name: messages id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.messages ALTER COLUMN id SET DEFAULT nextval('public.messages_id_seq'::regclass);


--
-- Name: muplis id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.muplis ALTER COLUMN id SET DEFAULT nextval('public.muplis_id_seq'::regclass);


--
-- Name: muplis_update id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.muplis_update ALTER COLUMN id SET DEFAULT nextval('public.muplis_update_id_seq'::regclass);


--
-- Name: natlangwords wordid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwords ALTER COLUMN wordid SET DEFAULT nextval('public.natlangwords_wordid_seq'::regclass);


--
-- Name: oauth_accounts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_accounts ALTER COLUMN id SET DEFAULT nextval('public.oauth_accounts_id_seq'::regclass);


--
-- Name: password_change_verifications id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_change_verifications ALTER COLUMN id SET DEFAULT nextval('public.password_change_verifications_id_seq'::regclass);


--
-- Name: password_reset_requests id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_reset_requests ALTER COLUMN id SET DEFAULT nextval('public.password_reset_requests_id_seq'::regclass);


--
-- Name: payment_audit_log id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_audit_log ALTER COLUMN id SET DEFAULT nextval('public.payment_audit_log_id_seq'::regclass);


--
-- Name: payments id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payments ALTER COLUMN id SET DEFAULT nextval('public.payments_id_seq'::regclass);


--
-- Name: paypal_subscriptions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.paypal_subscriptions ALTER COLUMN id SET DEFAULT nextval('public.paypal_subscriptions_id_seq'::regclass);


--
-- Name: permissions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.permissions ALTER COLUMN id SET DEFAULT nextval('public.permissions_id_seq'::regclass);


--
-- Name: private_messages message_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.private_messages ALTER COLUMN message_id SET DEFAULT nextval('public.private_messages_message_id_seq'::regclass);


--
-- Name: sent_emails id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sent_emails ALTER COLUMN id SET DEFAULT nextval('public.sent_emails_id_seq'::regclass);


--
-- Name: thread_participants participant_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.thread_participants ALTER COLUMN participant_id SET DEFAULT nextval('public.thread_participants_participant_id_seq'::regclass);


--
-- Name: threads threadid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.threads ALTER COLUMN threadid SET DEFAULT nextval('public.threads_threadid_seq'::regclass);


--
-- Name: user_flashcard_progress id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_flashcard_progress ALTER COLUMN id SET DEFAULT nextval('public.user_flashcard_progress_id_seq'::regclass);


--
-- Name: user_message_blocks block_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_message_blocks ALTER COLUMN block_id SET DEFAULT nextval('public.user_message_blocks_block_id_seq'::regclass);


--
-- Name: user_notifications notification_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_notifications ALTER COLUMN notification_id SET DEFAULT nextval('public.user_notifications_notification_id_seq'::regclass);


--
-- Name: user_quiz_answer_history history_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_quiz_answer_history ALTER COLUMN history_id SET DEFAULT nextval('public.user_quiz_answer_history_history_id_seq'::regclass);


--
-- Name: user_search_history id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_search_history ALTER COLUMN id SET DEFAULT nextval('public.user_search_history_id_seq'::regclass);


--
-- Name: user_session_events id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_session_events ALTER COLUMN id SET DEFAULT nextval('public.user_session_events_id_seq'::regclass);


--
-- Name: user_sessions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions ALTER COLUMN id SET DEFAULT nextval('public.user_sessions_id_seq'::regclass);


--
-- Name: users userid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users ALTER COLUMN userid SET DEFAULT nextval('public.users_userid_seq'::regclass);


--
-- Name: valsi valsiid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi ALTER COLUMN valsiid SET DEFAULT nextval('public.valsi_valsiid_seq'::regclass);


--
-- Name: valsi_sounds id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi_sounds ALTER COLUMN id SET DEFAULT nextval('public.valsi_sounds_id_seq'::regclass);


--
-- Name: valsi_subscriptions subscription_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi_subscriptions ALTER COLUMN subscription_id SET DEFAULT nextval('public.valsi_subscriptions_subscription_id_seq'::regclass);


--
-- Name: wave_search_documents document_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.wave_search_documents ALTER COLUMN document_id SET DEFAULT nextval('public.wave_search_documents_document_id_seq'::regclass);


--
-- Name: webrtc_signaling signal_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webrtc_signaling ALTER COLUMN signal_id SET DEFAULT nextval('public.webrtc_signaling_signal_id_seq'::regclass);


--
-- Name: wiki_articles id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.wiki_articles ALTER COLUMN id SET DEFAULT nextval('public.wiki_articles_id_seq'::regclass);


--
-- Name: assistant_chats assistant_chats_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.assistant_chats
    ADD CONSTRAINT assistant_chats_pkey PRIMARY KEY (id);


--
-- Name: balance_transactions balance_transactions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.balance_transactions
    ADD CONSTRAINT balance_transactions_pkey PRIMARY KEY (id);


--
-- Name: cached_dictionary_exports cached_dictionary_exports_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cached_dictionary_exports
    ADD CONSTRAINT cached_dictionary_exports_pkey PRIMARY KEY (id);


--
-- Name: collection_images collection_images_content_sha256_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_images
    ADD CONSTRAINT collection_images_content_sha256_key UNIQUE (content_sha256);


--
-- Name: collection_images collection_images_pkey1; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_images
    ADD CONSTRAINT collection_images_pkey1 PRIMARY KEY (collection_image_id);


--
-- Name: collection_item_images collection_item_images_item_id_side_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_images
    ADD CONSTRAINT collection_item_images_item_id_side_key UNIQUE (item_id, side);


--
-- Name: collection_item_images collection_item_images_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_images
    ADD CONSTRAINT collection_item_images_pkey PRIMARY KEY (id);


--
-- Name: collection_item_sounds collection_item_sounds_item_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_sounds
    ADD CONSTRAINT collection_item_sounds_item_id_key UNIQUE (item_id);


--
-- Name: collection_item_sounds collection_item_sounds_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_sounds
    ADD CONSTRAINT collection_item_sounds_pkey PRIMARY KEY (id);


--
-- Name: collection_items collection_items_collection_id_definition_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_items
    ADD CONSTRAINT collection_items_collection_id_definition_id_key UNIQUE (collection_id, definition_id);


--
-- Name: collection_items collection_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_items
    ADD CONSTRAINT collection_items_pkey PRIMARY KEY (item_id);


--
-- Name: collections collections_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collections
    ADD CONSTRAINT collections_pkey PRIMARY KEY (collection_id);


--
-- Name: comment_activity_counters comment_activity_counters_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_activity_counters
    ADD CONSTRAINT comment_activity_counters_pkey PRIMARY KEY (comment_id);


--
-- Name: comment_bookmarks comment_bookmarks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_bookmarks
    ADD CONSTRAINT comment_bookmarks_pkey PRIMARY KEY (comment_id, user_id);


--
-- Name: comment_counters comment_counters_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_counters
    ADD CONSTRAINT comment_counters_pkey PRIMARY KEY (comment_id);


--
-- Name: comment_media comment_media_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_media
    ADD CONSTRAINT comment_media_pkey PRIMARY KEY (media_id);


--
-- Name: comment_opinion_votes comment_opinion_votes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_opinion_votes
    ADD CONSTRAINT comment_opinion_votes_pkey PRIMARY KEY (user_id, comment_id, opinion_id);


--
-- Name: comment_opinions comment_opinions_comment_id_opinion_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_opinions
    ADD CONSTRAINT comment_opinions_comment_id_opinion_key UNIQUE (comment_id, opinion);


--
-- Name: comment_opinions comment_opinions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_opinions
    ADD CONSTRAINT comment_opinions_pkey PRIMARY KEY (id);


--
-- Name: comment_reactions comment_reactions_comment_id_user_id_reaction_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_reactions
    ADD CONSTRAINT comment_reactions_comment_id_user_id_reaction_key UNIQUE (comment_id, user_id, reaction);


--
-- Name: comment_reactions comment_reactions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_reactions
    ADD CONSTRAINT comment_reactions_pkey PRIMARY KEY (id);


--
-- Name: comments comments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_pkey PRIMARY KEY (commentid);


--
-- Name: definition_images definition_images_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_images
    ADD CONSTRAINT definition_images_pkey PRIMARY KEY (id);


--
-- Name: definition_links definition_links_definition_id_translation_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_links
    ADD CONSTRAINT definition_links_definition_id_translation_id_key UNIQUE (definition_id, translation_id);


--
-- Name: definition_links definition_links_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_links
    ADD CONSTRAINT definition_links_pkey PRIMARY KEY (id);


--
-- Name: definition_versions definition_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_versions
    ADD CONSTRAINT definition_versions_pkey PRIMARY KEY (version_id);


--
-- Name: definitions definitions_langid_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definitions
    ADD CONSTRAINT definitions_langid_key UNIQUE (langid, valsiid, definitionnum);


--
-- Name: definitions definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definitions
    ADD CONSTRAINT definitions_pkey PRIMARY KEY (definitionid);


--
-- Name: definitionvotes definitionvotes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definitionvotes
    ADD CONSTRAINT definitionvotes_pkey PRIMARY KEY (valsiid, langid, userid, definitionid);


--
-- Name: dictionary dictionary_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dictionary
    ADD CONSTRAINT dictionary_pkey PRIMARY KEY (id);


--
-- Name: etymology etymology_etymologyid_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.etymology
    ADD CONSTRAINT etymology_etymologyid_key UNIQUE (etymologyid);


--
-- Name: example example_exampleid_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.example
    ADD CONSTRAINT example_exampleid_key UNIQUE (exampleid);


--
-- Name: flashcard_level_items flashcard_level_items_level_id_position_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_level_items
    ADD CONSTRAINT flashcard_level_items_level_id_position_key UNIQUE (level_id, "position");


--
-- Name: flashcard_level_items flashcard_level_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_level_items
    ADD CONSTRAINT flashcard_level_items_pkey PRIMARY KEY (level_id, flashcard_id);


--
-- Name: flashcard_levels flashcard_levels_collection_id_position_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_levels
    ADD CONSTRAINT flashcard_levels_collection_id_position_key UNIQUE (collection_id, "position");


--
-- Name: flashcard_levels flashcard_levels_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_levels
    ADD CONSTRAINT flashcard_levels_pkey PRIMARY KEY (level_id);


--
-- Name: flashcard_quiz_options flashcard_quiz_options_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_quiz_options
    ADD CONSTRAINT flashcard_quiz_options_pkey PRIMARY KEY (quiz_option_id);


--
-- Name: flashcard_review_history flashcard_review_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_review_history
    ADD CONSTRAINT flashcard_review_history_pkey PRIMARY KEY (id);


--
-- Name: flashcards flashcards_collection_item_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcards
    ADD CONSTRAINT flashcards_collection_item_unique UNIQUE (collection_id, item_id);


--
-- Name: flashcards flashcards_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcards
    ADD CONSTRAINT flashcards_pkey PRIMARY KEY (id);


--
-- Name: follows follows_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.follows
    ADD CONSTRAINT follows_pkey PRIMARY KEY (follower_id, followee_id);


--
-- Name: hashtags hashtags_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.hashtags
    ADD CONSTRAINT hashtags_pkey PRIMARY KEY (id);


--
-- Name: hashtags hashtags_tag_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.hashtags
    ADD CONSTRAINT hashtags_tag_key UNIQUE (tag);


--
-- Name: keywordmapping keywordmapping_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.keywordmapping
    ADD CONSTRAINT keywordmapping_pkey PRIMARY KEY (natlangwordid, definitionid, place);


--
-- Name: languages languages_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.languages
    ADD CONSTRAINT languages_pkey PRIMARY KEY (langid);


--
-- Name: level_prerequisites level_prerequisites_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.level_prerequisites
    ADD CONSTRAINT level_prerequisites_pkey PRIMARY KEY (level_id, prerequisite_id);


--
-- Name: mail_imported_files mail_imported_files_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mail_imported_files
    ADD CONSTRAINT mail_imported_files_pkey PRIMARY KEY (file_path);


--
-- Name: message_encryption_keys message_encryption_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_encryption_keys
    ADD CONSTRAINT message_encryption_keys_pkey PRIMARY KEY (key_id);


--
-- Name: message_encryption_keys message_encryption_keys_thread_id_user_id_key_version_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_encryption_keys
    ADD CONSTRAINT message_encryption_keys_thread_id_user_id_key_version_key UNIQUE (thread_id, user_id, key_version);


--
-- Name: message_notifications message_notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_notifications
    ADD CONSTRAINT message_notifications_pkey PRIMARY KEY (notification_id);


--
-- Name: message_spam_votes message_spam_votes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_spam_votes
    ADD CONSTRAINT message_spam_votes_pkey PRIMARY KEY (message_id, user_id);


--
-- Name: message_threads message_threads_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_threads
    ADD CONSTRAINT message_threads_pkey PRIMARY KEY (thread_id);


--
-- Name: messages messages_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_pkey PRIMARY KEY (id);


--
-- Name: muplis muplis_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.muplis
    ADD CONSTRAINT muplis_pkey PRIMARY KEY (id);


--
-- Name: muplis_update muplis_update_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.muplis_update
    ADD CONSTRAINT muplis_update_pkey PRIMARY KEY (id);


--
-- Name: natlangwordbestplaces natlangwordbestplaces_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwordbestplaces
    ADD CONSTRAINT natlangwordbestplaces_pkey PRIMARY KEY (wordid);


--
-- Name: natlangwords natlangwords_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwords
    ADD CONSTRAINT natlangwords_pkey PRIMARY KEY (wordid);


--
-- Name: natlangwords natlangwords_unique_langid_word_meaning; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwords
    ADD CONSTRAINT natlangwords_unique_langid_word_meaning UNIQUE (langid, word, meaning);


--
-- Name: natlangwordvotes natlangwordvotes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwordvotes
    ADD CONSTRAINT natlangwordvotes_pkey PRIMARY KEY (natlangwordid, userid);


--
-- Name: oauth_accounts oauth_accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_accounts
    ADD CONSTRAINT oauth_accounts_pkey PRIMARY KEY (id);


--
-- Name: oauth_accounts oauth_accounts_provider_provider_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_accounts
    ADD CONSTRAINT oauth_accounts_provider_provider_id_key UNIQUE (provider, provider_id);


--
-- Name: oauth_login_transactions oauth_login_transactions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_login_transactions
    ADD CONSTRAINT oauth_login_transactions_pkey PRIMARY KEY (nonce);


--
-- Name: password_change_verifications one_active_verification_per_user; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_change_verifications
    ADD CONSTRAINT one_active_verification_per_user UNIQUE (user_id, verification_id) DEFERRABLE INITIALLY DEFERRED;


--
-- Name: pages pages_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_pkey PRIMARY KEY (pagename, version, langid);


--
-- Name: password_change_verifications password_change_verifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_change_verifications
    ADD CONSTRAINT password_change_verifications_pkey PRIMARY KEY (id);


--
-- Name: password_change_verifications password_change_verifications_verification_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_change_verifications
    ADD CONSTRAINT password_change_verifications_verification_id_key UNIQUE (verification_id);


--
-- Name: password_reset_requests password_reset_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_reset_requests
    ADD CONSTRAINT password_reset_requests_pkey PRIMARY KEY (id);


--
-- Name: password_reset_requests password_reset_requests_session_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_reset_requests
    ADD CONSTRAINT password_reset_requests_session_id_key UNIQUE (session_id);


--
-- Name: payment_audit_log payment_audit_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_audit_log
    ADD CONSTRAINT payment_audit_log_pkey PRIMARY KEY (id);


--
-- Name: payments payments_idempotency_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payments
    ADD CONSTRAINT payments_idempotency_key_key UNIQUE (idempotency_key);


--
-- Name: payments payments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payments
    ADD CONSTRAINT payments_pkey PRIMARY KEY (id);


--
-- Name: payments payments_provider_provider_payment_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payments
    ADD CONSTRAINT payments_provider_provider_payment_id_key UNIQUE (provider, provider_payment_id);


--
-- Name: paypal_subscriptions paypal_subscriptions_paypal_subscription_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.paypal_subscriptions
    ADD CONSTRAINT paypal_subscriptions_paypal_subscription_id_key UNIQUE (paypal_subscription_id);


--
-- Name: paypal_subscriptions paypal_subscriptions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.paypal_subscriptions
    ADD CONSTRAINT paypal_subscriptions_pkey PRIMARY KEY (id);


--
-- Name: permissions permissions_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.permissions
    ADD CONSTRAINT permissions_name_key UNIQUE (name);


--
-- Name: permissions permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.permissions
    ADD CONSTRAINT permissions_pkey PRIMARY KEY (id);


--
-- Name: post_hashtags post_hashtags_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.post_hashtags
    ADD CONSTRAINT post_hashtags_pkey PRIMARY KEY (post_id, hashtag_id);


--
-- Name: private_messages private_messages_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.private_messages
    ADD CONSTRAINT private_messages_pkey PRIMARY KEY (message_id);


--
-- Name: role_permissions role_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.role_permissions
    ADD CONSTRAINT role_permissions_pkey PRIMARY KEY (role, permission_id);


--
-- Name: sent_emails sent_emails_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sent_emails
    ADD CONSTRAINT sent_emails_pkey PRIMARY KEY (id);


--
-- Name: thread_participants thread_participants_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.thread_participants
    ADD CONSTRAINT thread_participants_pkey PRIMARY KEY (participant_id);


--
-- Name: thread_participants thread_participants_thread_id_user_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.thread_participants
    ADD CONSTRAINT thread_participants_thread_id_user_id_key UNIQUE (thread_id, user_id);


--
-- Name: threads threads_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.threads
    ADD CONSTRAINT threads_pkey PRIMARY KEY (threadid);


--
-- Name: messages unique_file_path; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT unique_file_path UNIQUE (file_path);


--
-- Name: flashcards unique_position_per_collection; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcards
    ADD CONSTRAINT unique_position_per_collection UNIQUE (collection_id, "position");


--
-- Name: user_balances user_balances_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_balances
    ADD CONSTRAINT user_balances_pkey PRIMARY KEY (user_id);


--
-- Name: user_flashcard_progress user_flashcard_progress_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_flashcard_progress
    ADD CONSTRAINT user_flashcard_progress_pkey PRIMARY KEY (id);


--
-- Name: user_flashcard_progress user_flashcard_progress_unique_side; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_flashcard_progress
    ADD CONSTRAINT user_flashcard_progress_unique_side UNIQUE (user_id, flashcard_id, card_side);


--
-- Name: user_level_progress user_level_progress_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_level_progress
    ADD CONSTRAINT user_level_progress_pkey PRIMARY KEY (user_id, level_id);


--
-- Name: user_message_blocks user_message_blocks_blocker_id_blocked_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_message_blocks
    ADD CONSTRAINT user_message_blocks_blocker_id_blocked_id_key UNIQUE (blocker_id, blocked_id);


--
-- Name: user_message_blocks user_message_blocks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_message_blocks
    ADD CONSTRAINT user_message_blocks_pkey PRIMARY KEY (block_id);


--
-- Name: user_notifications user_notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_notifications
    ADD CONSTRAINT user_notifications_pkey PRIMARY KEY (notification_id);


--
-- Name: user_profile_images user_profile_images_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_profile_images
    ADD CONSTRAINT user_profile_images_pkey PRIMARY KEY (user_id);


--
-- Name: user_quiz_answer_history user_quiz_answer_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_quiz_answer_history
    ADD CONSTRAINT user_quiz_answer_history_pkey PRIMARY KEY (history_id);


--
-- Name: user_search_history user_search_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_search_history
    ADD CONSTRAINT user_search_history_pkey PRIMARY KEY (id);


--
-- Name: user_session_events user_session_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_session_events
    ADD CONSTRAINT user_session_events_pkey PRIMARY KEY (id);


--
-- Name: user_sessions user_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT user_sessions_pkey PRIMARY KEY (id);


--
-- Name: user_settings user_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_settings
    ADD CONSTRAINT user_settings_pkey PRIMARY KEY (user_id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (userid);


--
-- Name: users users_username_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);


--
-- Name: valsi valsi_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi
    ADD CONSTRAINT valsi_pkey PRIMARY KEY (valsiid);


--
-- Name: valsi_sounds valsi_sounds_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi_sounds
    ADD CONSTRAINT valsi_sounds_pkey PRIMARY KEY (id);


--
-- Name: valsi_sounds valsi_sounds_valsi_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi_sounds
    ADD CONSTRAINT valsi_sounds_valsi_id_key UNIQUE (valsi_id);


--
-- Name: valsi_subscriptions valsi_subscriptions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi_subscriptions
    ADD CONSTRAINT valsi_subscriptions_pkey PRIMARY KEY (subscription_id);


--
-- Name: valsi_subscriptions valsi_subscriptions_valsi_id_user_id_trigger_type_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi_subscriptions
    ADD CONSTRAINT valsi_subscriptions_valsi_id_user_id_trigger_type_key UNIQUE (valsi_id, user_id, trigger_type);


--
-- Name: valsitypes valsitypes_descriptor_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsitypes
    ADD CONSTRAINT valsitypes_descriptor_key UNIQUE (descriptor);


--
-- Name: valsitypes valsitypes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsitypes
    ADD CONSTRAINT valsitypes_pkey PRIMARY KEY (typeid);


--
-- Name: wave_search_chunks wave_search_chunks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.wave_search_chunks
    ADD CONSTRAINT wave_search_chunks_pkey PRIMARY KEY (document_id, chunk_index);


--
-- Name: wave_search_documents wave_search_documents_kind_source_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.wave_search_documents
    ADD CONSTRAINT wave_search_documents_kind_source_id_key UNIQUE (kind, source_id);


--
-- Name: wave_search_documents wave_search_documents_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.wave_search_documents
    ADD CONSTRAINT wave_search_documents_pkey PRIMARY KEY (document_id);


--
-- Name: webrtc_signaling webrtc_signaling_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webrtc_signaling
    ADD CONSTRAINT webrtc_signaling_pkey PRIMARY KEY (signal_id);


--
-- Name: wiki_articles wiki_articles_namespace_title_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.wiki_articles
    ADD CONSTRAINT wiki_articles_namespace_title_key UNIQUE (namespace, title);


--
-- Name: wiki_articles wiki_articles_page_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.wiki_articles
    ADD CONSTRAINT wiki_articles_page_id_key UNIQUE (page_id);


--
-- Name: wiki_articles wiki_articles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.wiki_articles
    ADD CONSTRAINT wiki_articles_pkey PRIMARY KEY (id);


--
-- Name: wiki_sync_state wiki_sync_state_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.wiki_sync_state
    ADD CONSTRAINT wiki_sync_state_pkey PRIMARY KEY (id);


--
-- Name: definition_versions_mw_revid_uidx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX definition_versions_mw_revid_uidx ON public.definition_versions USING btree (mw_revid) WHERE (mw_revid IS NOT NULL);


--
-- Name: idx_assistant_chats_user_updated; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_assistant_chats_user_updated ON public.assistant_chats USING btree (user_id, updated_at DESC);


--
-- Name: idx_balance_transactions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_balance_transactions_user ON public.balance_transactions USING btree (user_id);


--
-- Name: idx_balances_premium; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_balances_premium ON public.user_balances USING btree (premium_expires_at);


--
-- Name: idx_cached_exports_cleanup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cached_exports_cleanup ON public.cached_dictionary_exports USING btree (created_at);


--
-- Name: idx_cached_exports_full_key; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_cached_exports_full_key ON public.cached_dictionary_exports USING btree (language_tag, source_language_tag, format, positive_scores_only);


--
-- Name: idx_collection_images_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collection_images_created_at ON public.collection_images USING btree (created_at);


--
-- Name: idx_collection_item_images_cleanup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collection_item_images_cleanup ON public.collection_item_images USING btree (created_at);


--
-- Name: idx_collection_item_images_collection_image_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collection_item_images_collection_image_id ON public.collection_item_images USING btree (collection_image_id);


--
-- Name: idx_collection_item_images_item; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collection_item_images_item ON public.collection_item_images USING btree (item_id);


--
-- Name: idx_collection_item_sounds_item; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collection_item_sounds_item ON public.collection_item_sounds USING btree (item_id);


--
-- Name: idx_collection_items_collection; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collection_items_collection ON public.collection_items USING btree (collection_id);


--
-- Name: idx_collection_items_definition_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collection_items_definition_id ON public.collection_items USING btree (definition_id) WHERE (definition_id IS NOT NULL);


--
-- Name: idx_collection_items_position; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collection_items_position ON public.collection_items USING btree (collection_id, "position");


--
-- Name: idx_collections_public; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collections_public ON public.collections USING btree (is_public);


--
-- Name: idx_collections_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collections_user ON public.collections USING btree (user_id);


--
-- Name: idx_comment_bookmarks_comment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_bookmarks_comment ON public.comment_bookmarks USING btree (comment_id);


--
-- Name: idx_comment_bookmarks_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_bookmarks_user ON public.comment_bookmarks USING btree (user_id);


--
-- Name: idx_comment_counters_activity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_counters_activity ON public.comment_activity_counters USING btree (last_activity_at DESC);


--
-- Name: idx_comment_counters_bookmarks; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_counters_bookmarks ON public.comment_activity_counters USING btree (total_bookmarks DESC);


--
-- Name: idx_comment_counters_reactions; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_counters_reactions ON public.comment_activity_counters USING btree (total_reactions DESC);


--
-- Name: idx_comment_media_comment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_media_comment ON public.comment_media USING btree (comment_id);


--
-- Name: idx_comment_media_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_media_type ON public.comment_media USING btree (media_type);


--
-- Name: idx_comment_opinion_votes_opinion; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_opinion_votes_opinion ON public.comment_opinion_votes USING btree (opinion_id);


--
-- Name: idx_comment_opinion_votes_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_opinion_votes_user ON public.comment_opinion_votes USING btree (user_id);


--
-- Name: idx_comment_opinions_comment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_opinions_comment ON public.comment_opinions USING btree (comment_id);


--
-- Name: idx_comment_opinions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_opinions_user ON public.comment_opinions USING btree (user_id);


--
-- Name: idx_comment_reactions_comment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_reactions_comment ON public.comment_reactions USING btree (comment_id);


--
-- Name: idx_comment_reactions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_reactions_user ON public.comment_reactions USING btree (user_id);


--
-- Name: idx_comments_import_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_import_source ON public.comments USING btree (import_source) WHERE (import_source IS NOT NULL);


--
-- Name: idx_comments_threadid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_threadid ON public.comments USING btree (threadid);


--
-- Name: idx_comments_userid_threadid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_userid_threadid ON public.comments USING btree (userid, threadid);


--
-- Name: idx_def_links_definition; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_def_links_definition ON public.definition_links USING btree (definition_id);


--
-- Name: idx_def_links_translation; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_def_links_translation ON public.definition_links USING btree (translation_id);


--
-- Name: idx_definition_images_definition_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definition_images_definition_id ON public.definition_images USING btree (definition_id);


--
-- Name: idx_definition_versions_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definition_versions_created_at ON public.definition_versions USING btree (created_at);


--
-- Name: idx_definition_versions_definition_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definition_versions_definition_id ON public.definition_versions USING btree (definition_id);


--
-- Name: idx_definition_versions_user_id_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definition_versions_user_id_created_at ON public.definition_versions USING btree (user_id, created_at DESC);


--
-- Name: idx_definitions_cached_canonical_word; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_cached_canonical_word ON public.definitions USING btree (cached_canonical_word) WHERE (cached_canonical_word IS NOT NULL);


--
-- Name: idx_definitions_cached_glosswords_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_cached_glosswords_gin ON public.definitions USING gin (cached_glosswords public.gin_trgm_ops);


--
-- Name: idx_definitions_cached_search_text_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_cached_search_text_gin ON public.definitions USING gin (cached_search_text public.gin_trgm_ops);


--
-- Name: idx_definitions_cached_typeid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_cached_typeid ON public.definitions USING btree (cached_typeid) WHERE (cached_typeid IS NOT NULL);


--
-- Name: idx_definitions_cached_username; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_cached_username ON public.definitions USING btree (cached_username) WHERE (cached_username IS NOT NULL);


--
-- Name: idx_definitions_cached_valsiword; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_cached_valsiword ON public.definitions USING btree (cached_valsiword) WHERE (cached_valsiword IS NOT NULL);


--
-- Name: idx_definitions_cached_valsiword_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_cached_valsiword_gin ON public.definitions USING gin (cached_valsiword public.gin_trgm_ops) WHERE (cached_valsiword IS NOT NULL);


--
-- Name: idx_definitions_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_created_at ON public.definitions USING btree (created_at);


--
-- Name: idx_definitions_definition_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_definition_gin ON public.definitions USING gin (definition public.gin_trgm_ops);


--
-- Name: idx_definitions_embedding_vector; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_embedding_vector ON public.definitions USING hnsw (embedding public.vector_cosine_ops) WITH (m='16', ef_construction='64') WHERE (embedding IS NOT NULL);


--
-- Name: idx_definitions_etymology; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_etymology ON public.definitions USING btree (etymology) WHERE (etymology IS NOT NULL);


--
-- Name: idx_definitions_fast_search_covering; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_fast_search_covering ON public.definitions USING btree (definitionid, valsiid, langid, selmaho, created_at, cached_username, cached_langrealname, cached_type_name) WHERE (definition <> ''::text);


--
-- Name: idx_definitions_fast_search_where; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_fast_search_where ON public.definitions USING btree (cached_source_langid, langid) INCLUDE (definitionid, cached_valsiword, cached_rafsi, cached_canonical_word, cached_username, cached_langrealname, cached_type_name, selmaho, created_at) WHERE ((definition <> ''::text) AND (cached_search_text IS NOT NULL));


--
-- Name: idx_definitions_langid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_langid ON public.definitions USING btree (langid);


--
-- Name: idx_definitions_langid_valsiid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_langid_valsiid ON public.definitions USING btree (langid, valsiid);


--
-- Name: idx_definitions_notes_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_notes_gin ON public.definitions USING gin (notes public.gin_trgm_ops) WHERE (notes IS NOT NULL);


--
-- Name: idx_definitions_selmaho_cached; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_selmaho_cached ON public.definitions USING btree (selmaho) WHERE (selmaho IS NOT NULL);


--
-- Name: idx_definitions_selmaho_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_selmaho_gin ON public.definitions USING gin (selmaho public.gin_trgm_ops) WHERE (selmaho IS NOT NULL);


--
-- Name: idx_definitions_userid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_userid ON public.definitions USING btree (userid);


--
-- Name: idx_definitions_valsiid_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitions_valsiid_time ON public.definitions USING btree (valsiid, "time" DESC);


--
-- Name: idx_definitionvotes_defid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitionvotes_defid ON public.definitionvotes USING btree (definitionid);


--
-- Name: idx_definitionvotes_userid_defid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_definitionvotes_userid_defid ON public.definitionvotes USING btree (userid, definitionid);


--
-- Name: idx_dictionary_d; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_dictionary_d ON public.dictionary USING gin (d public.gin_trgm_ops);


--
-- Name: idx_dictionary_w; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_dictionary_w ON public.dictionary USING gin (w public.gin_trgm_ops);


--
-- Name: idx_flashcard_level_items_flashcard; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_flashcard_level_items_flashcard ON public.flashcard_level_items USING btree (flashcard_id);


--
-- Name: idx_flashcard_progress_archived; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_flashcard_progress_archived ON public.user_flashcard_progress USING btree (archived);


--
-- Name: idx_flashcard_progress_side; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_flashcard_progress_side ON public.user_flashcard_progress USING btree (flashcard_id, user_id, card_side);


--
-- Name: idx_flashcard_review_history_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_flashcard_review_history_lookup ON public.flashcard_review_history USING btree (user_id, flashcard_id, review_time);


--
-- Name: idx_flashcard_review_history_user_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_flashcard_review_history_user_time ON public.flashcard_review_history USING btree (user_id, review_time);


--
-- Name: idx_flashcards_collection; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_flashcards_collection ON public.flashcards USING btree (collection_id);


--
-- Name: idx_flashcards_item_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_flashcards_item_id ON public.flashcards USING btree (item_id);


--
-- Name: idx_flashcards_position; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_flashcards_position ON public.flashcards USING btree (collection_id, "position");


--
-- Name: idx_hashtags_tag; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_hashtags_tag ON public.hashtags USING btree (tag);


--
-- Name: idx_keywordmapping_definitionid_place; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_keywordmapping_definitionid_place ON public.keywordmapping USING btree (definitionid, place);


--
-- Name: idx_keywordmapping_natlangwordid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_keywordmapping_natlangwordid ON public.keywordmapping USING btree (natlangwordid);


--
-- Name: idx_level_prerequisites_level; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_level_prerequisites_level ON public.level_prerequisites USING btree (level_id);


--
-- Name: idx_level_progress_completion; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_level_progress_completion ON public.user_level_progress USING btree (user_id, level_id) WHERE (completed_at IS NOT NULL);


--
-- Name: idx_level_progress_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_level_progress_user ON public.user_level_progress USING btree (user_id);


--
-- Name: idx_message_encryption_keys_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_encryption_keys_active ON public.message_encryption_keys USING btree (is_active) WHERE (is_active = true);


--
-- Name: idx_message_encryption_keys_thread_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_encryption_keys_thread_user ON public.message_encryption_keys USING btree (thread_id, user_id);


--
-- Name: idx_message_notifications_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_notifications_created ON public.message_notifications USING btree (created_at DESC);


--
-- Name: idx_message_notifications_unread; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_notifications_unread ON public.message_notifications USING btree (user_id, is_read) WHERE (is_read = false);


--
-- Name: idx_message_notifications_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_notifications_user ON public.message_notifications USING btree (user_id);


--
-- Name: idx_message_spam_votes_message_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_spam_votes_message_id ON public.message_spam_votes USING btree (message_id);


--
-- Name: idx_message_threads_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_threads_active ON public.message_threads USING btree (is_active) WHERE (is_active = true);


--
-- Name: idx_message_threads_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_threads_created_at ON public.message_threads USING btree (created_at DESC);


--
-- Name: idx_message_threads_created_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_threads_created_by ON public.message_threads USING btree (created_by);


--
-- Name: idx_message_threads_last_message_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_threads_last_message_at ON public.message_threads USING btree (last_message_at DESC);


--
-- Name: idx_messages_cleaned_subject; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_messages_cleaned_subject ON public.messages USING btree (cleaned_subject);


--
-- Name: idx_messages_content; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_messages_content ON public.messages USING gin (content public.gin_trgm_ops);


--
-- Name: idx_messages_file_path; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_messages_file_path ON public.messages USING btree (file_path);


--
-- Name: idx_messages_message_id_trimmed; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_messages_message_id_trimmed ON public.messages USING btree (btrim(message_id));


--
-- Name: idx_messages_subject; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_messages_subject ON public.messages USING gin (subject public.gin_trgm_ops);


--
-- Name: idx_messages_thread_sent_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_messages_thread_sent_at ON public.messages USING btree (cleaned_subject, sent_at DESC, id DESC);


--
-- Name: idx_messages_without_message_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_messages_without_message_id ON public.messages USING btree (date, subject, from_address) WHERE (NULLIF(btrim(message_id), ''::text) IS NULL);


--
-- Name: idx_muplis_english; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_muplis_english ON public.muplis USING gin (english public.gin_trgm_ops);


--
-- Name: idx_muplis_lojban; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_muplis_lojban ON public.muplis USING gin (lojban public.gin_trgm_ops);


--
-- Name: idx_natlangwords_meaning_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_natlangwords_meaning_gin ON public.natlangwords USING gin (meaning public.gin_trgm_ops) WHERE (meaning IS NOT NULL);


--
-- Name: idx_natlangwords_word_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_natlangwords_word_gin ON public.natlangwords USING gin (word public.gin_trgm_ops);


--
-- Name: idx_natlangwords_wordid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_natlangwords_wordid ON public.natlangwords USING btree (wordid);


--
-- Name: idx_notifications_pending_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_pending_email ON public.user_notifications USING btree (email_sent) WHERE (email_sent IS NULL);


--
-- Name: idx_notifications_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_user ON public.user_notifications USING btree (user_id, created_at);


--
-- Name: idx_password_change_cleanup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_password_change_cleanup ON public.password_change_verifications USING btree (expires_at) WHERE (completed_at IS NULL);


--
-- Name: idx_password_change_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_password_change_user ON public.password_change_verifications USING btree (user_id) WHERE (completed_at IS NULL);


--
-- Name: idx_password_reset_email_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_password_reset_email_created ON public.password_reset_requests USING btree (email, created_at);


--
-- Name: idx_password_reset_session_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_password_reset_session_token ON public.password_reset_requests USING btree (session_id, token);


--
-- Name: idx_payment_audit_payment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payment_audit_payment ON public.payment_audit_log USING btree (payment_id);


--
-- Name: idx_payment_audit_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payment_audit_user ON public.payment_audit_log USING btree (user_id);


--
-- Name: idx_payments_idempotency; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payments_idempotency ON public.payments USING btree (idempotency_key);


--
-- Name: idx_payments_provider_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payments_provider_id ON public.payments USING btree (provider, provider_payment_id);


--
-- Name: idx_payments_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payments_status ON public.payments USING btree (status);


--
-- Name: idx_payments_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payments_user ON public.payments USING btree (user_id);


--
-- Name: idx_paypal_subscriptions_paypal_subscription_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_paypal_subscriptions_paypal_subscription_id ON public.paypal_subscriptions USING btree (paypal_subscription_id);


--
-- Name: idx_paypal_subscriptions_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_paypal_subscriptions_user_id ON public.paypal_subscriptions USING btree (user_id);


--
-- Name: idx_post_hashtags_hashtag; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_post_hashtags_hashtag ON public.post_hashtags USING btree (hashtag_id);


--
-- Name: idx_post_hashtags_post; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_post_hashtags_post ON public.post_hashtags USING btree (post_id);


--
-- Name: idx_private_messages_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_private_messages_active ON public.private_messages USING btree (thread_id, is_deleted) WHERE (is_deleted = false);


--
-- Name: idx_private_messages_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_private_messages_created_at ON public.private_messages USING btree (created_at DESC);


--
-- Name: idx_private_messages_sender_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_private_messages_sender_id ON public.private_messages USING btree (sender_id);


--
-- Name: idx_private_messages_thread_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_private_messages_thread_created ON public.private_messages USING btree (thread_id, created_at DESC);


--
-- Name: idx_private_messages_thread_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_private_messages_thread_id ON public.private_messages USING btree (thread_id);


--
-- Name: idx_search_history_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_search_history_created ON public.user_search_history USING btree (created_at);


--
-- Name: idx_search_history_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_search_history_user ON public.user_search_history USING btree (user_id);


--
-- Name: idx_sent_emails_initiated_by_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sent_emails_initiated_by_user_id ON public.sent_emails USING btree (initiated_by_user_id) WHERE (initiated_by_user_id IS NOT NULL);


--
-- Name: idx_sent_emails_sent_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sent_emails_sent_at ON public.sent_emails USING btree (sent_at DESC);


--
-- Name: idx_sent_emails_to_address; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sent_emails_to_address ON public.sent_emails USING btree (to_address);


--
-- Name: idx_thread_participants_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_thread_participants_active ON public.thread_participants USING btree (is_active) WHERE (is_active = true);


--
-- Name: idx_thread_participants_thread_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_thread_participants_thread_id ON public.thread_participants USING btree (thread_id);


--
-- Name: idx_thread_participants_unread; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_thread_participants_unread ON public.thread_participants USING btree (user_id, unread_count DESC) WHERE (unread_count > 0);


--
-- Name: idx_thread_participants_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_thread_participants_user_id ON public.thread_participants USING btree (user_id);


--
-- Name: idx_threads_collection_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_threads_collection_id ON public.threads USING btree (collection_id) WHERE (collection_id IS NOT NULL);


--
-- Name: idx_threads_definition_link_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_threads_definition_link_id ON public.threads USING btree (definition_link_id);


--
-- Name: idx_threads_import_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_threads_import_source ON public.threads USING btree (import_source) WHERE (import_source IS NOT NULL);


--
-- Name: idx_threads_target_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_threads_target_user_id ON public.threads USING btree (target_user_id) WHERE (target_user_id IS NOT NULL);


--
-- Name: idx_threads_valsiid_defid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_threads_valsiid_defid ON public.threads USING btree (valsiid, definitionid);


--
-- Name: idx_unique_active_progress; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_unique_active_progress ON public.user_flashcard_progress USING btree (user_id, flashcard_id, card_side) WHERE (NOT archived);


--
-- Name: idx_user_message_blocks_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_message_blocks_active ON public.user_message_blocks USING btree (is_active) WHERE (is_active = true);


--
-- Name: idx_user_message_blocks_blocked; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_message_blocks_blocked ON public.user_message_blocks USING btree (blocked_id);


--
-- Name: idx_user_message_blocks_blocker; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_message_blocks_blocker ON public.user_message_blocks USING btree (blocker_id);


--
-- Name: idx_user_notifications_definition_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_notifications_definition_id ON public.user_notifications USING btree (definition_id);


--
-- Name: idx_user_progress_next_review; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_progress_next_review ON public.user_flashcard_progress USING btree (next_review_at);


--
-- Name: idx_user_progress_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_progress_status ON public.user_flashcard_progress USING btree (status);


--
-- Name: idx_user_progress_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_progress_user ON public.user_flashcard_progress USING btree (user_id);


--
-- Name: idx_user_quiz_answer_history_flashcard_selected_option; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_quiz_answer_history_flashcard_selected_option ON public.user_quiz_answer_history USING btree (flashcard_id, selected_option_text);


--
-- Name: idx_user_quiz_answer_history_user_flashcard; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_quiz_answer_history_user_flashcard ON public.user_quiz_answer_history USING btree (user_id, flashcard_id);


--
-- Name: idx_user_session_events_event_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_session_events_event_timestamp ON public.user_session_events USING btree (event_timestamp);


--
-- Name: idx_user_session_events_event_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_session_events_event_type ON public.user_session_events USING btree (event_type);


--
-- Name: idx_user_session_events_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_session_events_session_id ON public.user_session_events USING btree (session_id);


--
-- Name: idx_user_sessions_ended_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_ended_at ON public.user_sessions USING btree (ended_at);


--
-- Name: idx_user_sessions_last_active_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_last_active_at ON public.user_sessions USING btree (last_active_at);


--
-- Name: idx_user_sessions_session_uuid; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_user_sessions_session_uuid ON public.user_sessions USING btree (session_uuid);


--
-- Name: idx_user_sessions_started_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_started_at ON public.user_sessions USING btree (started_at);


--
-- Name: idx_user_sessions_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_user_id ON public.user_sessions USING btree (user_id);


--
-- Name: idx_user_settings_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_settings_user_id ON public.user_settings USING btree (user_id);


--
-- Name: idx_users_email_confirmed; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_email_confirmed ON public.users USING btree (email_confirmed);


--
-- Name: idx_users_email_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_email_token ON public.users USING btree (email_confirmation_token) WHERE (email_confirmation_token IS NOT NULL);


--
-- Name: idx_users_guest_role_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_guest_role_created ON public.users USING btree (created_at) WHERE (lower(role) = 'guest'::text);


--
-- Name: idx_users_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_role ON public.users USING btree (role);


--
-- Name: idx_valsi_dictionary_unique_word_source_lang; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_valsi_dictionary_unique_word_source_lang ON public.valsi USING btree (word, source_langid) WHERE (typeid <> 16);


--
-- Name: idx_valsi_lujvo_unchecked_canonical; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_lujvo_unchecked_canonical ON public.valsi USING btree (valsiid) WHERE ((typeid = 4) AND (source_langid = 1) AND (canonical_word IS NULL));


--
-- Name: idx_valsi_rafsi_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_rafsi_gin ON public.valsi USING gin (rafsi public.gin_trgm_ops) WHERE (rafsi IS NOT NULL);


--
-- Name: idx_valsi_related_expansion_key; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_related_expansion_key ON public.valsi USING btree (related_expansion_key) WHERE (related_expansion_key IS NOT NULL);


--
-- Name: idx_valsi_sounds_valsi; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_sounds_valsi ON public.valsi_sounds USING btree (valsi_id);


--
-- Name: idx_valsi_source_langid_word; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_source_langid_word ON public.valsi USING btree (source_langid, word);


--
-- Name: idx_valsi_subs_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_subs_user ON public.valsi_subscriptions USING btree (user_id);


--
-- Name: idx_valsi_subs_user_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_subs_user_active ON public.valsi_subscriptions USING btree (user_id, trigger_type) INCLUDE (valsi_id) WHERE (NOT unsubscribed);


--
-- Name: idx_valsi_subs_valsi_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_subs_valsi_user ON public.valsi_subscriptions USING btree (valsi_id, user_id);


--
-- Name: idx_valsi_trivial_se_unchecked; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_trivial_se_unchecked ON public.valsi USING btree (valsiid) WHERE ((source_langid = 1) AND (typeid = ANY (ARRAY[1, 4, 7, 17])) AND (NOT trivial_se_checked));


--
-- Name: idx_valsi_typeid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_typeid ON public.valsi USING btree (typeid);


--
-- Name: idx_valsi_valsiid_source_langid_word; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_valsiid_source_langid_word ON public.valsi USING btree (valsiid, source_langid, word) INCLUDE (rafsi, typeid);


--
-- Name: idx_valsi_wiki_unique_word_source_lang; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_valsi_wiki_unique_word_source_lang ON public.valsi USING btree (word, source_langid) WHERE (typeid = 16);


--
-- Name: idx_valsi_word_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_valsi_word_gin ON public.valsi USING gin (word public.gin_trgm_ops);


--
-- Name: idx_webrtc_signaling_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webrtc_signaling_expires ON public.webrtc_signaling USING btree (expires_at);


--
-- Name: idx_webrtc_signaling_to_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webrtc_signaling_to_user ON public.webrtc_signaling USING btree (to_user_id, is_processed);


--
-- Name: languages_tag_lower_uidx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX languages_tag_lower_uidx ON public.languages USING btree (lower((tag)::text));


--
-- Name: natlangwordbestplaces_definitionid_key; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX natlangwordbestplaces_definitionid_key ON public.natlangwordbestplaces USING btree (definitionid);


--
-- Name: natlangwords_lower_word; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX natlangwords_lower_word ON public.natlangwords USING btree (lower(word));


--
-- Name: natlangwords_unique_langid_word_null; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX natlangwords_unique_langid_word_null ON public.natlangwords USING btree (langid, word) WHERE (meaning IS NULL);


--
-- Name: oauth_login_transactions_expiry; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX oauth_login_transactions_expiry ON public.oauth_login_transactions USING btree (expires_at);


--
-- Name: uq_flashcard_quiz_options_flashcard_id; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_flashcard_quiz_options_flashcard_id ON public.flashcard_quiz_options USING btree (flashcard_id);


--
-- Name: user_reaction_counts_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX user_reaction_counts_idx ON public.user_reaction_counts USING btree (user_id, comment_id);


--
-- Name: valsi_lower_word; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX valsi_lower_word ON public.valsi USING btree (lower(word));


--
-- Name: wave_chunk_embedding; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wave_chunk_embedding ON public.wave_search_chunks USING hnsw (embedding public.vector_cosine_ops);


--
-- Name: wave_search_author; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wave_search_author ON public.wave_search_documents USING gin (author public.gin_trgm_ops);


--
-- Name: wave_search_body; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wave_search_body ON public.wave_search_documents USING gin (body public.gin_trgm_ops);


--
-- Name: wave_search_embedding; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wave_search_embedding ON public.wave_search_documents USING hnsw (embedding public.vector_cosine_ops) WHERE (embedding IS NOT NULL);


--
-- Name: wave_search_english; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wave_search_english ON public.wave_search_documents USING gin (search_english);


--
-- Name: wave_search_mirror_identity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wave_search_mirror_identity ON public.wiki_articles USING btree (((page_id)::text));


--
-- Name: wave_search_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wave_search_pending ON public.wave_search_documents USING btree (document_id) WHERE (embedding IS NULL);


--
-- Name: wave_search_simple; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wave_search_simple ON public.wave_search_documents USING gin (search_simple);


--
-- Name: wave_search_title; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wave_search_title ON public.wave_search_documents USING gin (title public.gin_trgm_ops);


--
-- Name: wiki_articles_last_edited_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wiki_articles_last_edited_idx ON public.wiki_articles USING btree (last_edited DESC NULLS LAST);


--
-- Name: wiki_articles_plain_trgm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wiki_articles_plain_trgm ON public.wiki_articles USING gin (plain_text public.gin_trgm_ops);


--
-- Name: wiki_articles_title_trgm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX wiki_articles_title_trgm ON public.wiki_articles USING gin (title public.gin_trgm_ops);


--
-- Name: flashcard_review_history after_flashcard_review; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER after_flashcard_review AFTER INSERT ON public.flashcard_review_history FOR EACH ROW EXECUTE FUNCTION public.update_level_progress();


--
-- Name: collection_items cleanup_collection_item_images; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER cleanup_collection_item_images BEFORE DELETE ON public.collection_items FOR EACH ROW EXECUTE FUNCTION public.cleanup_item_images();


--
-- Name: collection_items cleanup_collection_item_sounds; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER cleanup_collection_item_sounds BEFORE DELETE ON public.collection_items FOR EACH ROW EXECUTE FUNCTION public.cleanup_item_sounds();


--
-- Name: definitions cleanup_definition_images; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER cleanup_definition_images BEFORE DELETE ON public.definitions FOR EACH ROW EXECUTE FUNCTION public.cleanup_orphaned_images();


--
-- Name: valsi cleanup_valsi_sounds_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER cleanup_valsi_sounds_trigger BEFORE DELETE ON public.valsi FOR EACH ROW EXECUTE FUNCTION public.cleanup_valsi_sounds();


--
-- Name: collection_item_images collection_item_images_after_delete_cleanup; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER collection_item_images_after_delete_cleanup AFTER DELETE ON public.collection_item_images FOR EACH ROW EXECUTE FUNCTION public.trg_collection_item_images_after_delete_cleanup();


--
-- Name: collection_item_images collection_item_images_after_update_cleanup; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER collection_item_images_after_update_cleanup AFTER UPDATE OF collection_image_id ON public.collection_item_images FOR EACH ROW EXECUTE FUNCTION public.trg_collection_item_images_after_update_cleanup();


--
-- Name: collections collections_before_delete_cover_cleanup; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER collections_before_delete_cover_cleanup BEFORE DELETE ON public.collections FOR EACH ROW EXECUTE FUNCTION public.trg_collections_before_delete_cover_cleanup();


--
-- Name: collections collections_cover_update_cleanup; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER collections_cover_update_cleanup AFTER UPDATE OF cover_collection_image_id ON public.collections FOR EACH ROW EXECUTE FUNCTION public.trg_collections_cover_update_cleanup();


--
-- Name: comments comment_stats_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER comment_stats_trigger AFTER INSERT OR DELETE OR UPDATE ON public.comments FOR EACH ROW EXECUTE FUNCTION public.update_thread_stats();


--
-- Name: comments comment_subscription_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER comment_subscription_trigger AFTER INSERT ON public.comments FOR EACH ROW EXECUTE FUNCTION public.auto_subscribe_on_content();


--
-- Name: definitions definition_subscription_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER definition_subscription_trigger AFTER INSERT ON public.definitions FOR EACH ROW EXECUTE FUNCTION public.auto_subscribe_on_content();


--
-- Name: users ensure_user_balance; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER ensure_user_balance AFTER INSERT ON public.users FOR EACH ROW EXECUTE FUNCTION public.create_user_balance();


--
-- Name: definitions invalidate_cached_exports_on_definition_change; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER invalidate_cached_exports_on_definition_change AFTER INSERT OR DELETE OR UPDATE ON public.definitions FOR EACH ROW EXECUTE FUNCTION public.trg_invalidate_cached_exports_for_definition();


--
-- Name: definitionvotes invalidate_cached_exports_on_vote_change; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER invalidate_cached_exports_on_vote_change AFTER INSERT OR DELETE OR UPDATE ON public.definitionvotes FOR EACH ROW EXECUTE FUNCTION public.trg_invalidate_cached_exports_for_vote();


--
-- Name: comments maintain_comment_reply_count; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER maintain_comment_reply_count AFTER INSERT OR DELETE ON public.comments FOR EACH ROW EXECUTE FUNCTION public.update_comment_reply_count();


--
-- Name: user_balances maintain_premium_status; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER maintain_premium_status BEFORE UPDATE ON public.user_balances FOR EACH ROW EXECUTE FUNCTION public.update_premium_status();


--
-- Name: private_messages maintain_thread_message_stats; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER maintain_thread_message_stats AFTER INSERT OR UPDATE ON public.private_messages FOR EACH ROW EXECUTE FUNCTION public.update_thread_message_stats();


--
-- Name: thread_participants mark_thread_messages_read; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER mark_thread_messages_read AFTER UPDATE OF last_read_at ON public.thread_participants FOR EACH ROW WHEN ((new.last_read_at IS DISTINCT FROM old.last_read_at)) EXECUTE FUNCTION public.mark_messages_as_read();


--
-- Name: messages message_sent_at_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER message_sent_at_trigger BEFORE INSERT OR UPDATE ON public.messages FOR EACH ROW EXECUTE FUNCTION public.set_message_sent_at();


--
-- Name: natlangwords natlangwords_cleanup_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER natlangwords_cleanup_trigger AFTER INSERT OR UPDATE ON public.natlangwords FOR EACH ROW EXECUTE FUNCTION public.trigger_cleanup_natlangwords();


--
-- Name: natlangwordvotes on_natlangwordvotes_delete; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER on_natlangwordvotes_delete AFTER DELETE ON public.natlangwordvotes FOR EACH ROW EXECUTE FUNCTION public.refresh_natlangwordbestplaces_for_delete();


--
-- Name: natlangwordvotes on_natlangwordvotes_insert; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER on_natlangwordvotes_insert AFTER INSERT ON public.natlangwordvotes FOR EACH ROW EXECUTE FUNCTION public.refresh_natlangwordbestplaces_for_upsert();


--
-- Name: natlangwordvotes on_natlangwordvotes_update; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER on_natlangwordvotes_update AFTER UPDATE ON public.natlangwordvotes FOR EACH ROW EXECUTE FUNCTION public.refresh_natlangwordbestplaces_for_upsert();


--
-- Name: role_permissions prevent_admin_role_deletion_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER prevent_admin_role_deletion_trigger BEFORE DELETE ON public.role_permissions FOR EACH ROW EXECUTE FUNCTION public.prevent_admin_role_deletion();


--
-- Name: collection_items set_timestamp_collection_items; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_timestamp_collection_items BEFORE UPDATE ON public.collection_items FOR EACH ROW EXECUTE FUNCTION public.trigger_set_collection_item_timestamp();


--
-- Name: paypal_subscriptions set_timestamp_paypal_subscriptions; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_timestamp_paypal_subscriptions BEFORE UPDATE ON public.paypal_subscriptions FOR EACH ROW EXECUTE FUNCTION public.trigger_set_timestamp();


--
-- Name: permissions trg_sync_admin_after_permission_changes; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_sync_admin_after_permission_changes AFTER INSERT OR DELETE OR UPDATE ON public.permissions FOR EACH STATEMENT EXECUTE FUNCTION public.sync_admin_permissions();


--
-- Name: role_permissions trg_sync_admin_after_role_perms_changes; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_sync_admin_after_role_perms_changes AFTER DELETE OR UPDATE ON public.role_permissions FOR EACH ROW WHEN ((old.role = 'admin'::text)) EXECUTE FUNCTION public.sync_admin_permissions();


--
-- Name: valsi trigger_invalidate_valsi_cached_decomposition; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_invalidate_valsi_cached_decomposition BEFORE UPDATE ON public.valsi FOR EACH ROW EXECUTE FUNCTION public.invalidate_valsi_cached_decomposition();


--
-- Name: valsi trigger_invalidate_valsi_canonical_word; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_invalidate_valsi_canonical_word BEFORE UPDATE ON public.valsi FOR EACH ROW EXECUTE FUNCTION public.invalidate_valsi_canonical_word();


--
-- Name: valsi trigger_invalidate_valsi_trivial_se; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_invalidate_valsi_trivial_se BEFORE UPDATE ON public.valsi FOR EACH ROW EXECUTE FUNCTION public.invalidate_valsi_trivial_se();


--
-- Name: definitions trigger_rafsi_dependents_definition; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_rafsi_dependents_definition AFTER INSERT OR DELETE OR UPDATE OF rafsi, valsiid ON public.definitions FOR EACH ROW EXECUTE FUNCTION public.invalidate_rafsi_dependents();


--
-- Name: valsi trigger_rafsi_dependents_valsi; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_rafsi_dependents_valsi AFTER INSERT OR DELETE OR UPDATE OF rafsi, word, typeid, source_langid ON public.valsi FOR EACH ROW EXECUTE FUNCTION public.invalidate_rafsi_dependents();


--
-- Name: definitions trigger_sync_definition_cache; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_sync_definition_cache AFTER INSERT OR UPDATE OF definition, notes, selmaho, userid, langid, valsiid, rafsi ON public.definitions FOR EACH ROW EXECUTE FUNCTION public.sync_definition_cache_fields();


--
-- Name: keywordmapping trigger_sync_definition_cache_from_keywords; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_sync_definition_cache_from_keywords AFTER INSERT OR DELETE OR UPDATE ON public.keywordmapping FOR EACH ROW EXECUTE FUNCTION public.sync_definition_cache_fields();


--
-- Name: languages trigger_sync_definition_cache_from_languages; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_sync_definition_cache_from_languages AFTER UPDATE OF realname ON public.languages FOR EACH ROW EXECUTE FUNCTION public.sync_definition_cache_fields();


--
-- Name: natlangwords trigger_sync_definition_cache_from_natlangwords; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_sync_definition_cache_from_natlangwords AFTER DELETE OR UPDATE OF word, meaning ON public.natlangwords FOR EACH ROW EXECUTE FUNCTION public.sync_definition_cache_fields();


--
-- Name: users trigger_sync_definition_cache_from_users; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_sync_definition_cache_from_users AFTER UPDATE OF username ON public.users FOR EACH ROW EXECUTE FUNCTION public.sync_definition_cache_fields();


--
-- Name: valsi trigger_sync_definition_cache_from_valsi; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_sync_definition_cache_from_valsi AFTER UPDATE OF word, rafsi, typeid, source_langid ON public.valsi FOR EACH ROW EXECUTE FUNCTION public.sync_definition_cache_fields();


--
-- Name: valsitypes trigger_sync_definition_cache_from_valsitypes; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_sync_definition_cache_from_valsitypes AFTER UPDATE OF descriptor ON public.valsitypes FOR EACH ROW EXECUTE FUNCTION public.sync_definition_cache_fields();


--
-- Name: messages update_cleaned_subject; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_cleaned_subject BEFORE INSERT OR UPDATE ON public.messages FOR EACH ROW EXECUTE FUNCTION public.update_cleaned_subject();


--
-- Name: comment_activity_counters update_comment_activity_timestamp; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_comment_activity_timestamp BEFORE UPDATE ON public.comment_activity_counters FOR EACH ROW EXECUTE FUNCTION public.update_comment_activity_timestamp();


--
-- Name: comment_counters update_comment_counter_timestamp; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_comment_counter_timestamp BEFORE UPDATE ON public.comment_counters FOR EACH ROW EXECUTE FUNCTION public.update_comment_counter_timestamp();


--
-- Name: follows update_followers_count; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_followers_count AFTER INSERT OR DELETE ON public.follows FOR EACH ROW EXECUTE FUNCTION public.update_followers_count();


--
-- Name: message_threads update_message_threads_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_message_threads_updated_at BEFORE UPDATE ON public.message_threads FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: comments update_plain_content; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_plain_content BEFORE INSERT OR UPDATE OF content ON public.comments FOR EACH ROW EXECUTE FUNCTION public.extract_plain_content();


--
-- Name: private_messages update_private_messages_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_private_messages_updated_at BEFORE UPDATE ON public.private_messages FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: comment_reactions update_reaction_count; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_reaction_count AFTER INSERT OR DELETE ON public.comment_reactions FOR EACH ROW EXECUTE FUNCTION public.update_reaction_count();


--
-- Name: comment_reactions update_reaction_counts; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_reaction_counts AFTER INSERT OR DELETE ON public.comment_reactions FOR EACH STATEMENT EXECUTE FUNCTION public.refresh_reaction_counts();


--
-- Name: users wave_search_author_sync; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER wave_search_author_sync AFTER UPDATE OF username ON public.users FOR EACH ROW EXECUTE FUNCTION public.wave_search_author_sync();


--
-- Name: comments wave_search_comment_sync; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER wave_search_comment_sync AFTER INSERT OR DELETE OR UPDATE ON public.comments FOR EACH ROW EXECUTE FUNCTION public.wave_search_comment_sync();


--
-- Name: definitions wave_search_definition_sync; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER wave_search_definition_sync AFTER INSERT OR DELETE OR UPDATE OF definition, valsiid, metadata ON public.definitions FOR EACH ROW EXECUTE FUNCTION public.wave_search_definition_sync();


--
-- Name: messages wave_search_mail_sync; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER wave_search_mail_sync AFTER INSERT OR DELETE OR UPDATE ON public.messages FOR EACH ROW EXECUTE FUNCTION public.wave_search_mail_sync();


--
-- Name: valsi wave_search_valsi_sync; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER wave_search_valsi_sync AFTER UPDATE OF word, typeid ON public.valsi FOR EACH ROW EXECUTE FUNCTION public.wave_search_valsi_sync();


--
-- Name: wiki_articles wave_search_wiki_sync; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER wave_search_wiki_sync AFTER INSERT OR DELETE OR UPDATE ON public.wiki_articles FOR EACH ROW EXECUTE FUNCTION public.wave_search_wiki_sync();


--
-- Name: keywordmapping $1; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.keywordmapping
    ADD CONSTRAINT "$1" FOREIGN KEY (natlangwordid) REFERENCES public.natlangwords(wordid);


--
-- Name: natlangwordvotes $1; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwordvotes
    ADD CONSTRAINT "$1" FOREIGN KEY (natlangwordid) REFERENCES public.natlangwords(wordid);


--
-- Name: valsi $1; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi
    ADD CONSTRAINT "$1" FOREIGN KEY (typeid) REFERENCES public.valsitypes(typeid);


--
-- Name: keywordmapping $2; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.keywordmapping
    ADD CONSTRAINT "$2" FOREIGN KEY (definitionid) REFERENCES public.definitions(definitionid);


--
-- Name: natlangwordvotes $2; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwordvotes
    ADD CONSTRAINT "$2" FOREIGN KEY (definitionid) REFERENCES public.definitions(definitionid);


--
-- Name: natlangwordvotes $3; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwordvotes
    ADD CONSTRAINT "$3" FOREIGN KEY (userid) REFERENCES public.users(userid);


--
-- Name: assistant_chats assistant_chats_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.assistant_chats
    ADD CONSTRAINT assistant_chats_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: balance_transactions balance_transactions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.balance_transactions
    ADD CONSTRAINT balance_transactions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: collection_item_images collection_item_images_collection_image_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_images
    ADD CONSTRAINT collection_item_images_collection_image_id_fkey FOREIGN KEY (collection_image_id) REFERENCES public.collection_images(collection_image_id);


--
-- Name: collection_item_images collection_item_images_item_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_images
    ADD CONSTRAINT collection_item_images_item_id_fkey FOREIGN KEY (item_id) REFERENCES public.collection_items(item_id) ON DELETE CASCADE;


--
-- Name: collection_item_sounds collection_item_sounds_item_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_sounds
    ADD CONSTRAINT collection_item_sounds_item_id_fkey FOREIGN KEY (item_id) REFERENCES public.collection_items(item_id) ON DELETE CASCADE;


--
-- Name: collection_items collection_items_collection_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_items
    ADD CONSTRAINT collection_items_collection_id_fkey FOREIGN KEY (collection_id) REFERENCES public.collections(collection_id);


--
-- Name: collection_items collection_items_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_items
    ADD CONSTRAINT collection_items_definition_id_fkey FOREIGN KEY (definition_id) REFERENCES public.definitions(definitionid);


--
-- Name: collection_items collection_items_langid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_items
    ADD CONSTRAINT collection_items_langid_fkey FOREIGN KEY (langid) REFERENCES public.languages(langid);


--
-- Name: collection_items collection_items_owner_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_items
    ADD CONSTRAINT collection_items_owner_user_id_fkey FOREIGN KEY (owner_user_id) REFERENCES public.users(userid);


--
-- Name: collections collections_cover_collection_image_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collections
    ADD CONSTRAINT collections_cover_collection_image_id_fkey FOREIGN KEY (cover_collection_image_id) REFERENCES public.collection_images(collection_image_id);


--
-- Name: collections collections_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collections
    ADD CONSTRAINT collections_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: comment_activity_counters comment_activity_counters_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_activity_counters
    ADD CONSTRAINT comment_activity_counters_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(commentid) ON DELETE CASCADE;


--
-- Name: comment_bookmarks comment_bookmarks_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_bookmarks
    ADD CONSTRAINT comment_bookmarks_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(commentid) ON DELETE CASCADE;


--
-- Name: comment_bookmarks comment_bookmarks_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_bookmarks
    ADD CONSTRAINT comment_bookmarks_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: comment_counters comment_counters_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_counters
    ADD CONSTRAINT comment_counters_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(commentid) ON DELETE CASCADE;


--
-- Name: comment_media comment_media_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_media
    ADD CONSTRAINT comment_media_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(commentid) ON DELETE CASCADE;


--
-- Name: comment_opinion_votes comment_opinion_votes_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_opinion_votes
    ADD CONSTRAINT comment_opinion_votes_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(commentid) ON DELETE CASCADE;


--
-- Name: comment_opinion_votes comment_opinion_votes_opinion_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_opinion_votes
    ADD CONSTRAINT comment_opinion_votes_opinion_id_fkey FOREIGN KEY (opinion_id) REFERENCES public.comment_opinions(id) ON DELETE CASCADE;


--
-- Name: comment_opinion_votes comment_opinion_votes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_opinion_votes
    ADD CONSTRAINT comment_opinion_votes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: comment_opinions comment_opinions_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_opinions
    ADD CONSTRAINT comment_opinions_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(commentid) ON DELETE CASCADE;


--
-- Name: comment_opinions comment_opinions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_opinions
    ADD CONSTRAINT comment_opinions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: comment_reactions comment_reactions_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_reactions
    ADD CONSTRAINT comment_reactions_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(commentid) ON DELETE CASCADE;


--
-- Name: comment_reactions comment_reactions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_reactions
    ADD CONSTRAINT comment_reactions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: comments comments_parentid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_parentid FOREIGN KEY (parentid) REFERENCES public.comments(commentid) MATCH FULL;


--
-- Name: comments comments_threadid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_threadid FOREIGN KEY (threadid) REFERENCES public.threads(threadid) MATCH FULL;


--
-- Name: comments comments_userid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_userid FOREIGN KEY (userid) REFERENCES public.users(userid) MATCH FULL;


--
-- Name: definition_images definition_images_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_images
    ADD CONSTRAINT definition_images_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(userid);


--
-- Name: definition_images definition_images_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_images
    ADD CONSTRAINT definition_images_definition_id_fkey FOREIGN KEY (definition_id) REFERENCES public.definitions(definitionid) ON DELETE CASCADE;


--
-- Name: definition_links definition_links_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_links
    ADD CONSTRAINT definition_links_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(userid);


--
-- Name: definition_links definition_links_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_links
    ADD CONSTRAINT definition_links_definition_id_fkey FOREIGN KEY (definition_id) REFERENCES public.definitions(definitionid) ON DELETE CASCADE;


--
-- Name: definition_links definition_links_translation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_links
    ADD CONSTRAINT definition_links_translation_id_fkey FOREIGN KEY (translation_id) REFERENCES public.definitions(definitionid) ON DELETE CASCADE;


--
-- Name: definition_versions definition_versions_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_versions
    ADD CONSTRAINT definition_versions_definition_id_fkey FOREIGN KEY (definition_id) REFERENCES public.definitions(definitionid);


--
-- Name: definition_versions definition_versions_langid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_versions
    ADD CONSTRAINT definition_versions_langid_fkey FOREIGN KEY (langid) REFERENCES public.languages(langid);


--
-- Name: definition_versions definition_versions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_versions
    ADD CONSTRAINT definition_versions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: definition_versions definition_versions_valsiid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definition_versions
    ADD CONSTRAINT definition_versions_valsiid_fkey FOREIGN KEY (valsiid) REFERENCES public.valsi(valsiid);


--
-- Name: definitions definitions_langid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definitions
    ADD CONSTRAINT definitions_langid FOREIGN KEY (langid) REFERENCES public.languages(langid) MATCH FULL;


--
-- Name: definitions definitions_userid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definitions
    ADD CONSTRAINT definitions_userid FOREIGN KEY (userid) REFERENCES public.users(userid) MATCH FULL;


--
-- Name: definitions definitions_valsiid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definitions
    ADD CONSTRAINT definitions_valsiid FOREIGN KEY (valsiid) REFERENCES public.valsi(valsiid) MATCH FULL;


--
-- Name: definitionvotes definitionvotes_definitionid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definitionvotes
    ADD CONSTRAINT definitionvotes_definitionid FOREIGN KEY (definitionid) REFERENCES public.definitions(definitionid) MATCH FULL;


--
-- Name: definitionvotes definitionvotes_langid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definitionvotes
    ADD CONSTRAINT definitionvotes_langid FOREIGN KEY (langid) REFERENCES public.languages(langid) MATCH FULL;


--
-- Name: definitionvotes definitionvotes_userid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definitionvotes
    ADD CONSTRAINT definitionvotes_userid FOREIGN KEY (userid) REFERENCES public.users(userid) MATCH FULL;


--
-- Name: definitionvotes definitionvotes_valsiid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.definitionvotes
    ADD CONSTRAINT definitionvotes_valsiid FOREIGN KEY (valsiid) REFERENCES public.valsi(valsiid) MATCH FULL;


--
-- Name: etymology etymology_langid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.etymology
    ADD CONSTRAINT etymology_langid FOREIGN KEY (langid) REFERENCES public.languages(langid) MATCH FULL;


--
-- Name: etymology etymology_userid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.etymology
    ADD CONSTRAINT etymology_userid FOREIGN KEY (userid) REFERENCES public.users(userid) MATCH FULL;


--
-- Name: etymology etymology_valsiid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.etymology
    ADD CONSTRAINT etymology_valsiid FOREIGN KEY (valsiid) REFERENCES public.valsi(valsiid) MATCH FULL;


--
-- Name: example example_userid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.example
    ADD CONSTRAINT example_userid FOREIGN KEY (userid) REFERENCES public.users(userid) MATCH FULL;


--
-- Name: example example_valsiid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.example
    ADD CONSTRAINT example_valsiid FOREIGN KEY (valsiid) REFERENCES public.valsi(valsiid) MATCH FULL;


--
-- Name: message_spam_votes fk_message; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_spam_votes
    ADD CONSTRAINT fk_message FOREIGN KEY (message_id) REFERENCES public.messages(id) ON DELETE CASCADE;


--
-- Name: user_session_events fk_session; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_session_events
    ADD CONSTRAINT fk_session FOREIGN KEY (session_id) REFERENCES public.user_sessions(id) ON DELETE CASCADE;


--
-- Name: message_spam_votes fk_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_spam_votes
    ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: user_sessions fk_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: flashcard_level_items flashcard_level_items_flashcard_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_level_items
    ADD CONSTRAINT flashcard_level_items_flashcard_id_fkey FOREIGN KEY (flashcard_id) REFERENCES public.flashcards(id) ON DELETE CASCADE;


--
-- Name: flashcard_level_items flashcard_level_items_level_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_level_items
    ADD CONSTRAINT flashcard_level_items_level_id_fkey FOREIGN KEY (level_id) REFERENCES public.flashcard_levels(level_id) ON DELETE CASCADE;


--
-- Name: flashcard_levels flashcard_levels_collection_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_levels
    ADD CONSTRAINT flashcard_levels_collection_id_fkey FOREIGN KEY (collection_id) REFERENCES public.collections(collection_id) ON DELETE CASCADE;


--
-- Name: flashcard_quiz_options flashcard_quiz_options_flashcard_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_quiz_options
    ADD CONSTRAINT flashcard_quiz_options_flashcard_id_fkey FOREIGN KEY (flashcard_id) REFERENCES public.flashcards(id) ON DELETE CASCADE;


--
-- Name: flashcard_review_history flashcard_review_history_flashcard_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_review_history
    ADD CONSTRAINT flashcard_review_history_flashcard_id_fkey FOREIGN KEY (flashcard_id) REFERENCES public.flashcards(id);


--
-- Name: flashcard_review_history flashcard_review_history_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_review_history
    ADD CONSTRAINT flashcard_review_history_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: flashcard_review_history flashcard_review_history_user_progress_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcard_review_history
    ADD CONSTRAINT flashcard_review_history_user_progress_fkey FOREIGN KEY (user_id, flashcard_id, card_side) REFERENCES public.user_flashcard_progress(user_id, flashcard_id, card_side);


--
-- Name: flashcards flashcards_collection_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcards
    ADD CONSTRAINT flashcards_collection_id_fkey FOREIGN KEY (collection_id) REFERENCES public.collections(collection_id) ON DELETE CASCADE;


--
-- Name: flashcards flashcards_item_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.flashcards
    ADD CONSTRAINT flashcards_item_id_fkey FOREIGN KEY (item_id) REFERENCES public.collection_items(item_id);


--
-- Name: follows follows_followee_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.follows
    ADD CONSTRAINT follows_followee_id_fkey FOREIGN KEY (followee_id) REFERENCES public.users(userid);


--
-- Name: follows follows_follower_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.follows
    ADD CONSTRAINT follows_follower_id_fkey FOREIGN KEY (follower_id) REFERENCES public.users(userid);


--
-- Name: keywordmapping keywordmapping_defbestguessid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.keywordmapping
    ADD CONSTRAINT keywordmapping_defbestguessid FOREIGN KEY (definitionid) REFERENCES public.definitions(definitionid) MATCH FULL;


--
-- Name: keywordmapping keywordmapping_natlangwordid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.keywordmapping
    ADD CONSTRAINT keywordmapping_natlangwordid FOREIGN KEY (natlangwordid) REFERENCES public.natlangwords(wordid) MATCH FULL;


--
-- Name: level_prerequisites level_prerequisites_level_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.level_prerequisites
    ADD CONSTRAINT level_prerequisites_level_id_fkey FOREIGN KEY (level_id) REFERENCES public.flashcard_levels(level_id) ON DELETE CASCADE;


--
-- Name: level_prerequisites level_prerequisites_prerequisite_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.level_prerequisites
    ADD CONSTRAINT level_prerequisites_prerequisite_id_fkey FOREIGN KEY (prerequisite_id) REFERENCES public.flashcard_levels(level_id) ON DELETE CASCADE;


--
-- Name: mail_imported_files mail_imported_files_message_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mail_imported_files
    ADD CONSTRAINT mail_imported_files_message_id_fkey FOREIGN KEY (message_id) REFERENCES public.messages(id) ON DELETE CASCADE;


--
-- Name: message_encryption_keys message_encryption_keys_thread_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_encryption_keys
    ADD CONSTRAINT message_encryption_keys_thread_id_fkey FOREIGN KEY (thread_id) REFERENCES public.message_threads(thread_id) ON DELETE CASCADE;


--
-- Name: message_encryption_keys message_encryption_keys_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_encryption_keys
    ADD CONSTRAINT message_encryption_keys_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: message_notifications message_notifications_message_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_notifications
    ADD CONSTRAINT message_notifications_message_id_fkey FOREIGN KEY (message_id) REFERENCES public.private_messages(message_id) ON DELETE CASCADE;


--
-- Name: message_notifications message_notifications_sender_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_notifications
    ADD CONSTRAINT message_notifications_sender_id_fkey FOREIGN KEY (sender_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: message_notifications message_notifications_thread_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_notifications
    ADD CONSTRAINT message_notifications_thread_id_fkey FOREIGN KEY (thread_id) REFERENCES public.message_threads(thread_id) ON DELETE CASCADE;


--
-- Name: message_notifications message_notifications_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_notifications
    ADD CONSTRAINT message_notifications_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: message_threads message_threads_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_threads
    ADD CONSTRAINT message_threads_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: natlangwordbestplaces natlangwordbestplaces_definitionid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwordbestplaces
    ADD CONSTRAINT natlangwordbestplaces_definitionid FOREIGN KEY (definitionid) REFERENCES public.definitions(definitionid);


--
-- Name: natlangwordbestplaces natlangwordbestplaces_wordid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwordbestplaces
    ADD CONSTRAINT natlangwordbestplaces_wordid FOREIGN KEY (wordid) REFERENCES public.natlangwords(wordid);


--
-- Name: threads natlangwordid_natlangwordid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.threads
    ADD CONSTRAINT natlangwordid_natlangwordid FOREIGN KEY (natlangwordid) REFERENCES public.natlangwords(wordid) MATCH FULL;


--
-- Name: natlangwords natlangwords_langid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwords
    ADD CONSTRAINT natlangwords_langid FOREIGN KEY (langid) REFERENCES public.languages(langid) MATCH FULL;


--
-- Name: natlangwordvotes natlangwordvotes_natlangwordid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwordvotes
    ADD CONSTRAINT natlangwordvotes_natlangwordid FOREIGN KEY (natlangwordid) REFERENCES public.natlangwords(wordid) MATCH FULL;


--
-- Name: natlangwordvotes natlangwordvotes_voter; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.natlangwordvotes
    ADD CONSTRAINT natlangwordvotes_voter FOREIGN KEY (userid) REFERENCES public.users(userid) MATCH FULL;


--
-- Name: oauth_accounts oauth_accounts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_accounts
    ADD CONSTRAINT oauth_accounts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: pages pages_langid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_langid FOREIGN KEY (langid) REFERENCES public.languages(langid) MATCH FULL;


--
-- Name: pages pages_userid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_userid FOREIGN KEY (userid) REFERENCES public.users(userid) MATCH FULL;


--
-- Name: password_change_verifications password_change_verifications_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_change_verifications
    ADD CONSTRAINT password_change_verifications_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: payment_audit_log payment_audit_log_payment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_audit_log
    ADD CONSTRAINT payment_audit_log_payment_id_fkey FOREIGN KEY (payment_id) REFERENCES public.payments(id);


--
-- Name: payment_audit_log payment_audit_log_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_audit_log
    ADD CONSTRAINT payment_audit_log_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: payments payments_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payments
    ADD CONSTRAINT payments_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: paypal_subscriptions paypal_subscriptions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.paypal_subscriptions
    ADD CONSTRAINT paypal_subscriptions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: post_hashtags post_hashtags_hashtag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.post_hashtags
    ADD CONSTRAINT post_hashtags_hashtag_id_fkey FOREIGN KEY (hashtag_id) REFERENCES public.hashtags(id) ON DELETE CASCADE;


--
-- Name: post_hashtags post_hashtags_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.post_hashtags
    ADD CONSTRAINT post_hashtags_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.comments(commentid) ON DELETE CASCADE;


--
-- Name: private_messages private_messages_deleted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.private_messages
    ADD CONSTRAINT private_messages_deleted_by_fkey FOREIGN KEY (deleted_by) REFERENCES public.users(userid) ON DELETE SET NULL;


--
-- Name: private_messages private_messages_reply_to_message_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.private_messages
    ADD CONSTRAINT private_messages_reply_to_message_id_fkey FOREIGN KEY (reply_to_message_id) REFERENCES public.private_messages(message_id) ON DELETE SET NULL;


--
-- Name: private_messages private_messages_sender_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.private_messages
    ADD CONSTRAINT private_messages_sender_id_fkey FOREIGN KEY (sender_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: private_messages private_messages_thread_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.private_messages
    ADD CONSTRAINT private_messages_thread_id_fkey FOREIGN KEY (thread_id) REFERENCES public.message_threads(thread_id) ON DELETE CASCADE;


--
-- Name: role_permissions role_permissions_permission_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.role_permissions
    ADD CONSTRAINT role_permissions_permission_id_fkey FOREIGN KEY (permission_id) REFERENCES public.permissions(id) ON DELETE CASCADE;


--
-- Name: thread_participants thread_participants_thread_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.thread_participants
    ADD CONSTRAINT thread_participants_thread_id_fkey FOREIGN KEY (thread_id) REFERENCES public.message_threads(thread_id) ON DELETE CASCADE;


--
-- Name: thread_participants thread_participants_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.thread_participants
    ADD CONSTRAINT thread_participants_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: threads threads_collection_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.threads
    ADD CONSTRAINT threads_collection_id_fkey FOREIGN KEY (collection_id) REFERENCES public.collections(collection_id) ON DELETE CASCADE;


--
-- Name: threads threads_creator_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.threads
    ADD CONSTRAINT threads_creator_user_id_fkey FOREIGN KEY (creator_user_id) REFERENCES public.users(userid);


--
-- Name: threads threads_last_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.threads
    ADD CONSTRAINT threads_last_comment_id_fkey FOREIGN KEY (last_comment_id) REFERENCES public.comments(commentid);


--
-- Name: threads threads_last_comment_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.threads
    ADD CONSTRAINT threads_last_comment_user_id_fkey FOREIGN KEY (last_comment_user_id) REFERENCES public.users(userid);


--
-- Name: threads threads_target_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.threads
    ADD CONSTRAINT threads_target_user_id_fkey FOREIGN KEY (target_user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: threads threads_valsiid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.threads
    ADD CONSTRAINT threads_valsiid FOREIGN KEY (valsiid) REFERENCES public.valsi(valsiid) MATCH FULL;


--
-- Name: user_balances user_balances_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_balances
    ADD CONSTRAINT user_balances_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: user_flashcard_progress user_flashcard_progress_flashcard_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_flashcard_progress
    ADD CONSTRAINT user_flashcard_progress_flashcard_id_fkey FOREIGN KEY (flashcard_id) REFERENCES public.flashcards(id) ON DELETE CASCADE;


--
-- Name: user_flashcard_progress user_flashcard_progress_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_flashcard_progress
    ADD CONSTRAINT user_flashcard_progress_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: user_level_progress user_level_progress_level_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_level_progress
    ADD CONSTRAINT user_level_progress_level_id_fkey FOREIGN KEY (level_id) REFERENCES public.flashcard_levels(level_id) ON DELETE CASCADE;


--
-- Name: user_level_progress user_level_progress_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_level_progress
    ADD CONSTRAINT user_level_progress_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: user_message_blocks user_message_blocks_blocked_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_message_blocks
    ADD CONSTRAINT user_message_blocks_blocked_id_fkey FOREIGN KEY (blocked_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: user_message_blocks user_message_blocks_blocker_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_message_blocks
    ADD CONSTRAINT user_message_blocks_blocker_id_fkey FOREIGN KEY (blocker_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: user_notifications user_notifications_actor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_notifications
    ADD CONSTRAINT user_notifications_actor_id_fkey FOREIGN KEY (actor_id) REFERENCES public.users(userid);


--
-- Name: user_notifications user_notifications_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_notifications
    ADD CONSTRAINT user_notifications_definition_id_fkey FOREIGN KEY (definition_id) REFERENCES public.definitions(definitionid);


--
-- Name: user_notifications user_notifications_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_notifications
    ADD CONSTRAINT user_notifications_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: user_notifications user_notifications_valsi_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_notifications
    ADD CONSTRAINT user_notifications_valsi_id_fkey FOREIGN KEY (valsi_id) REFERENCES public.valsi(valsiid);


--
-- Name: user_profile_images user_profile_images_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_profile_images
    ADD CONSTRAINT user_profile_images_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: user_quiz_answer_history user_quiz_answer_history_flashcard_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_quiz_answer_history
    ADD CONSTRAINT user_quiz_answer_history_flashcard_id_fkey FOREIGN KEY (flashcard_id) REFERENCES public.flashcards(id) ON DELETE CASCADE;


--
-- Name: user_quiz_answer_history user_quiz_answer_history_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_quiz_answer_history
    ADD CONSTRAINT user_quiz_answer_history_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: user_search_history user_search_history_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_search_history
    ADD CONSTRAINT user_search_history_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid) ON DELETE SET NULL;


--
-- Name: user_settings user_settings_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_settings
    ADD CONSTRAINT user_settings_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: users users_disabled_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_disabled_by_fkey FOREIGN KEY (disabled_by) REFERENCES public.users(userid);


--
-- Name: valsi_sounds valsi_sounds_valsi_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi_sounds
    ADD CONSTRAINT valsi_sounds_valsi_id_fkey FOREIGN KEY (valsi_id) REFERENCES public.valsi(valsiid) ON DELETE CASCADE;


--
-- Name: valsi valsi_source_langid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi
    ADD CONSTRAINT valsi_source_langid_fkey FOREIGN KEY (source_langid) REFERENCES public.languages(langid);


--
-- Name: valsi_subscriptions valsi_subscriptions_source_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi_subscriptions
    ADD CONSTRAINT valsi_subscriptions_source_comment_id_fkey FOREIGN KEY (source_comment_id) REFERENCES public.comments(commentid);


--
-- Name: valsi_subscriptions valsi_subscriptions_source_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi_subscriptions
    ADD CONSTRAINT valsi_subscriptions_source_definition_id_fkey FOREIGN KEY (source_definition_id) REFERENCES public.definitions(definitionid);


--
-- Name: valsi_subscriptions valsi_subscriptions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi_subscriptions
    ADD CONSTRAINT valsi_subscriptions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(userid);


--
-- Name: valsi_subscriptions valsi_subscriptions_valsi_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi_subscriptions
    ADD CONSTRAINT valsi_subscriptions_valsi_id_fkey FOREIGN KEY (valsi_id) REFERENCES public.valsi(valsiid);


--
-- Name: valsi valsi_userid; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.valsi
    ADD CONSTRAINT valsi_userid FOREIGN KEY (userid) REFERENCES public.users(userid) MATCH FULL;


--
-- Name: wave_search_chunks wave_search_chunks_document_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.wave_search_chunks
    ADD CONSTRAINT wave_search_chunks_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.wave_search_documents(document_id) ON DELETE CASCADE;


--
-- Name: webrtc_signaling webrtc_signaling_from_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webrtc_signaling
    ADD CONSTRAINT webrtc_signaling_from_user_id_fkey FOREIGN KEY (from_user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- Name: webrtc_signaling webrtc_signaling_to_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webrtc_signaling
    ADD CONSTRAINT webrtc_signaling_to_user_id_fkey FOREIGN KEY (to_user_id) REFERENCES public.users(userid) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--


--
-- PostgreSQL database dump
--


-- Dumped from database version 17.11 (Debian 17.11-0+deb13u1)
-- Dumped by pg_dump version 17.11 (Debian 17.11-0+deb13u1)


--
-- Data for Name: languages; Type: TABLE DATA; Schema: public; Owner: -
--

COPY public.languages (langid, tag, englishname, lojbanname, realname, forlojban, url) FROM stdin;
0	xx	nolanguage	nolanguage	nolanguage	noforlojban	http://nowhere/
19	no	Norwegian		norsk	lojban	http://www.sprakrad.no/
2	en	English	gli'icybau	English	Lojban	\N
3	hi	Hindi	xinbau	हिन्दी	\N	\N
4	es	Spanish	spa'anybau	Español	\N	\N
6	zh	Chinese	jugbau	中文	\N	\N
7	ar	Arabic	rabybau	‮العربية‬	\N	\N
10	ja	Japanese	ponbau	日本語	ロジバン	\N
11	pl	Polish		Polski	\N	\N
13	it	Italian		Italiano	\N	\N
14	ko	Korean		한국어	\N	\N
16	el	Greek	xesybau	Ελληνικά	Λόζμπαν	\N
15	ro	Romanian		Română	\N	\N
21	pt	Portuguese	potybau	Português	\N	\N
22	sv	Swedish		Svenska	\N	\N
23	sr	Serbian		Srpski	\N	\N
24	tr	Turkish	bangrtirki	Türkçe	\N	\N
26	ka	Georgian	kartuli	ქართველი	\N	\N
27	gu	Gujarati	gudjrati	ગુજરાતી	\N	\N
28	sq	Albanian	ckiptare	shqiptare	\N	\N
29	eu	Basque	skalduna	Euskara	\N	\N
31	id	Indonesian	bidbau	Bahasa Indonesia	\N	\N
32	mg	Malagasy	malgaci	Malagasy	\N	\N
33	ne	Nepali	bangrnepali	नेपाली	\N	\N
34	sa	Sanskrit	srito	संस्कृत	\N	\N
35	so	Somali	bangrsomali	Soomaali	\N	\N
36	br	Breton	bre'one	brezhoneg	\N	\N
37	ch	Chamorro	tcamoru	Chamoru	\N	\N
38	kw	Cornish	kernauke	Kernewek	\N	\N
39	ca	Catalan	katlana	català	\N	\N
40	la	Latin	latmo	Latina	\N	\N
41	hr	Croatian	bangrxrvatski	hrvatski	\N	\N
42	nl	Dutch	bangrnedrlanda	Nederlands	\N	\N
43	hu	Hungarian	banrmadiaru	Magyarul	\N	\N
44	lt	Lithuanian	bangrlietuva	Lietuvių	\N	\N
45	bg	Bulgarian	bolgaro	Български	\N	\N
46	sk	Slovak	slovako	Slovenský	\N	\N
47	sl	Slovenian	slovino	Slovenski	\N	\N
48	vi	Vietnamese	vietnama	Tiếng Việt	\N	\N
49	et	Estonian	bangrxesti	Eesti	\N	\N
50	gl	Galician	bangrgalego	Galego	\N	\N
51	uk	Ukrainian	vukro	Українська	\N	\N
52	am	Amharic	amxari	አማርኛ	\N	\N
53	cy	Welsh	kamro	Cymraeg	\N	\N
54	ga	Irish	gailge	Gaeilge	\N	\N
55	ia	Interlingua	binbau	Interlingua	\N	\N
56	wa	Walloon	frasrvalona	Walon	\N	\N
58	art-loglan	Loglan	loglan	loglan	\N	http://www.loglan.net/
59	eo	Esperanto	spe'atybau	Esperanto	Loĵbano	http://www.esperanto.net/
9	de	German	dotybau	Deutsch	Lojban	\N
5	ru	Russian	rukybau	Русский	Ложбан	\N
60	tpi	Tok Pisin	bangrtokpisina	tok Pisin	tok Losban	\N
1	jbo	Lojban	jbobau	lojban	\N	http://www.lojban.org/
315	test	Test Language	cipybau	Test Language	Lojban	
314	ta	Tamil	bangrtamiji	தமிழ்	யாழ்வாண்	\N
8	fr	French	fasybau	Français	\N	\N
316	art-guaspi	Gua\\spi	gaspo	gua\\spi	 	http://www.math.ucla.edu/~jimc/guaspi
18	cs	Czech	tce'exo	Česká	\N	\N
12	da	Danish	danseke	Dansk	\N	\N
317	en-simple	Simple English	sampu glibau	Simple English	\N	\N
20	fi	Finnish	suomne	Suomi	\N	\N
25	fa	Farsi	bangrfarsi	‮فرسى‬	\N	\N
17	he	Hebrew	xebybau	עברית	לוז'בן	\N
30	be	Belarusian	labru'obau	Беларуская	\N	\N
318	fr-facile	Easy French	frili fasybau	Français facile	\N	\N
319	lv	Latvian	bangrlatfiacu	Latviešu	\N	\N
320	pt-br	Portuguese (Brazil)	razypotybau	Português do Brasil	\N	\N
321	en-bpfk	BPFK English	glico bangu be bypyfyky.	BPFK English	\N	https://groups.google.com/forum/#!forum/bpfk-list
322	tok	Toki Pona	tokpona	toki pona	toki losupan	https://en.wikipedia.org/wiki/Toki_Pona
57	tlh	Klingon	bangrtlingana	tlhIngan	\N	http://www.kli.org/
323	art-ithkuil	Ithkuil	.itku'le	wiosaḑca iţkuil	\N	https://yuorb.github.io/en/
324	art-picture	Picture language	xrabau	🖼️	\N	\N
\.


--
-- Data for Name: valsitypes; Type: TABLE DATA; Schema: public; Owner: -
--

COPY public.valsitypes (typeid, descriptor) FROM stdin;
0	nalvla
1	gismu
2	cmavo
4	lujvo
5	fu'ivla
7	experimental gismu
8	experimental cmavo
6	cmavo-compound
9	bu-letteral
10	zei-lujvo
12	obsolete fu'ivla
13	obsolete zei-lujvo
3	cmevla
11	obsolete cmevla
14	obsolete cmavo
15	phrase
16	wiki
17	non-canonical lujvo
\.


--
-- Name: languages_langid_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('public.languages_langid_seq', 324, true);


--
-- PostgreSQL database dump complete
--



REFRESH MATERIALIZED VIEW public.user_reaction_counts;

-- The dump above clears search_path; the hand-appended migrations below use
-- unqualified names.
SELECT pg_catalog.set_config('search_path', 'public', false);

-- V206 dialogue cards
-- Dialogue ("story") cards: a script of speaker lines (front) plus questions, see src/dialogue.
-- Never Auto, never answer material; they count toward level strength and completion.
ALTER TYPE public.flashcard_direction ADD VALUE IF NOT EXISTS 'dialogue';

-- Stable speaker → Kokoro voice map, saved on first audio generation (e.g. {"Marian":"Victoria"}).
ALTER TABLE collection_items ADD COLUMN IF NOT EXISTS dialogue_voices JSONB;

-- One TTS clip per script line. `text_hash` = sha256(voice|speed|text): unchanged lines are reused.
CREATE TABLE IF NOT EXISTS dialogue_line_sounds (
    item_id INTEGER NOT NULL REFERENCES collection_items(item_id) ON DELETE CASCADE,
    line_index INTEGER NOT NULL,
    text_hash TEXT NOT NULL,
    voice TEXT NOT NULL,
    sound_data BYTEA NOT NULL,
    mime_type TEXT NOT NULL DEFAULT 'audio/ogg',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (item_id, line_index)
);

-- Each question of a dialogue is a separate answer with its own score. One attempt (one pass
-- through the dialogue) commits a single FSRS review for the card: the lowest question rating.
CREATE TABLE IF NOT EXISTS dialogue_question_answers (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(userid) ON DELETE CASCADE,
    flashcard_id INTEGER NOT NULL REFERENCES flashcards(id) ON DELETE CASCADE,
    attempt_id UUID NOT NULL,
    question_index INTEGER NOT NULL,
    question_kind TEXT NOT NULL,
    answer TEXT NOT NULL,
    correct BOOLEAN NOT NULL,
    hint_used BOOLEAN NOT NULL DEFAULT false,
    rating INTEGER NOT NULL,
    answered_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (attempt_id, question_index)
);
CREATE INDEX IF NOT EXISTS idx_dialogue_answers_user_card
    ON dialogue_question_answers (user_id, flashcard_id, answered_at);
