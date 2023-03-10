-- Add down migration script here
ALTER TABLE dates RENAME COLUMN date_ TO d;
