CREATE TABLE groups (
    id INTEGER PRIMARY KEY,
    text TEXT NOT NULL
);
ALTER TABLE answers
    ADD COLUMN group_ INTEGER REFERENCES groups (id);
CREATE INDEX answer_group_idx ON answers (group_);
