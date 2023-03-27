-- Add down migration script here
ALTER TABLE answers DROP COLUMN question_id;
