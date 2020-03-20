CREATE TABLE answers_revert_tmp (
    question_id INTEGER PRIMARY KEY REFERENCES question_strings (question_id),
    answer1 INTEGER REFERENCES answer_strings (answer_id),
    answer2 INTEGER REFERENCES answer_strings (answer_id),
    answer3 INTEGER REFERENCES answer_strings (answer_id),
    answer4 INTEGER REFERENCES answer_strings (answer_id),
    answer_used INTEGER NOT NULL REFERENCES answer_strings (answer_id),
    reviewed BOOLEAN NOT NULL DEFAULT 'f',
    group_ INTEGER NOT NULL REFERENCES groups (id),
    valid_answers INTEGER ARRAY NOT NULL DEFAULT '{}'
);
INSERT INTO
    answers_revert_tmp (question_id, answer_used, reviewed, group_, valid_answers)
SELECT * FROM answers;
DROP TABLE answers;
ALTER TABLE answers_revert_tmp
    RENAME TO answers;
