-- Run in a disposable database, inserting V182 at the marker below.
CREATE TABLE valsi (
    valsiid serial PRIMARY KEY, word text NOT NULL, source_langid integer NOT NULL,
    typeid smallint NOT NULL, rafsi text, time integer NOT NULL DEFAULT 0,
    cached_decomposition text, canonical_word text, related_expansion_key text,
    trivial_se_checked boolean NOT NULL DEFAULT true,
    CONSTRAINT valsi_word_source_langid_key UNIQUE (word, source_langid)
);
CREATE TABLE valsitypes (typeid smallint PRIMARY KEY, descriptor text NOT NULL);
INSERT INTO valsitypes VALUES (5, 'lujvo'), (15, 'phrase'), (16, 'wiki');
CREATE TABLE threads (threadid integer, valsiid integer, definitionid integer);
CREATE TABLE comments (commentid integer, threadid integer);
INSERT INTO threads VALUES (1, 1, 1);
CREATE TABLE users (userid integer PRIMARY KEY, username text);
INSERT INTO users VALUES (1, 'test editor');
CREATE TABLE languages (langid integer PRIMARY KEY, realname text);
INSERT INTO languages VALUES (2, 'English');
CREATE TABLE definitions (
    definitionid integer PRIMARY KEY, metadata jsonb, valsiid integer,
    langid integer NOT NULL DEFAULT 2, userid integer NOT NULL DEFAULT 1,
    definition text NOT NULL DEFAULT 'wiki body', notes text, etymology text,
    selmaho text, jargon text, rafsi text, definitionnum integer NOT NULL DEFAULT 1,
    time integer NOT NULL DEFAULT 0, owner_only boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now()
);
INSERT INTO definitions (definitionid, valsiid, metadata)
VALUES (1, 1, '{"source_comment_id":42,"former_titles":["old"]}');
CREATE TABLE definition_images (definition_id integer);
CREATE TABLE definitionvotes (definitionid integer, value integer, userid integer);
CREATE TABLE example (exampleid integer, definitionid integer, content text, examplenum integer, time integer, userid integer);
CREATE TABLE valsi_sounds (valsi_id integer, sound_data bytea, mime_type text);
CREATE TABLE keywordmapping (definitionid integer, natlangwordid integer, place integer);
CREATE TABLE natlangwords (wordid integer, word text, meaning text);
CREATE FUNCTION can_edit_definition(integer, integer) RETURNS boolean
LANGUAGE sql AS $$ SELECT true $$;
CREATE UNIQUE INDEX idx_valsi_wiki_unique_word_source_lang
ON valsi (word, source_langid) WHERE typeid = 16;
INSERT INTO valsi (word, source_langid, typeid) VALUES ('vlasisku', 1, 16);
-- APPLY NAMESPACE MIGRATION HERE
INSERT INTO valsi (word, source_langid, typeid) VALUES ('vlasisku', 1, 15);
INSERT INTO definitions (definitionid, valsiid, definition)
SELECT 2, valsiid, 'dictionary body' FROM valsi WHERE word = 'vlasisku' AND typeid <> 16;
INSERT INTO valsi (word, source_langid, typeid) VALUES ('dict first', 1, 15), ('dict first', 1, 16);
INSERT INTO valsi (word, source_langid, typeid) VALUES ('rename target', 1, 15);
UPDATE valsi SET word = 'rename target' WHERE word = 'dict first' AND typeid = 16;
DO $$
BEGIN
    ASSERT (SELECT count(*) = 1 FROM valsi WHERE word = 'vlasisku' AND typeid <> 16);
    ASSERT (SELECT count(*) = 1 FROM valsi WHERE word = 'vlasisku' AND typeid = 16);
    ASSERT (SELECT count(*) = 2 FROM valsi WHERE word = 'rename target');
    BEGIN
        INSERT INTO valsi (word, source_langid, typeid) VALUES ('vlasisku', 1, 4);
        RAISE EXCEPTION 'Duplicate dictionary title accepted';
    EXCEPTION WHEN unique_violation THEN NULL;
    END;
    BEGIN
        INSERT INTO valsi (word, source_langid, typeid) VALUES ('vlasisku', 1, 16);
        RAISE EXCEPTION 'Duplicate wiki title accepted';
    EXCEPTION WHEN unique_violation THEN NULL;
    END;
END $$;
