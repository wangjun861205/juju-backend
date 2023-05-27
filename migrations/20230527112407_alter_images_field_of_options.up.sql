-- Add up migration script here
ALTER TABLE options ADD COLUMN images VARCHAR[] NOT NULL DEFAULT '{}'::VARCHAR[];
