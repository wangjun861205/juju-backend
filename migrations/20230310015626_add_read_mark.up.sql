-- Add up migration script here
CREATE TABLE vote_read_marks (
	id SERIAL NOT NULL,
	vote_id INT NOT NULL REFERENCES votes(id) ON DELETE CASCADE,
	user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	has_read BOOLEAN NOT NULL,
	PRIMARY KEY (id)
);
