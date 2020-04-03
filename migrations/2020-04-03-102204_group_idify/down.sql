ALTER TABLE answers
    DROP CONSTRAINT answers_test_fkey,
    ALTER COLUMN test TYPE TEXT;
UPDATE answers
SET test = (
    SELECT groups.text
    FROM groups
    WHERE answers.test::integer = groups.id
);
DROP TABLE groups;
