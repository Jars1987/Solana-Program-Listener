use anyhow::Result;
use clap::{command, Parser, Subcommand};
use voting_dapp_listener::db::db::{establish_pool, list_polls};
use voting_dapp_listener::db::models::Poll;

/// CLI for querying indexed poll data
#[derive(Parser)]
#[command(name = "Voting DApp CLI")]
#[command(about = "Query the indexed poll data", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all polls
    ListPolls,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ListPolls => {
            let pool = establish_pool()?;
            let polls: Vec<Poll> = list_polls(&pool)?;

            for p in polls {
                println!(
                    "🗳️ Poll #{}: {} | {} → {}",
                    p.poll_id, p.poll_name, p.poll_start, p.poll_end
                );
            }
        }
    }

    Ok(())
}
