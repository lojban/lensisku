-- Temporary tables shadow application tables; the test transaction rolls them back.
CREATE TEMP TABLE users (userid integer, username text);
CREATE TEMP TABLE languages (langid integer, realname text, englishname text, lojbanname text);
CREATE TEMP TABLE valsi (valsiid integer, word text, typeid smallint, source_langid integer);
CREATE TEMP TABLE definitions (definitionid integer, valsiid integer, langid integer);
CREATE TEMP TABLE definition_versions (version_id bigint, definition_id integer, langid integer, user_id integer, created_at timestamptz, message text, mw_revid bigint);
CREATE TEMP TABLE threads (threadid integer, valsiid integer, natlangwordid integer, definitionid integer, definition_link_id integer, target_user_id integer, collection_id integer);
CREATE TEMP TABLE comments (commentid integer, threadid integer, userid integer, time integer, subject text, content jsonb, commentnum integer, parentid integer, import_source text);
CREATE TEMP TABLE collections (collection_id integer, is_public boolean);
CREATE TEMP TABLE messages (id bigint, subject text, content text, from_address text, sent_at timestamptz);
CREATE TEMP TABLE message_spam_votes (message_id bigint);
CREATE TEMP TABLE wiki_articles (id bigint, title text, plain_text text, revision_id bigint, last_edited timestamptz, is_redirect boolean);
INSERT INTO users VALUES (1, 'alice'), (2, 'officialdata');
INSERT INTO languages VALUES (1, 'Lojban', 'Lojban', 'lojban');
INSERT INTO valsi VALUES (1, 'wiki page', 16, 1), (2, 'word', 1, 2);
INSERT INTO definitions VALUES (1, 1, 1), (2, 2, 1);
INSERT INTO definition_versions VALUES
 (10, 1, 1, 1, to_timestamp(1000), 'create wiki', NULL),
 (11, 2, 1, 1, to_timestamp(1000), 'define word', NULL),
 (12, 1, 1, 1, to_timestamp(1000), 'edit wiki', NULL),
 (13, 1, 1, 1, to_timestamp(1000), 'mirrored revision', 99);
INSERT INTO threads VALUES
 (1, NULL, NULL, NULL, NULL, NULL, NULL),
 (2, 2, NULL, 2, NULL, NULL, NULL),
 (3, NULL, NULL, NULL, NULL, NULL, 1);
INSERT INTO collections VALUES (1, false);
INSERT INTO comments VALUES
 (1, 1, 1, 1000, 'free wave', '[]', 1, NULL, NULL),
 (2, 1, 1, 1000, '', '[]', 2, 1, NULL),
 (3, 2, 1, 1000, 'definition comment', '[]', 1, NULL, NULL),
 (4, 1, 1, 1000, 'jbotcan', '[]', 3, NULL, 'jbotcan'),
 (5, 1, 1, 1000, 'forum', '[]', 4, NULL, 'freeforums'),
 (6, 3, 1, 1000, 'private', '[]', 1, NULL, NULL),
 (7, 1, 2, 1000, 'system', '[]', 5, NULL, NULL);
INSERT INTO messages VALUES (100, 'mail', 'mail body', 'author@example.test', to_timestamp(1000)), (101, 'spam', 'spam body', 'spam@example.test', to_timestamp(1000));
INSERT INTO message_spam_votes VALUES (101);
INSERT INTO wiki_articles VALUES (10, 'mirrored', 'wiki body', 99, to_timestamp(1000), false), (11, 'redirect', '', 100, to_timestamp(1000), true);
