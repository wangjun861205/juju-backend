-- This file should undo anything in `up.sql`
drop table if exists users cascade;
drop table if exists organizations cascade;
drop table if exists users_organizations cascade;
drop table if exists votes cascade;
drop table if exists questions cascade;
drop table if exists options cascade;
drop table if exists answers cascade;
drop table if exists dates cascade;
drop table if exists invite_codes cascade;
drop type if exists vote_status cascade;