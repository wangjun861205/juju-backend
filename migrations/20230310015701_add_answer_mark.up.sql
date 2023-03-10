-- Add up migration script here
CREATE TABLE question_update_marks (
	id SERIAL NOT NULL,
	question_id INT NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
	user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	has_updated BOOLEAN NOT NULL,
	PRIMARY KEY (id)
);
