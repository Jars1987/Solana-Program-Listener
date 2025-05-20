use diesel::prelude::*;

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::db::schema::polls)]
pub struct NewPoll {
    pub poll_id: i64,
    pub poll_owner: [u8; 32],
    pub poll_name: String,
    pub poll_description: String,
    pub poll_start: i64,
    pub poll_end: i64,
    pub candidate_amount: i64,
    pub candidate_winner: [u8; 32],
}

#[derive(Queryable, Debug)]
pub struct Poll {
    pub id: i32,
    pub poll_id: i64,
    pub poll_owner: Vec<u8>,
    pub poll_name: String,
    pub poll_description: String,
    pub poll_start: i64,
    pub poll_end: i64,
    pub candidate_amount: i64,
    pub candidate_winner: Vec<u8>,
}
