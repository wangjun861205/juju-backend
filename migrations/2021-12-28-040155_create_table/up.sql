-- Your SQL goes here
create table users (
	id serial primary key,
	nickname varchar not null,
	phone varchar not null unique,
	email varchar not null unique,
	password varchar not null,
	salt varchar not null
);

create table organizations (
	id serial primary key,
	name varchar not null
);

create table users_organizations (
	id serial primary key,
	user_id int not null references users(id),
	organization_id int not null references organizations(id),
	unique(user_id, organization_id)
);

create type vote_status as ENUM ('collecting', 'closed');

create table votes (
	id serial primary key,
	name varchar not null,
	deadline date,
	status vote_status not null,
	organization_id int not null references organizations(id)
);

create table questions (
	id serial primary key,
	description varchar not null,
	vote_id int not null references votes(id)
);

create table options (
	id serial primary key,
	option varchar not null,
	question_id int not null references questions(id)
);

create table answers (
	id serial primary key,
	user_id int not null references users(id),
	option_id int not null references options(id),
	unique(user_id, option_id)
);

create table dates (
	id serial primary key,
	d date not null,
	user_id int not null references users(id),
	vote_id int not null references votes(id),
	unique(user_id, vote_id, d)
);

create table invite_codes (
	id serial primary key,
	code varchar not null
);