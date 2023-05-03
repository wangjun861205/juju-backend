-- Add up migration script here
ALTER TABLE organizations ADD COLUMN description TEXT NOT NULL DEFAULT '';
