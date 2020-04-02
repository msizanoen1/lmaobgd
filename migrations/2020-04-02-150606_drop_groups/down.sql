CREATE TABLE groups (
    text TEXT CONSTRAINT groups_pkey1 PRIMARY KEY
);
INSERT INTO groups
SELECT test FROM answers
ON CONFLICT DO NOTHING;
ALTER TABLE answers
    ADD CONSTRAINT answers_test_fkey FOREIGN KEY (test) REFERENCES groups (text);
