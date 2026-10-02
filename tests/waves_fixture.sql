BEGIN;
CREATE SCHEMA __TEST_SCHEMA__;
SET LOCAL search_path = __TEST_SCHEMA__, public;
CREATE TABLE users (userid integer primary key, username text);
CREATE TABLE comments (commentid integer primary key, threadid integer, userid integer, subject text, plain_content text, import_source text, time integer);
CREATE TABLE threads (threadid integer primary key, collection_id integer);
CREATE TABLE messages (id integer primary key, subject text, content text, from_address text, sent_at timestamptz);
CREATE TABLE wiki_articles (page_id integer primary key, title text, plain_text text, is_redirect boolean, last_edited timestamptz);
CREATE TABLE valsi (valsiid integer primary key, word text, typeid integer);
CREATE TABLE definitions (definitionid integer primary key, valsiid integer, definition text, metadata jsonb, created_at timestamptz);
CREATE TABLE comment_activity_counters (comment_id integer primary key, total_reactions bigint, total_replies bigint);
INSERT INTO users VALUES (1,'alice');
INSERT INTO threads VALUES (1,10),(2,20);
INSERT INTO comments VALUES (1,1,1,'logical scope','A detailed discussion of quantifier scope.',NULL,1000),
 (2,2,1,'forum','Predicates have interesting meanings.', 'jbotcan',2000),
 (3,2,1,'not a wildcard','Literal 50%_exact substring.','freeforums',3000);
INSERT INTO messages VALUES (1,'mail scope','Quantifier binding in sentences.','bob@example.com',now());
INSERT INTO wiki_articles VALUES (1,'logical scope','Scope and quantification.',false,now()),(2,'redirect','logical scope',true,now());
INSERT INTO valsi VALUES (1,'Native scope',16),(2,'ordinary dictionary word',1);
INSERT INTO definitions VALUES (1,1,'Quantifier scope in native wiki.', '{}',now()),(2,2,'ordinary definition','{}',now());
