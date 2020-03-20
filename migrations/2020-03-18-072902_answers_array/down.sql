DELETE FROM answers WHERE
    valid_answers[1] IS NULL
    OR valid_answers[2] IS NULL
    OR valid_answers[3] IS NULL
    OR valid_answers[4] IS NULL;
UPDATE answers SET
    answer1 = valid_answers[1],
    answer2 = valid_answers[2],
    answer3 = valid_answers[3],
    answer4 = valid_answers[4];
ALTER TABLE answers
    ALTER COLUMN answer1 SET NOT NULL,
    ALTER COLUMN answer2 SET NOT NULL,
    ALTER COLUMN answer3 SET NOT NULL,
    ALTER COLUMN answer4 SET NOT NULL,
    DROP COLUMN valid_answers;
