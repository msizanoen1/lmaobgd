CREATE TABLE groups (
    id SERIAL PRIMARY KEY,
    text TEXT NOT NULL UNIQUE
);

INSERT INTO groups (text)
SELECT test FROM answers
ON CONFLICT DO NOTHING;

UPDATE answers
SET test = (
    SELECT groups.id
    FROM groups
    WHERE answers.test = groups.text
);

ALTER TABLE answers
    ALTER COLUMN test TYPE INTEGER USING test::INTEGER,
    ADD FOREIGN KEY (test) REFERENCES groups (id);
