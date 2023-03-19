-- Add down migration script here
ALTER TYPE question_type RENAME VALUE 'SINGLE' TO 'single';
ALTER TYPE question_type RENAME VALUE 'MULTI' TO 'multi';

