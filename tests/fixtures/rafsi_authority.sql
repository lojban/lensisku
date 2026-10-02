-- Run only through scripts/test_rafsi_authority.sh against a disposable database.
-- The Rust regression test applies the standalone V176 migration.

CREATE TABLE users (userid INTEGER PRIMARY KEY, username TEXT);
CREATE TABLE languages (langid INTEGER PRIMARY KEY, realname TEXT, tag TEXT, englishname TEXT, lojbanname TEXT);
CREATE TABLE valsitypes (typeid SMALLINT PRIMARY KEY, descriptor TEXT);
CREATE TABLE valsi (
 valsiid INTEGER PRIMARY KEY, word TEXT, rafsi TEXT, source_langid INTEGER DEFAULT 1,
 typeid SMALLINT, cached_decomposition TEXT, canonical_word TEXT,
 related_expansion_key TEXT, trivial_se_checked BOOLEAN DEFAULT FALSE
);
CREATE TABLE definitions (
 definitionid INTEGER PRIMARY KEY, valsiid INTEGER REFERENCES valsi, userid INTEGER REFERENCES users,
 langid INTEGER DEFAULT 2, definition TEXT DEFAULT '', etymology TEXT, notes TEXT, selmaho TEXT, jargon TEXT,
 time INTEGER DEFAULT 0, definitionnum INTEGER DEFAULT 1, rafsi TEXT,
 cached_username TEXT, cached_langrealname TEXT, cached_type_name TEXT, cached_valsiword TEXT,
 cached_rafsi TEXT, cached_decomposition TEXT, cached_canonical_word TEXT,
 cached_source_langid INTEGER, cached_typeid SMALLINT, cached_glosswords TEXT, cached_search_text TEXT,
 embedding TEXT
);
CREATE TABLE natlangwords (wordid INTEGER PRIMARY KEY, word TEXT, meaning TEXT);
CREATE TABLE keywordmapping (definitionid INTEGER, natlangwordid INTEGER, place INTEGER);
CREATE TABLE definition_versions (
 version_id SERIAL PRIMARY KEY, definition_id INTEGER, rafsi TEXT,
 created_at TIMESTAMPTZ DEFAULT now(), user_id INTEGER, message TEXT, word TEXT,
 langid INTEGER, valsiid INTEGER, definition TEXT DEFAULT '', notes TEXT,
 etymology TEXT, selmaho TEXT, jargon TEXT, gloss_keywords JSONB, place_keywords JSONB
);
CREATE TABLE definition_images (definition_id INTEGER, created_at TIMESTAMPTZ);
CREATE TABLE cached_dictionary_exports (content TEXT);
CREATE FUNCTION sync_definition_cache_fields() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN RETURN COALESCE(NEW, OLD); END $$;
CREATE TRIGGER trigger_sync_definition_cache AFTER INSERT OR UPDATE OF definition, notes, selmaho, userid, langid, valsiid, rafsi
ON definitions FOR EACH ROW EXECUTE FUNCTION sync_definition_cache_fields();
CREATE TRIGGER trigger_sync_valsi_cache AFTER UPDATE OF word, rafsi, cached_decomposition, canonical_word, typeid
ON valsi FOR EACH ROW EXECUTE FUNCTION sync_definition_cache_fields();
INSERT INTO users VALUES (1, 'officialdata'), (2, 'bairyn'), (3, 'proposer');
INSERT INTO languages (langid, realname, tag) VALUES (2, 'English', 'en');
INSERT INTO valsitypes VALUES (1, 'gismu'), (2, 'cmavo'), (4, 'lujvo'), (7, 'experimental gismu'), (8, 'experimental cmavo'), (17, 'non-canonical lujvo');
INSERT INTO valsi (valsiid, word, typeid, rafsi) VALUES
 (1, 'ckeji', 1, NULL), (2, 'kenjo', 7, NULL), (3, 'notci', 1, 'noi'),
 (4, 'cmoni', 1, 'cmo'), (5, 'co''i', 2, 'co''i'), (6, 'tunlo', 1, 'tul'),
 (7, 'tu''o', 2, 'tu''o'), (8, 'da', 2, NULL), (9, 'dau', 2, 'duv'),
 (10, 'za''u', 2, 'zab'), (11, 'ma', 2, 'maz'), (12, 'mo', 2, 'moz'),
 (13, 'xo', 2, 'xlo'), (14, 'fei', 2, 'fel'), (15, 'gai', 2, 'gam'),
 (16, 'jau', 2, 'juz'), (17, 'vai', 2, 'vav'), (18, 'vo''a', 2, 'vob'),
 (19, 'kejnoi', 17, NULL), (20, 'datni', 1, NULL), (21, 'broda', 1, NULL);
INSERT INTO definitions (definitionid, valsiid, userid, notes, rafsi)
SELECT valsiid, valsiid, CASE WHEN typeid = 7 THEN 3 ELSE 1 END, NULL,
       CASE WHEN typeid = 7 THEN 'kej kenj' ELSE NULL END FROM valsi;
INSERT INTO definitions (definitionid, valsiid, userid, notes) VALUES
 (101, 10, 2, 'I propose the experimental rafsi zab'), (102, 9, 3, 'experimental rafsi duv');
INSERT INTO definition_versions (definition_id, rafsi) VALUES (1, NULL), (101, 'zab');
UPDATE valsi SET cached_decomposition = '["kenjo","notci"]', canonical_word = 'kejnoi',
 trivial_se_checked = TRUE WHERE valsiid = 19;
-- APPLY RAFSI MIGRATION HERE
DO $$
BEGIN
 ASSERT NOT EXISTS (
     SELECT 1 FROM valsi WHERE typeid = 1 AND word NOT LIKE 'brod_'
       AND NOT (left(word, 4) = ANY(string_to_array(rafsi, ' ')))
 );
 ASSERT (SELECT 'datn' = ANY(string_to_array(rafsi, ' ')) FROM valsi WHERE word = 'datni');
 ASSERT (SELECT NOT ('brod' = ANY(string_to_array(rafsi, ' '))) FROM valsi WHERE word = 'broda');
 ASSERT (SELECT 'kej' = ANY(string_to_array(rafsi, ' ')) FROM valsi WHERE word = 'ckeji');
 ASSERT (SELECT 'co''i' = ANY(string_to_array(rafsi, ' ')) FROM valsi WHERE word = 'cmoni');
 ASSERT (SELECT 'tu''o' = ANY(string_to_array(rafsi, ' ')) FROM valsi WHERE word = 'tunlo');
 ASSERT (SELECT 'dav' = ANY(string_to_array(rafsi, ' ')) FROM valsi WHERE word = 'da');
 ASSERT (SELECT rafsi IS NULL FROM valsi WHERE word = 'dau');
 ASSERT (SELECT rafsi = 'duv' FROM definitions WHERE definitionid = 102);
 ASSERT (SELECT rafsi = 'zab' FROM definitions WHERE definitionid = 101);
 ASSERT (SELECT rafsi = 'maz' FROM definitions WHERE valsiid = 11);
 ASSERT (SELECT typeid = 4 AND cached_decomposition IS NULL AND canonical_word IS NULL
                AND NOT trivial_se_checked FROM valsi WHERE word = 'kejnoi');
 ASSERT (SELECT cached_official_rafsi = 'dav dza' AND cached_rafsi = 'dav dza' FROM definitions WHERE valsiid = 8);
 ASSERT (SELECT rafsi IS NULL AND experimental_rafsi = 'duv' FROM convenientdefinitions WHERE definitionid = 102);
 ASSERT editable_definition_rafsi(101) = 'zab';
 ASSERT editable_definition_rafsi(8) = 'dav dza';
END $$;
-- Verify cache synchronization and cross-word invalidation on proposal changes.
INSERT INTO definitions (definitionid, valsiid, userid, rafsi) VALUES (103, 1, 3, 'kex');
UPDATE valsi SET cached_decomposition = '["ckeji","notci"]', canonical_word = 'kejnoi',
 trivial_se_checked = TRUE WHERE valsiid = 19;
UPDATE definitions SET rafsi = 'key' WHERE definitionid = 103;
DO $$
BEGIN
 ASSERT (SELECT cached_decomposition IS NULL AND canonical_word IS NULL FROM valsi WHERE valsiid = 19);
 ASSERT (SELECT 'kej' = ANY(string_to_array(cached_rafsi, ' ')) AND 'key' = ANY(string_to_array(cached_rafsi, ' '))
         FROM definitions WHERE definitionid = 103);
END $$;
UPDATE valsi SET cached_decomposition = '["ckeji","notci"]' WHERE valsiid = 19;
DELETE FROM definitions WHERE definitionid = 103;
DO $$ BEGIN ASSERT (SELECT cached_decomposition IS NULL FROM valsi WHERE valsiid = 19); END $$;
-- Leave the fixture available for the ignored Rust database integration test.

