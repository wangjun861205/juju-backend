-- Add down migration script here
ALTER TABLE uploaded_files DROP COLUMN extension, ADD COLUMN fetch_code VARCHAR NOT NULL;
ALTER TABLE uploaded_files RENAME COLUMN owner_id TO ownner;
