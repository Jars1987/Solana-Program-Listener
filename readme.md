# 🗳️ Solana Voting Program Listener (Learning Edition)

This Rust project demonstrates how to listen for real-time state changes in a
Solana program using WebSockets. It decodes on-chain account data (like polls,
candidates, and votes) using known Anchor discriminators and prints it to the
console.

| ⚠️ Note: This project is for learning purposes only. It’s a minimal example
meant to help you understand how to use Solana WebSockets in Rust, including
account decoding, signal handling, and async | patterns. It is not
production-ready.

## ✨ What You’ll Learn

How to subscribe to Solana program account updates using WebSockets

How to decode on-chain account data using Anchor-style discriminators

How to use tokio, PubsubClient, and async Rust patterns

## 🔧 Setup

Prerequisites Rust (2021 edition)

Solana CLI (configured to Devnet or your desired cluster)

Your program deployed on-chain (or use the existing Voting Program for testing)

Anchor IDL (to extract the discriminators and struct layout)

1. Clone the Repo

```bash
git clone https://github.com/your-repo/voting-listener.git
cd voting-listener
```

2. Update Program ID In main.rs, replace the placeholder with your deployed
   voting program’s ID:

```
let program_id = pubkey!("HH6z4hgoYg2ZsSkceAUxPZUJdWt8hLqUm1SoEmWqYhPh");

```

3. Add Your Struct In state/pool.rs, we defined a Poll struct and implemented
   try_from_anchor_bytes() for decoding but you can adapt to your own:

```
use solana_sdk::pubkey::Pubkey;

#[derive(Debug)]
pub struct Poll {
    pub poll_id: u64,
    pub poll_owner: Pubkey,
    pub poll_name: String,
    pub poll_description: String,
    pub poll_start: u64,
    pub poll_end: u64,
    pub candidate_amount: u32,
    pub candidate_winner: Pubkey,
}

impl Poll {
    pub fn try_from_anchor_bytes(data: &[u8]) -> Option<Self> {
        // Implement manual parsing here based on Anchor layout
        todo!()
    }
}
```

🚀 Run the Listener

```
cargo run
```

Example output:

```bash
Listening for state changes to program: HH6z4hgo...

New Poll account updated:
ID: 12
Owner: F7x...
Name: Do you like Rust?
Description: Absolutely.
Start: 1716220000
End: 1716223600
Candidates: 3
Winner: 11111111111111111111111111111111
```

## 🧠 Notes

Anchor accounts start with an 8-byte discriminator. These help you identify the
account type.

The program uses UiAccountEncoding::Base64 to ensure correct decoding of binary
data.

Candidate and Vote accounts are detected but not yet decoded. You can extend the
logic similarly.

## 📜 License

MIT
