-- Your SQL goes here
ALTER TABLE organizations ADD COLUMN version BIGINT NOT NULL DEFAULT 1;
ALTER TABLE votes ADD COLUMN version BIGINT NOT NULL DEFAULT 1;
ALTER TABLE questions ADD COLUMN version BIGINT NOT NULL DEFAULT 1;