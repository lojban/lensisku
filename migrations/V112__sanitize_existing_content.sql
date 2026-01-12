-- Migration to sanitize existing content by removing HTML tags
-- Equivalent to the Rust remove_html_tags function

CREATE OR REPLACE FUNCTION strip_html_tags(input text) RETURNS text AS $$
BEGIN
    IF input IS NULL THEN
        RETURN NULL;
    END IF;
    -- Regex to match HTML tags: < followed by /, ?, or letter, then anything until >
    -- Using 'gi' for global and case-insensitive replacement
    RETURN regexp_replace(input, '<[/?a-zA-Z][^>]*>', '', 'gi');
END;
$$ LANGUAGE plpgsql;

-- 1. Sanitize definitions table
-- We only update if a tag is found, and we nullify embedding to trigger recalculation
UPDATE definitions
SET 
    definition = strip_html_tags(definition),
    notes = strip_html_tags(notes),
    etymology = strip_html_tags(etymology),
    selmaho = strip_html_tags(selmaho),
    jargon = strip_html_tags(jargon),
    embedding = NULL
WHERE definition ~ '<[/?a-zA-Z][^>]*>'
   OR notes ~ '<[/?a-zA-Z][^>]*>'
   OR etymology ~ '<[/?a-zA-Z][^>]*>'
   OR selmaho ~ '<[/?a-zA-Z][^>]*>'
   OR jargon ~ '<[/?a-zA-Z][^>]*>';

-- 2. Sanitize natlangwords table
UPDATE natlangwords
SET 
    word = strip_html_tags(word),
    meaning = strip_html_tags(meaning)
WHERE word ~ '<[/?a-zA-Z][^>]*>'
   OR meaning ~ '<[/?a-zA-Z][^>]*>';

-- 3. Sanitize users table
UPDATE users
SET 
    username = strip_html_tags(username),
    realname = strip_html_tags(realname),
    url = strip_html_tags(url),
    personal = strip_html_tags(personal)
WHERE username ~ '<[/?a-zA-Z][^>]*>'
   OR realname ~ '<[/?a-zA-Z][^>]*>'
   OR url ~ '<[/?a-zA-Z][^>]*>'
   OR personal ~ '<[/?a-zA-Z][^>]*>';

-- 4. Sanitize collections table
UPDATE collections
SET 
    name = strip_html_tags(name),
    description = strip_html_tags(description)
WHERE name ~ '<[/?a-zA-Z][^>]*>'
   OR description ~ '<[/?a-zA-Z][^>]*>';

-- 5. Sanitize collection_items table
UPDATE collection_items
SET 
    free_content_front = strip_html_tags(free_content_front),
    free_content_back = strip_html_tags(free_content_back),
    notes = strip_html_tags(notes)
WHERE free_content_front ~ '<[/?a-zA-Z][^>]*>'
   OR free_content_back ~ '<[/?a-zA-Z][^>]*>'
   OR notes ~ '<[/?a-zA-Z][^>]*>';

-- 6. Sanitize muplis table
UPDATE muplis
SET 
    lojban = strip_html_tags(lojban),
    english = strip_html_tags(english)
WHERE lojban ~ '<[/?a-zA-Z][^>]*>'
   OR english ~ '<[/?a-zA-Z][^>]*>';

-- 7. Sanitize comments table
-- update_plain_content trigger will handle updating the plain_content column
UPDATE comments
SET 
    subject = strip_html_tags(subject),
    content = COALESCE((
        SELECT jsonb_agg(
            CASE 
                WHEN (elem->>'type' = 'text' OR elem->>'type' = 'header')
                THEN jsonb_set(elem, '{data}', to_jsonb(strip_html_tags(elem->>'data')))
                ELSE elem
            END
        )
        FROM jsonb_array_elements(content) AS elem
    ), '[]'::jsonb)
WHERE jsonb_typeof(content) = 'array'
  AND (subject ~ '<[/?a-zA-Z][^>]*>' OR content::text ~ '<[/?a-zA-Z][^>]*>');

-- 8. Sanitize definition_versions table
UPDATE definition_versions
SET
    definition = strip_html_tags(definition),
    notes = strip_html_tags(notes),
    etymology = strip_html_tags(etymology),
    selmaho = strip_html_tags(selmaho),
    jargon = strip_html_tags(jargon),
    gloss_keywords = COALESCE((
        SELECT jsonb_agg(
            jsonb_build_object(
                'word', strip_html_tags(elem->>'word'),
                'meaning', strip_html_tags(elem->>'meaning')
            )
        )
        FROM jsonb_array_elements(gloss_keywords) AS elem
    ), '[]'::jsonb),
    place_keywords = COALESCE((
        SELECT jsonb_agg(
            jsonb_build_object(
                'word', strip_html_tags(elem->>'word'),
                'meaning', strip_html_tags(elem->>'meaning'),
                'place', elem->'place'
            )
        )
        FROM jsonb_array_elements(place_keywords) AS elem
    ), '[]'::jsonb)
WHERE definition ~ '<[/?a-zA-Z][^>]*>'
   OR notes ~ '<[/?a-zA-Z][^>]*>'
   OR etymology ~ '<[/?a-zA-Z][^>]*>'
   OR selmaho ~ '<[/?a-zA-Z][^>]*>'
   OR jargon ~ '<[/?a-zA-Z][^>]*>'
   OR gloss_keywords::text ~ '<[/?a-zA-Z][^>]*>'
   OR place_keywords::text ~ '<[/?a-zA-Z][^>]*>';

DROP FUNCTION strip_html_tags(text);
