-- This file should undo anything in `up.sql`
ALTER TABLE organizations DROP COLUMN version;
ALTER TABLE votes DROP COLUMN version;
ALTER TABLE questions DROP COLUMN version;