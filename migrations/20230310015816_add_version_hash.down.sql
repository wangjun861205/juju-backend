-- Add down migration script here
ALTER TABLE organizations DROP COLUMN version;
ALTER TABLE votes DROP COLUMN version;
ALTER TABLE questions DROP COLUMN version;
