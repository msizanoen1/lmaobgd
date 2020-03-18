ALTER TABLE answers
    ADD COLUMN valid_answers INTEGER ARRAY NOT NULL DEFAULT '{}',
    ALTER COLUMN answer1 DROP NOT NULL,
    ALTER COLUMN answer2 DROP NOT NULL,
    ALTER COLUMN answer3 DROP NOT NULL,
    ALTER COLUMN answer4 DROP NOT NULL;
UPDATE answers
    SET valid_answers = ARRAY[answer1, answer2, answer3, answer4];
