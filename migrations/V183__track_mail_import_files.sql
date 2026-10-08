-- Track every source file, including files whose email already exists.
-- Deleting a message also clears its file records so the archive can be reimported.
CREATE TABLE mail_imported_files (
    file_path TEXT PRIMARY KEY,
    message_id INTEGER REFERENCES messages(id) ON DELETE CASCADE
);

INSERT INTO mail_imported_files (file_path, message_id)
SELECT file_path, id FROM messages WHERE file_path IS NOT NULL
ON CONFLICT (file_path) DO NOTHING;

CREATE INDEX idx_messages_message_id_trimmed ON messages ((btrim(message_id)));
CREATE INDEX idx_messages_without_message_id ON messages (date, subject, from_address)
    WHERE nullif(btrim(message_id), '') IS NULL;
CREATE INDEX idx_messages_thread_sent_at ON messages (cleaned_subject, sent_at DESC, id DESC);
