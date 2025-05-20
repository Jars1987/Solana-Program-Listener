use super::models::Poll;
use super::schema::polls::dsl::*;
use crate::db::models::NewPoll;
use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy::dotenv;
use std::env;
use std::time::Duration;

/// Type alias for a pooled Postgres connection using Diesel and r2d2.
///
/// Diesel manages database connections via a connection pool.
/// r2d2 is a popular Rust connection pool manager.
///
/// Instead of opening a new DB connection on every request (which is expensive),
/// a pool allows us to reuse live connections, improving performance.
///
/// PgPool = Pool<ConnectionManager<PgConnection>>
pub type PgPool = Pool<ConnectionManager<PgConnection>>;

// Establishes a PostgreSQL connection pool using environment variables.
///
/// This function loads the `.env` file to retrieve `DATABASE_URL`, creates a Diesel connection manager,
/// and builds a pool with sensible defaults like max size, lifetime, and health checks.
pub fn establish_pool() -> Result<PgPool> {
    // Loads variables from `.env` into the environment.
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").context("DATABASE_URL must be set in .env")?;

    let manager = ConnectionManager::<PgConnection>::new(db_url);

    // Build the pool with configuration.
    let pool = Pool::builder()
        .max_size(15) // max 15 concorrent connections
        .max_lifetime(Some(Duration::from_secs(300))) // drop idle conns after 5 min
        .test_on_check_out(true) // Check if the connection is alive before handing it over.
        .build(manager)
        .context("Failed to build connection pool")?;

    Ok(pool)
}

/// Inserts a new poll into the database (used only for raw insert).
///
/// Not used in the live app since we want to upsert instead,
/// but this demonstrates a basic Diesel insert.
pub fn _insert_poll(pool: &PgPool, poll: NewPoll) -> anyhow::Result<()> {
    // Get a connection from the pool.
    let mut conn = pool
        .get()
        .context("Failed to get DB connection from pool")?;

    // Perform the insert into the `polls` table.
    diesel::insert_into(polls)
        .values(&poll)
        .execute(&mut conn)
        .context("Failed to insert poll into database")?;

    Ok(())
}

/// Inserts or updates a poll using its `poll_id` as the unique key.
///
/// If a poll with the same `poll_id` already exists, it will be updated
/// with the new values. This ensures we always have the latest on-chain state.
pub fn upsert_poll(pool: &PgPool, poll: &NewPoll) -> anyhow::Result<()> {
    // Get a database connection from the pool.
    let mut conn = pool
        .get()
        .context("Failed to get DB connection from pool")?;

    // Perform an upsert: insert if not exists, update otherwise.
    // Diesel requires the column in `on_conflict()` to have a UNIQUE constraint in the DB schema.
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

/// Fetches all stored polls from the database.
///
/// Used in the CLI to display all indexed poll records.
/// Returns a vector of `Poll` structs.
pub fn list_polls(pool: &PgPool) -> anyhow::Result<Vec<Poll>> {
    // Get a connection from the pool.
    let mut conn = pool.get()?;

    // Load all rows from the `polls` table.
    let results = polls.load::<Poll>(&mut conn)?;
    Ok(results)
}
