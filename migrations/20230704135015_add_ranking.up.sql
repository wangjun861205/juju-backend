-- Add up migration script here
ALTER TABLE votes ADD COLUMN likes INTEGER NOT NULL DEFAULT 0, ADD COLUMN dislikes INTEGER NOT NULL DEFAULT 0;

ALTER TABLE questions ADD COLUMN likes INTEGER NOT NULL DEFAULT 0, ADD COLUMN dislikes INTEGER NOT NULL DEFAULT 0;

CREATE TABLE favorite_votes (
    user_id INTEGER NOT NULL,
    vote_id INTEGER NOT NULL,
    attitude INTEGER NOT NULL,
    PRIMARY KEY (user_id, vote_id)
);
COMMENT ON COLUMN favorite_votes.attitude IS '1 for like, -1 for dislike';

CREATE TABLE favorite_questions (
    user_id INTEGER NOT NULL,
    vote_id INTEGER NOT NULL,
    attitude INTEGER NOT NULL,
    PRIMARY KEY (user_id, vote_id)
);
COMMENT ON COLUMN favorite_questions.attitude IS '1 for like, -1 for dislike';