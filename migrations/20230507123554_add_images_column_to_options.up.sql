-- Add up migration script here
ALTER TABLE options ADD COLUMN images BYTEA[] NOT NULL DEFAULT '{}'::BYTEA[];
