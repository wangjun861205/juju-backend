-- Add up migration script here
ALTER TABLE answers ADD CONSTRAINT unique__user_id__option_id__question_id UNIQUE (user_id, option_id, question_id);
