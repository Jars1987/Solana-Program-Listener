-- This file should undo anything in `up.sql`
ALTER TABLE polls DROP CONSTRAINT polls_poll_id_unique;
