create type question_type as ENUM ('single', 'multi');

alter table questions add column type_ question_type not null default 'single';

alter table answers add column question_id int not null references questions(id);
