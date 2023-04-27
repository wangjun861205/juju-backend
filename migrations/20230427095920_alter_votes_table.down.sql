-- Add down migration script here
ALTER TABLE votes DROP COLUMN visibility;
DROP TYPE VoteVisibility;
