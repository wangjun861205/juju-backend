-- Add up migration script here
CREATE TYPE ApplicationStatus AS ENUM (
    'Pending',
    'Approved',
    'Rejected'
);

CREATE TABLE join_applications (
id SERIAL NOT NULL,
user_id INTEGER NOT NULL REFERENCES users (id),
organization_id INTEGER NOT NULL REFERENCES organizations (id),
status ApplicationStatus NOT NULL DEFAULT 'Pending',
PRIMARY KEY (id),
CONSTRAINT unique_user_id_organization_id UNIQUE (user_id, organization_id)
);


