-- Add up migration script here
alter table users_organizations drop constraint users_organizations_organization_id_fkey,
add constraint users_organizations_organization_id_fkey
foreign key (organization_id)
references organizations(id)
on delete cascade;

alter table users_organizations drop constraint users_organizations_user_id_fkey,
add constraint users_organizations_user_id_fkey
foreign key (user_id)
references users(id)
on delete cascade;
