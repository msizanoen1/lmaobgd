DROP INDEX answer_group_idx;
ALTER TABLE groups RENAME TO groups_old;
CREATE TABLE groups (
    id INTEGER PRIMARY KEY,
    text TEXT NOT NULL
);
INSERT INTO
    groups
SELECT
    question_id, test
FROM
    answers;
ALTER TABLE answers
    DROP CONSTRAINT answers_test_fkey;
ALTER TABLE answers
    RENAME test TO test_id;
ALTER TABLE answers
    ALTER COLUMN test_id TYPE INTEGER USING question_id,
    ALTER COLUMN test_id SET NOT NULL,
    ADD CONSTRAINT answers_group__fkey FOREIGN KEY (test_id) REFERENCES groups (id);
DROP TABLE groups_old;
CREATE INDEX answer_group_idx ON answers (test_id);
