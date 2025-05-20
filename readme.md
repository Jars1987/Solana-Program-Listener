# 🗳️ Solana Voting Program Listener & Indexer (Learning Edition)

This Rust project demonstrates how to build a **real-time on-chain listener**
for a Solana program, decode Anchor-based accounts, and index the data into a
PostgreSQL database using Diesel ORM. It also includes a simple **CLI tool** to
query the indexed data.

> ⚠️ This is a learning-focused project meant to teach you the basics of
> indexing, decoding, WebSockets, SQL integration, and command-line tooling in
> Rust + Solana. It is **not production-ready**, but can be extended.

---

## ✨ What You’ll Learn

✅ How to subscribe to Solana account updates using **WebSockets**  
✅ How to **decode on-chain data** using Anchor discriminators  
✅ How to write decoded data to **PostgreSQL** using **Diesel ORM**  
✅ How to build a simple **CLI** to query your indexed blockchain state  
✅ How to use `tokio`, `spawn_blocking`, and async Rust patterns

---

## 🧠 How It Works

- Connects to the Solana Devnet via `program_subscribe` (WebSockets)
- Filters and decodes specific on-chain accounts (e.g. `Poll`)
- Persists data to a SQL database in real time
- Lets you query stored data using a CLI (built with `clap`)

You can easily replace the `Poll` model with your own program’s state.

---

## 🛠️ Project Setup

### 1. Clone the Project

```bash
git clone https://github.com/your-username/voting-dapp-listener.git
cd voting-dapp-listener
```

2. Configure Your Program Edit src/main.rs:

```
let program_id = pubkey!("HH6z4hgoYg2ZsSkceAUxPZUJdWt8hLqUm1SoEmWqYhPh");

```

Replace this with your own deployed program’s ID. You can also update the struct
in src/state/pool.rs to match your on-chain data.

3. 3. Set Up PostgreSQL Install Postgres:

```bash
brew install postgresql@14
brew services start postgresql
```

Create user and database:

```sql
CREATE USER voting_user WITH PASSWORD 'test123!';
CREATE DATABASE voting_dapp OWNER voting_user;
```

4. Configure .env

Create a file called .env:

```
DATABASE_URL=postgres://voting_user:test123!@localhost/voting_dapp
```

5. Set Up Diesel

Install Diesel CLI:

```bash
cargo install diesel_cli --no-default-features --features postgres
Run the migrations:
```

```bash
diesel setup --migration-dir db/migrations
diesel migration run --migration-dir db/migrations
diesel print-schema --output-file src/db/schema.rs
```

🚀 Run the Listener

```
cargo run --bin voting-dapp-listener

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

🔍 Querying with the CLI

In a new terminal window:

```bash
cargo run --bin cli -- list-polls

```

Example output:

```bash
🗳️ Poll #21: Final Vote | 1747695600000 → 1747785600000
```

## 🧠 Notes

Uses spawn_blocking to safely insert data from async context

WebSocket shutdown is cleanly handled with ctrl_c()

poll_id is used as the unique key for upserts

You can extend the logic for Candidates or Votes

🚧 Optional Extensions Add filters to CLI (e.g. --owner, --active)

Add REST API layer over the SQL database

Index additional account types (candidates, votes)

Export CSVs or generate charts from the DB

## 📜 License

MIT
