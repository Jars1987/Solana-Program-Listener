-- Your SQL goes here
ALTER TABLE polls ADD CONSTRAINT polls_poll_id_unique UNIQUE (poll_id);
