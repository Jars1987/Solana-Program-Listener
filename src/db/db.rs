use crate::db::models::NewPoll;
use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy::dotenv;
use std::env;
use std::time::Duration;

use super::models::Poll;
use super::schema::polls::dsl::*;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn establish_pool() -> Result<PgPool> {
    dotenv().ok(); // Load .env file

    let db_url = env::var("DATABASE_URL").context("DATABASE_URL must be set in .env")?;

    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder()
        .max_size(15) // max 15 connections
        .max_lifetime(Some(Duration::from_secs(300))) // drop idle conns after 5 min
        .test_on_check_out(true) // verify health before use
        .build(manager)
        .context("Failed to build connection pool")?;

    Ok(pool)
}

/// Insert a new poll row into the database
pub fn _insert_poll(pool: &PgPool, poll: NewPoll) -> anyhow::Result<()> {
    use crate::db::schema::polls::dsl::*;

    let mut conn = pool
        .get()
        .context("Failed to get DB connection from pool")?;

    diesel::insert_into(polls)
        .values(&poll)
        .execute(&mut conn)
        .context("Failed to insert poll into database")?;

    Ok(())
}

pub fn upsert_poll(pool: &PgPool, poll: &NewPoll) -> anyhow::Result<()> {
    use crate::db::schema::polls::dsl::*;

    let mut conn = pool
        .get()
        .context("Failed to get DB connection from pool")?;

    diesel::insert_into(polls)
        .values(poll)
        .on_conflict(poll_id) // Unique column (you must add UNIQUE constraint in schema)
        .do_update()
        .set((
            poll_owner.eq(&poll.poll_owner),
            poll_name.eq(&poll.poll_name),
            poll_description.eq(&poll.poll_description),
            poll_start.eq(poll.poll_start),
            poll_end.eq(poll.poll_end),
            candidate_amount.eq(poll.candidate_amount),
            candidate_winner.eq(&poll.candidate_winner),
        ))
        .execute(&mut conn)?;

    Ok(())
}

pub fn list_polls(pool: &PgPool) -> anyhow::Result<Vec<Poll>> {
    let mut conn = pool.get()?;
    let results = polls.load::<Poll>(&mut conn)?;
    Ok(results)
}
