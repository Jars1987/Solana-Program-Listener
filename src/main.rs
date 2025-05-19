use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_response::{Response, RpcKeyedAccount},
};
use solana_sdk::pubkey;
use tokio::{self, sync::Notify};

mod state;
use state::pool::Poll;

// Descriminator obtained from the IDL
const POLL_DISCRIMINATOR: [u8; 8] = [110, 234, 167, 188, 231, 136, 153, 111];
const POOL_CANDIDATE_DISCRIMINATOR: [u8; 8] = [86, 69, 250, 96, 193, 10, 222, 123];
const VOTE_DISCRIMINATOR: [u8; 8] = [241, 93, 35, 191, 254, 147, 17, 202];

#[tokio::main]
async fn main() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let notify = Arc::new(Notify::new());

    let shutdown_clone = shutdown.clone();
    let notify_clone = notify.clone();

    // Replace with your deployed voting program ID
    let program_id = pubkey!("HH6z4hgoYg2ZsSkceAUxPZUJdWt8hLqUm1SoEmWqYhPh");

    //important to set the encoding in the config other wise it might lead to unknown behaviour
    let config = RpcProgramAccountsConfig {
        filters: None,
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..Default::default()
        },
        with_context: None,
        sort_results: None,
    };

    let (client, receiver) = match PubsubClient::program_subscribe(
        "wss://api.devnet.solana.com/",
        &program_id,
        Some(config),
    ) {
        Ok((client, receiver)) => (client, receiver),
        Err(err) => {
            eprintln!("Subscription failed: {:?}", err);
            return;
        }
    };

    println!("Listening for state changes to program: {}", program_id);

    // Spawn blocking receiver loop
    let recv_handle = tokio::task::spawn_blocking(move || {
        while !shutdown_clone.load(Ordering::SeqCst) {
            match receiver.recv() {
                Ok(response) => handle_response(response),
                Err(_) => break, // channel closed
            }
        }
        notify_clone.notify_one(); // notify main that receiver finished
    });

    // Wait for ctrl+c or receiver to finish
    let shutdown_reason = tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Ctrl+C pressed");
            shutdown.store(true, Ordering::SeqCst);

            if let Err(e) = client.send_unsubscribe() {
             eprintln!("Unsubscribe failed: {:?}", e);
         }
            // Wait for receiver loop to finish
            notify.notified().await;
            "Ctrl+C"
        },
        _ = recv_handle => "Receiver Closed"
    };

    println!("Shutting down due to: {}", shutdown_reason);

    if let Err(e) = client.send_unsubscribe() {
        eprintln!("Unsubscribe failed: {:?}", e);
    }

    println!("Unsubscribed and exiting.");
}

fn handle_response(response: Response<RpcKeyedAccount>) {
    let account = response.value.account;
    let data = account.data.decode();

    if let Some(acc_data) = data {
        if acc_data.len() >= 8 {
            let account_type = match_voting_account_type(&acc_data[..8]);
            match account_type {
                VotingAccountType::Poll => {
                    if let Some(poll) = decode_poll(&acc_data) {
                        //then print
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
