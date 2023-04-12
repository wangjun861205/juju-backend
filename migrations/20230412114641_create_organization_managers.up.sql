-- Add up migration script here
CREATE TABLE organization_managers (
    id SERIAL NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    organization_id INTEGER NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    PRIMARY KEY (id),
    CONSTRAINT unique_organization_managers_user_id_organization_id UNIQUE (user_id, organization_id)
);
