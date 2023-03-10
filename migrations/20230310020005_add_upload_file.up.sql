-- Add up migration script here
CREATE TABLE uploaded_files (
	id SERIAL NOT NULL,
	name VARCHAR NOT NULL,
	fetch_code VARCHAR NOT NULL,
	ownner INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	PRIMARY KEY (id)
);
