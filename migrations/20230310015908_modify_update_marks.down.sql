-- Add down migration script here
DROP TABLE organization_read_marks;

ALTER TABLE vote_read_marks DROP COLUMN version;
ALTER TABLE vote_read_marks ADD COLUMN has_updated BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE vote_read_marks RENAME TO vote_update_marks, DROP COLUMN version;

ALTER TABLE question_read_marks DROP COLUMN version;
ALTER TABLE question_read_marks ADD COLUMN has_updated BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE question_read_marks RENAME TO question_update_marks, DROP COLUMN version;
