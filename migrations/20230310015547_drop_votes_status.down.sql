-- Add down migration script here
ALTER TABLE votes ADD COLUMN status vote_status NOT NULL;
