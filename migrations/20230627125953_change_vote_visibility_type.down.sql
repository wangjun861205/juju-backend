-- Add down migration script here
ALTER TABLE votes ALTER COLUMN visibility SET DATA TYPE votevisibility;
