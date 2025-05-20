use anyhow::Result;
use clap::{command, Parser, Subcommand};
use voting_dapp_listener::db::db::{establish_pool, list_polls};
use voting_dapp_listener::db::models::Poll;

/// CLI for querying indexed poll data from the PostgreSQL database.
/// This CLI interfaces with the off-chain indexer database populated by the listener.
#[derive(Parser)]
#[command(name = "Voting DApp CLI")]
#[command(about = "Query the indexed poll data", long_about = None)]
struct Cli {
    /// The root command, which delegates to subcommands (e.g., list, query, etc.)
    #[command(subcommand)]
    command: Commands,
}

/// Enum representing available subcommands for the CLI.
/// Each variant becomes a CLI command, e.g., `voting-dapp-cli list-polls`
#[derive(Subcommand)]
enum Commands {
    /// Fetch and list all polls currently stored in the local database
    ListPolls,
}

#[tokio::main]
async fn main() -> Result<()> {
    //    Parse command-line arguments into the `Cli` struct using `clap`
    //    This automatically handles `--help`, argument errors, etc.
    let cli = Cli::parse();

    //Dispatch based on the subcommand provided by the user
    match cli.command {
        Commands::ListPolls => {
            //     Establish a connection pool to the Postgres database
            //     Uses environment variable DATABASE_URL (.env) via Diesel
            let pool = establish_pool()?;
            //Query all polls from the DB using Diesel
            let polls: Vec<Poll> = list_polls(&pool)?;
            //Print results in a user-friendly format
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
