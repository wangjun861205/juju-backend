-- Add up migration script here
ALTER TABLE questions ADD COLUMN owner INTEGER REFERENCES users(id) NOT NULL;
