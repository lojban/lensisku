-- Give existing accounts the same private starter collections and flashcard course
-- that new registrations receive in the auth services.
WITH defaults(name, ordinal) AS (
    VALUES
        ('Definitions I like', 1),
        ('Definitions I don''t like', 2),
        ('Definitions that need improvement', 3),
        ('My first flashcard course', 4)
), inserted AS (
    INSERT INTO collections (user_id, name, description, is_public)
    SELECT u.userid, d.name,
           'A starter collection created automatically to show how collections work. You can rename, edit, or delete it.',
           false
    FROM users u CROSS JOIN defaults d
    WHERE NOT EXISTS (
        SELECT 1 FROM collections c WHERE c.user_id = u.userid AND c.name = d.name
    )
    RETURNING collection_id, user_id, name
), courses AS (
    SELECT collection_id
    FROM inserted
    WHERE name = 'My first flashcard course'
)
INSERT INTO flashcard_levels (collection_id, name, position)
SELECT courses.collection_id, levels.name, levels.position
FROM courses
CROSS JOIN (VALUES ('Level 1', 0), ('Level 2', 1), ('Level 3', 2)) AS levels(name, position)
WHERE NOT EXISTS (
    SELECT 1 FROM flashcard_levels fl
    WHERE fl.collection_id = courses.collection_id AND fl.position = levels.position
);
