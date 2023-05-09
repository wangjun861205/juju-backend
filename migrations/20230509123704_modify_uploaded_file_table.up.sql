-- Add up migration script here
ALTER TABLE uploaded_files DROP COLUMN fetch_code, ADD COLUMN extension VARCHAR;
ALTER TABLE uploaded_files RENAME COLUMN ownner TO owner_id;
