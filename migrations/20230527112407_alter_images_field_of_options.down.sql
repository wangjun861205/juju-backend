-- Add down migration script here
ALTER TABLE options DROP COLUMN IF EXISTS images;
