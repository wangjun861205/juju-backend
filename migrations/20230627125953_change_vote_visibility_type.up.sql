-- Add up migration script here
ALTER TABLE votes ALTER COLUMN visibility SET DATA TYPE VARCHAR;
