-- Add up migration script here
CREATE TYPE VoteVisibility AS ENUM ('Public', 'Organization', 'WhiteList');
ALTER TABLE votes ADD COLUMN visibility VoteVisibility NOT NULL DEFAULT 'Organization';
