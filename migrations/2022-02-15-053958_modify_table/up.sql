-- Your SQL goes here
ALTER TABLE votes DROP CONSTRAINT votes_organization_id_fkey, ADD CONSTRAINT votes_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE;
ALTER TABLE questions DROP CONSTRAINT questions_vote_id_fkey, ADD CONSTRAINT questions_vote_id_fkey FOREIGN KEY (vote_id) REFERENCES votes(id) ON DELETE CASCADE;
ALTER TABLE options DROP CONSTRAINT options_question_id_fkey, ADD CONSTRAINT options_question_id_fkey FOREIGN KEY (question_id) REFERENCES questions(id) ON DELETE CASCADE;
ALTER TABLE answers DROP CONSTRAINT answers_option_id_fkey, ADD CONSTRAINT answers_option_id_fkey FOREIGN KEY (option_id) REFERENCES options(id) ON DELETE CASCADE;
ALTER TABLE answers DROP CONSTRAINT answers_user_id_fkey, ADD CONSTRAINT answers_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE date_ranges DROP CONSTRAINT date_ranges_user_id_fkey, ADD CONSTRAINT date_ranges_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE date_ranges DROP CONSTRAINT date_ranges_vote_id_fkey, ADD CONSTRAINT date_ranges_vote_id_fkey FOREIGN KEY (vote_id) REFERENCES votes(id) ON DELETE CASCADE;
ALTER TABLE dates DROP CONSTRAINT dates_user_id_fkey, ADD CONSTRAINT dates_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE dates DROP CONSTRAINT dates_vote_id_fkey, ADD CONSTRAINT dates_vote_id_fkey FOREIGN KEY (vote_id) REFERENCES votes(id) ON DELETE CASCADE;

ALTER TABLE answers DROP COLUMN question_id;

