-- Add down migration script here
ALTER TABLE organization_members RENAME TO users_organizations;
