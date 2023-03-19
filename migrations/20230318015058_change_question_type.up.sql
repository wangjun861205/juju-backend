-- Add up migration script here
ALTER TYPE question_type RENAME VALUE 'single' TO 'SINGLE';
ALTER TYPE question_type RENAME VALUE 'multi' TO 'MULTI';