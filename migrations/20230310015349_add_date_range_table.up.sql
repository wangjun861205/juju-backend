-- Add up migration script here
CREATE TABLE date_ranges (
	id serial PRIMARY KEY,
	range_ DATERANGE NOT NULL,
	vote_id INT NOT NULL REFERENCES votes(id),
	user_id INT NOT NULL REFERENCES users(id)
);

