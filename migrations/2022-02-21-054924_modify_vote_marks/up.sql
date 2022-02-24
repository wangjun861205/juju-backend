-- Your SQL goes here
ALTER TABLE vote_read_marks RENAME TO vote_update_marks;
ALTER TABLE vote_update_marks RENAME COLUMN has_read TO has_updated;