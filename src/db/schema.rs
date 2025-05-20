// @generated automatically by Diesel CLI.

diesel::table! {
    polls (id) {
        id -> Int4,
        poll_id -> Int8,
        poll_owner -> Bytea,
        #[max_length = 64]
        poll_name -> Varchar,
        #[max_length = 280]
        poll_description -> Varchar,
        poll_start -> Int8,
        poll_end -> Int8,
        candidate_amount -> Int8,
        candidate_winner -> Bytea,
    }
}
