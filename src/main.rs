use anyhow::{Context, Result};
use futures::StreamExt;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_response::{Response, RpcKeyedAccount},
};
use solana_sdk::pubkey;
use tokio::{self, signal};

mod state;
use state::pool::Poll;
mod db;
use db::db::{establish_pool, upsert_poll, PgPool};
use db::models::NewPoll;

// Descriminator obtained from the IDL
const POLL_DISCRIMINATOR: [u8; 8] = [110, 234, 167, 188, 231, 136, 153, 111];
const POOL_CANDIDATE_DISCRIMINATOR: [u8; 8] = [86, 69, 250, 96, 193, 10, 222, 123];
const VOTE_DISCRIMINATOR: [u8; 8] = [241, 93, 35, 191, 254, 147, 17, 202];
const URL: &'static str = "wss://api.devnet.solana.com/";

#[tokio::main]
async fn main() -> Result<()> {
    // Step 1: Connect to Solana RPC WebSocket server using the async PubsubClient.
    // This client manages a WebSocket connection to listen for events (e.g. account updates).
    // Unlike the blocking version, this is fully async and cancelable
    let client = PubsubClient::new(URL)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to connect to PubsubClient")?;

    let db_pool = establish_pool()?;

    // Step 2: Define the Program ID you want to listen to.
    // This is the public key of the on-chain Solana program you're interested in (e.g. a voting dApp).
    // Only accounts owned by this program will trigger updates via `program_subscribe`.
    let program_id = pubkey!("HH6z4hgoYg2ZsSkceAUxPZUJdWt8hLqUm1SoEmWqYhPh");

    // Step 3: Define the subscription config for program accounts.
    // Without explicitly setting Base64 encoding, account data may come back as "legacy" format,
    // or be inconsistently decoded (leading to decode errors).
    // Other options (like filters, context, and sorting) are left default or None here.
    let config = RpcProgramAccountsConfig {
        filters: None,
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..Default::default()
        },
        with_context: None,
        sort_results: None,
    };

    // Step 4: Subscribe to program-owned accounts using `program_subscribe`.
    // Returns:
    // - `stream`: a `futures::Stream` of account changes (as `RpcResponse<RpcKeyedAccount>`)
    // - `_unsubscribe`: a closure to manually unsubscribe (not used here)
    //
    // If subscription fails (e.g. network issue, bad program ID), the error is wrapped in context.
    let (mut stream, _unsubscribe) = client
        .program_subscribe(&program_id, Some(config))
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to subscribe to the program")?;

    println!("Listening for state changes to program: {}", program_id);

    // Step 5: Use `tokio::select!` to wait for either:
    // 1. The `stream` finishing (due to RPC server closing connection)
    // 2. The user pressing Ctrl+C (for graceful shutdown)
    tokio::select! {
        // Loop over incoming updates (stream is an async stream of account changes)
        // As long as messages are coming in, this loop runs and processes them one by one.
        _ = async {
            while let Some(response) = stream.next().await {
                // Process each account update (e.g. decode poll state and print info)
                handle_response(response, &db_pool);
            }
        } => {}
        // If Ctrl+C is received, we break the listener loop and begin shutdown.
        _ = signal::ctrl_c() => {
            println!("Ctrl+C received, shutting down...");
        }
    }

    // Step 6: Drop the stream before shutting down the client.
    // Important: the stream borrows from `client`, so we must drop it explicitly
    // to avoid "cannot move out of borrowed value" compiler error.
    drop(stream);
    // Step 7: Gracefully shut down the WebSocket connection.
    // This sends the shutdown signal to the internal WebSocket task spawned by `PubsubClient`.
    client.shutdown().await?;
    println!("Good Bye");
    Ok(())
}

/// Handles a single account update message received from the Solana websocket subscription.
/// This function:
/// 1. Decodes the raw account data.
/// 2. Checks the account's type via its 8-byte Anchor discriminator.
/// 3. If it's a `Poll`, it parses the full state and inserts or updates it in the SQL database.
/// 4. Runs the Diesel DB operation in a blocking thread to avoid stalling the async stream.
///
/// # Arguments
/// - `response`: A Solana `RpcResponse` containing the updated account state.
/// - `db_pool`: A reference to the Diesel PostgreSQL connection pool.
fn handle_response(response: Response<RpcKeyedAccount>, db_pool: &PgPool) {
    // Extract the inner Solana account info
    let account = response.value.account;
    // Decode the account data (Base64 → raw Vec<u8>)
    let data = account.data.decode();

    // Only proceed if the decoding worked and we got enough bytes to inspect
    if let Some(acc_data) = data {
        if acc_data.len() >= 8 {
            // Determine the type of Solana account using the first 8 bytes (Anchor discriminator)
            let account_type = match_voting_account_type(&acc_data[..8]);
            match account_type {
                VotingAccountType::Poll => {
                    // If it's a Poll account, try to deserialize the Poll struct
                    if let Some(poll) = decode_poll(&acc_data) {
                        // Build a `NewPoll` struct that matches your SQL schema
                        // This maps the on-chain Poll to a format Diesel understands
                        let new_poll = NewPoll {
                            poll_id: poll.poll_id as i64, // Diesel uses i64 instead of u64
                            poll_owner: poll.poll_owner.to_bytes(),
                            poll_name: poll.poll_name.clone(),
                            poll_description: poll.poll_description.clone(),
                            poll_start: poll.poll_start as i64,
                            poll_end: poll.poll_end as i64,
                            candidate_amount: poll.candidate_amount as i64,
                            candidate_winner: poll.candidate_winner.to_bytes(),
                        };

                        // Clone the r2d2 pool — this is cheap and encouraged.
                        // The pool itself is internally wrapped in an Arc, so clones are safe.
                        let db_pool_clone = db_pool.clone();

                        // Offload DB write to a blocking thread
                        // Diesel is synchronous and would block the async runtime if run here directly.
                        // `spawn_blocking` tells Tokio: "Run this on a dedicated thread."
                        tokio::task::spawn_blocking(move || {
                            if let Err(e) = upsert_poll(&db_pool_clone, &new_poll) {
                                eprintln!("DB insert failed: {:?}", e);
                            }
                        });

                        // These logs are printed **regardless of DB success** (which is decoupled).
                        println!("New Poll account updated:");
                        println!("ID: {}", poll.poll_id);
                        println!("Owner: {}", poll.poll_owner);
                        println!("Name: {}", poll.poll_name);
                        println!("Description: {}", poll.poll_description);
                        println!("Start: {}", poll.poll_start);
                        println!("End: {}", poll.poll_end);
                        println!("Candidates: {}", poll.candidate_amount);
                        println!("Winner: {}", poll.candidate_winner);
                    } else {
                        println!("Could not decode as Poll");
                    }
                }
                // These are stubs for now — you can later implement decoding + DB storage here too
                VotingAccountType::Candidate => {
                    println!("Candidate account update");
                }
                VotingAccountType::Vote => {
                    println!("Voter account update");
                }
                VotingAccountType::Unknown => {
                    println!("Unknown account type.");
                }
            }
        }
    }
}

pub enum VotingAccountType {
    Poll,
    Candidate,
    Vote,
    Unknown,
}
pub fn match_voting_account_type(data: &[u8]) -> VotingAccountType {
    if data.len() < 8 {
        return VotingAccountType::Unknown;
    }

    let mut descriminator = [0u8; 8];
    descriminator.copy_from_slice(data);

    match descriminator {
        POLL_DISCRIMINATOR => VotingAccountType::Poll,
        POOL_CANDIDATE_DISCRIMINATOR => VotingAccountType::Candidate,
        VOTE_DISCRIMINATOR => VotingAccountType::Vote,
        _ => VotingAccountType::Unknown,
    }
}

fn decode_poll(data: &[u8]) -> Option<Poll> {
    println!("started decoding");
    if data.len() < 8 {
        return None;
    }

    let (_discriminator, body) = data.split_at(8);
    Poll::try_from_anchor_bytes(body)
}
