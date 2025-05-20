CREATE TABLE polls (
    id SERIAL PRIMARY KEY,
    poll_id BIGINT UNIQUE NOT NULL,
    poll_owner BYTEA NOT NULL,
    poll_name VARCHAR(64) NOT NULL,
    poll_description VARCHAR(280) NOT NULL,
    poll_start BIGINT NOT NULL,
    poll_end BIGINT NOT NULL,
    candidate_amount BIGINT NOT NULL,
    candidate_winner BYTEA NOT NULL
);
