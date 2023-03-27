-- Add down migration script here
ALTER TABLE answers DROP CONSTRAINT unique__user_id__option_id__question_id;
