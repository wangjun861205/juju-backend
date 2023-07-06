-- Add down migration script here
ALTER TABLE votes DROP COLUMN likes, DROP COLUMN dislikes;
ALTER TABLE questions DROP COLUMN likes, DROP COLUMN dislikes;

DROP TABLE favorite_votes;
DROP TABLE favorite_questions;