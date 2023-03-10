-- Add down migration script here
alter table questions drop column type_;

alter table answers drop column question_id;

drop type question_type;
