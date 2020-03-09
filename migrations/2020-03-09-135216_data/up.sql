CREATE TABLE question_strings (
    question_id INTEGER PRIMARY KEY,
    question_string TEXT NOT NULL
);
CREATE TABLE answer_strings (
    answer_id INTEGER PRIMARY KEY,
    answer_string TEXT NOT NULL
);
CREATE TABLE answers (
    question_id INTEGER PRIMARY KEY REFERENCES question_strings (question_id),
    answer1 INTEGER NOT NULL REFERENCES answer_strings (answer_id),
    answer2 INTEGER NOT NULL REFERENCES answer_strings (answer_id),
    answer3 INTEGER NOT NULL REFERENCES answer_strings (answer_id),
    answer4 INTEGER NOT NULL REFERENCES answer_strings (answer_id),
    answer_used INTEGER NOT NULL REFERENCES answer_strings (answer_id)
);
