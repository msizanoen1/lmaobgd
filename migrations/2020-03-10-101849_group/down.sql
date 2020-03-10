DROP INDEX answer_group_idx;
ALTER TABLE answers
    DROP COLUMN group_;
DROP TABLE groups;
