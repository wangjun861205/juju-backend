-- Your SQL goes here
CREATE TABLE organization_read_marks (
	id SERIAL NOT NULL,
	version BIGINT NOT NULL,
	has_read BOOLEAN NOT NULL,
	organization_id INT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
	user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	PRIMARY KEY (id)
);

ALTER TABLE vote_update_marks ADD COLUMN version BIGINT NOT NULL;
ALTER TABLE vote_update_marks RENAME COLUMN has_updated TO has_read;
ALTER TABLE vote_update_marks RENAME TO vote_read_marks;

ALTER TABLE question_update_marks ADD COLUMN version BIGINT NOT NULL;
ALTER TABLE question_update_marks RENAME COLUMN has_updated TO has_read;
ALTER TABLE question_update_marks RENAME TO vote_read_marks;