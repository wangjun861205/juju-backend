-- Add up migration script here
ALTER TABLE votes ADD COLUMN likes INTEGER NOT NULL DEFAULT 0, ADD COLUMN dislikes INTEGER NOT NULL DEFAULT 0;

ALTER TABLE questions ADD COLUMN likes INTEGER NOT NULL DEFAULT 0, ADD COLUMN dislikes INTEGER NOT NULL DEFAULT 0;

CREATE TABLE favorite_votes (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    vote_id INTEGER NOT NULL REFERENCES votes(id) ON DELETE CASCADE,
    attitude INTEGER NOT NULL,
    PRIMARY KEY (user_id, vote_id)
);
CREATE INDEX idx_favorite_votes_user_id_attitude ON favorite_votes (user_id, attitude);
CREATE INDEX idx_favorite_votes_vote_id_attitude ON favorite_votes (vote_id, attitude);
COMMENT ON COLUMN favorite_votes.attitude IS '1 for like, -1 for dislike';

CREATE TABLE favorite_questions (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    attitude INTEGER NOT NULL,
    PRIMARY KEY (user_id, question_id)
);
CREATE INDEX idx_favorite_questions_user_id_attitude ON favorite_questions (user_id, attitude);
CREATE INDEX idx_favorite_questions_question_id_attitude ON favorite_questions (question_id, attitude);
COMMENT ON COLUMN favorite_questions.attitude IS '1 for like, -1 for dislike';