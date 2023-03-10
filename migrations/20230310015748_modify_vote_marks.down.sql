-- Add down migration script here
ALTER TABLE  RENAME vote_update_marks TO vote_read_marks;
ALTER TABLE vote_update_marks RENAME COLUMN has_updated TO has_read;
