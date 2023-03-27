-- Add up migration script here
ALTER TABLE answers ADD COLUMN question_id INTEGER NOT NULL REFERENCES questions (id);
