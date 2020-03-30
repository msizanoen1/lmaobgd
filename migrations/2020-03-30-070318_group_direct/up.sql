DROP INDEX answer_group_idx;
ALTER TABLE answers
    RENAME test_id TO test;

ALTER TABLE answers
    DROP CONSTRAINT answers_group__fkey,
    ALTER COLUMN test TYPE TEXT,
    ALTER COLUMN test SET NOT NULL;

ALTER TABLE groups
    RENAME TO groups_old;

CREATE TABLE groups (
    text TEXT PRIMARY KEY
);

INSERT INTO
    groups
SELECT
    text
FROM
    groups_old
ON CONFLICT DO NOTHING;

UPDATE
    answers
SET
    test = (
        SELECT
            groups_old.text
        FROM
            groups_old
        WHERE
            answers.test::integer = groups_old.id
    );

ALTER TABLE answers
    ADD CONSTRAINT answers_test_fkey FOREIGN KEY (test) REFERENCES groups (text);

DROP TABLE groups_old;

CREATE INDEX answer_group_idx ON answers (test);
