DROP INDEX answers_reviewed_idx;
ALTER TABLE answers
    DROP COLUMN reviewed;
