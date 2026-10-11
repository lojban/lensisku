CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TABLE users (userid SERIAL PRIMARY KEY, username TEXT NOT NULL, email TEXT NOT NULL, password TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(), followers INTEGER DEFAULT 0, role TEXT DEFAULT 'editor', disabled BOOLEAN DEFAULT false,
    email_confirmed BOOLEAN DEFAULT true, email_confirmation_token TEXT, email_confirmation_sent_at TIMESTAMPTZ,
    realname TEXT DEFAULT '', url TEXT, personal TEXT);
CREATE TABLE permissions (id SERIAL PRIMARY KEY, name TEXT UNIQUE, description TEXT);
CREATE TABLE role_permissions (role TEXT, permission_id INTEGER, PRIMARY KEY(role, permission_id));
CREATE TABLE user_sessions (id BIGSERIAL PRIMARY KEY, session_uuid UUID UNIQUE DEFAULT gen_random_uuid(), user_id INTEGER REFERENCES users,
    started_at TIMESTAMPTZ DEFAULT NOW(), ended_at TIMESTAMPTZ, last_active_at TIMESTAMPTZ DEFAULT NOW(), ip_address INET, user_agent TEXT);
CREATE TABLE collections (collection_id SERIAL PRIMARY KEY, user_id INTEGER REFERENCES users, name TEXT DEFAULT '', description TEXT,
    is_public BOOLEAN DEFAULT false, cover_collection_image_id INTEGER, created_at TIMESTAMPTZ DEFAULT NOW(), updated_at TIMESTAMPTZ DEFAULT NOW());
CREATE TABLE collection_items (item_id SERIAL PRIMARY KEY, collection_id INTEGER REFERENCES collections, definition_id INTEGER,
    free_content_front TEXT, free_content_back TEXT, langid INTEGER, owner_user_id INTEGER, license TEXT, script TEXT,
    is_original BOOLEAN, notes TEXT, position INTEGER, auto_progress BOOLEAN, canonical_form TEXT);
CREATE TABLE collection_images (collection_image_id SERIAL PRIMARY KEY, content_sha256 BYTEA UNIQUE, image_data BYTEA, mime_type TEXT);
CREATE TABLE collection_item_images (item_id INTEGER, collection_image_id INTEGER, side TEXT);
CREATE TABLE collection_item_sounds (item_id INTEGER, sound_data BYTEA, mime_type TEXT);
CREATE TABLE flashcards (id SERIAL PRIMARY KEY, collection_id INTEGER, item_id INTEGER);
CREATE TABLE flashcard_levels (level_id SERIAL PRIMARY KEY, collection_id INTEGER);
CREATE TABLE flashcard_level_items (level_id INTEGER, flashcard_id INTEGER, position INTEGER);
CREATE TABLE threads (threadid SERIAL PRIMARY KEY, collection_id INTEGER, valsiid INTEGER, natlangwordid INTEGER, definitionid INTEGER,
    definition_link_id INTEGER, target_user_id INTEGER);
CREATE TABLE comments (commentid SERIAL PRIMARY KEY, threadid INTEGER REFERENCES threads, parentid INTEGER, userid INTEGER,
    commentnum INTEGER DEFAULT 1, time INTEGER DEFAULT 1, subject TEXT DEFAULT '', content JSONB DEFAULT '[]', plain_content TEXT);
CREATE TABLE comment_counters (comment_id INTEGER, total_reactions BIGINT DEFAULT 0, total_replies BIGINT DEFAULT 0);
CREATE TABLE comment_bookmarks (comment_id INTEGER, user_id INTEGER, created_at TIMESTAMPTZ DEFAULT NOW());
CREATE TABLE comment_reactions (comment_id INTEGER, user_id INTEGER, reaction TEXT);
CREATE TABLE comment_activity_counters (comment_id INTEGER, total_bookmarks BIGINT, total_replies BIGINT, total_opinions BIGINT,
    total_reactions BIGINT, last_activity_at TIMESTAMPTZ);
CREATE TABLE valsi (valsiid INTEGER PRIMARY KEY, word TEXT);
CREATE TABLE definitions (definitionid INTEGER PRIMARY KEY, definition TEXT);
CREATE VIEW convenientcomments AS SELECT c.*, u.username, u.realname, cc.total_reactions, cc.total_replies,
    t.valsiid, t.definitionid, t.definition_link_id, t.collection_id FROM comments c
    JOIN users u ON u.userid = c.userid JOIN threads t USING(threadid) LEFT JOIN comment_counters cc ON cc.comment_id = c.commentid;
CREATE TABLE assistant_chats (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), user_id INTEGER REFERENCES users, title TEXT DEFAULT '',
    messages JSONB DEFAULT '[]', primary_model_id TEXT, scroll_top DOUBLE PRECISION DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(), updated_at TIMESTAMPTZ DEFAULT NOW());
INSERT INTO permissions(name) VALUES ('edit_definition');
INSERT INTO role_permissions VALUES ('editor', 1);
INSERT INTO users(username,email,password) VALUES ('owner','owner@example.test','placeholder'),('other','other@example.test','placeholder');
INSERT INTO collections(user_id,name,is_public) VALUES (1,'private',false),(2,'other private',false),(1,'public',true);
INSERT INTO collection_items(collection_id,free_content_front,notes) VALUES (1,'private content','private note'),(2,'own content','own note'),(3,'public content','public note');
INSERT INTO threads(collection_id) VALUES (1),(3);
INSERT INTO comments(threadid,userid,subject,content,plain_content) VALUES (1,1,'private','[{"type":"text","data":"private content"}]','private content'),(2,1,'public','[{"type":"text","data":"public content"}]','public content');
INSERT INTO comment_counters(comment_id) VALUES (1),(2);
INSERT INTO comment_activity_counters VALUES (1,0,0,0,0,NOW()),(2,0,0,0,0,NOW());
INSERT INTO flashcards(collection_id,item_id) VALUES (1,1),(2,2);
INSERT INTO flashcard_levels(collection_id) VALUES (2);
