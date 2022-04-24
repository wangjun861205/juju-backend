-- Your SQL goes here
CREATE TABLE uploaded_files (
	id SERIAL NOT NULL,
	name VARCHAR NOT NULL,
	fetch_code VARCHAR NOT NULL,
	PRIMARY KEY (id)
);