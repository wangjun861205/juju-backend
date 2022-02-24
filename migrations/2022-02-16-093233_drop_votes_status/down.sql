-- This file should undo anything in `up.sql`
ALTER TABLE votes ADD COLUMN status vote_status NOT NULL;