use anyhow::{Context, Ok};
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

// Descriminator obtained from the IDL
const POLL_DISCRIMINATOR: [u8; 8] = [110, 234, 167, 188, 231, 136, 153, 111];
const POOL_CANDIDATE_DISCRIMINATOR: [u8; 8] = [86, 69, 250, 96, 193, 10, 222, 123];
const VOTE_DISCRIMINATOR: [u8; 8] = [241, 93, 35, 191, 254, 147, 17, 202];
const URL: &'static str = "wss://api.devnet.solana.com/";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //Connect to the RPC
    let client = PubsubClient::new(URL)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to connect to PubsubClient")?;

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

    let (mut stream, _unsubscribe) = client
        .program_subscribe(&program_id, Some(config))
        .await
        .unwrap();

    println!("Listening for state changes to program: {}", program_id);

    tokio::select! {
        _ = async {
            while let Some(response) = stream.next().await {
                handle_response(response);
            }
        } => {}

        _ = signal::ctrl_c() => {
            println!("Ctrl+C received, shutting down...");
        }
    }

    //Stream needs to be dropepds as its borrowing client
    drop(stream);
    client.shutdown().await?;
    println!(" clean exit");
    Ok(())
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
