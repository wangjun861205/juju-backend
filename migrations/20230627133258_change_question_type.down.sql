-- Add down migration script here
ALTER TABLE questions ALTER COLUMN type_ SET DATA TYPE question_type;