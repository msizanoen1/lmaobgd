ALTER TABLE answers
    ADD COLUMN reviewed BOOLEAN NOT NULL DEFAULT 'f';
CREATE INDEX answers_reviewed_idx ON answers (reviewed);
