-- This file should undo anything in `up.sql`
DROP TABLE organization_read_marks;
ALTER TABLE vote_read_marks RENAME TO vote_update_marks, DROP COLUMN version;
ALTER TABLE question_read_marks RENAME TO question_update_marks, DROP COLUMN version;