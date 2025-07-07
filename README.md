# Skyscope Solana MEV Bot - An experimental tool from Skyscope Sentinel Intelligence

Developer: Miss Casey Jay Topojani
Business: Skyscope Sentinel Intelligence


## Overview
A high-frequency trading bot designed to identify and exploit arbitrage opportunities across various decentralized exchanges (DEXs) on the Solana blockchain.
It incorporates features like zero-slot MEV, offline signing, advance nonce management, Jito tip integration, and leverages Solana programs for sandwich attack capabilities (where applicable and ethical).

The bot can run multiple "instances" concurrently, each with its own dedicated Solana keypair and USDT-denominated budget, allowing for diversified strategies or risk management.

## Features

- **Multi-DEX Support**: Works with Raydium (CLMM and standard pools), Orca Whirlpools, and Meteora DEXs.
- **Real-time Pool Monitoring**: Continuously scans for new liquidity pools.
- **Advanced Arbitrage Detection**: Identifies profitable 1-hop and 2-hop arbitrage paths.
- **Simulation Engine**: Tests potential trades before execution to estimate profitability.
- **Configurable Automated Execution**: Automatically executes profitable trades based on user-defined thresholds.
- **Multi-Instance Concurrent Operation**: Supports running up to 4 independent bot instances, each with its own keypair and budget, operating concurrently.
- **Budget Management**: Each instance operates within a specified USDT budget, converted to a SOL cap for spending.
- **Direct On-Chain Execution or External Dispatch**: Choose between direct transaction submission or sending trade details to an external executor.
- **Performance Tracking**: Records all arbitrage attempts and results, typically to MongoDB (instance-specific collections for trades).
- **Verbose Real-time CLI**: Offers a modern, user-friendly command-line interface with real-time verbose output, including current tasks, instance IDs, and profits (in SOL and estimated USDT equivalent).

## Supported DEXs

- Raydium (CLMM and standard pools)
- Orca Whirlpools
- Meteora

## Code Structure
```
src/
├── arbitrage/
│ ├── calc_arb.rs # Arbitrage calculation logic
│ ├── simulate.rs # Trade simulation
│ ├── streams.rs # Real-time data streams
│ └── types.rs # Data structures
├── markets/
│ ├── meteora.rs # Meteora DEX integration
│ ├── orca_whirpools.rs # Orca integration
│ ├── raydium.rs # Raydium integration
│ └── types.rs # Market data structures
└── common/ # Shared utilities, constants, and budget manager
```

## Key Components

### Pool Discovery
```rust
pub async fn get_fresh_pools(tokens: Vec<TokenInArb>) -> HashMap<String, Market> {
    // Scans supported DEXs for new pools containing specified tokens
    // Implements rate limiting between requests
}
```

## Configuration

The bot is primarily configured using environment variables. Create a `.env` file in the root directory of the project or set environment variables directly in your deployment environment.

### Single Instance Mode (Default / Legacy)

If you only want to run a single bot instance, you can use the primary `PAYER_KEYPAIR_PATH`:

- **`PAYER_KEYPAIR_PATH`**: Filesystem path to the Solana keypair JSON file for the main bot instance. (e.g., `/path/to/your/main_wallet.json`)
- **`DEFAULT_BUDGET_USDT`** (Optional, default: `3.0`): The operational budget in USDT for the single instance if no multi-instance configuration is provided.

### Multi-Instance Mode (Recommended for multiple strategies/risk levels)

To run multiple bot instances (up to 4), configure each one:

- **`BOT_INSTANCE_1_KEYPAIR_PATH`**: Path to the keypair file for bot instance 1.
- **`BOT_INSTANCE_1_BUDGET_USDT`** (Optional, default: `3.0`): Budget in USDT for instance 1.
- **`BOT_INSTANCE_2_KEYPAIR_PATH`**: Path to the keypair file for bot instance 2.
- **`BOT_INSTANCE_2_BUDGET_USDT`** (Optional, default: `3.0`): Budget in USDT for instance 2.
- ...and so on, up to `BOT_INSTANCE_4_KEYPAIR_PATH` and `BOT_INSTANCE_4_BUDGET_USDT`.

**Note**: If `BOT_INSTANCE_1_KEYPAIR_PATH` is set, the bot will operate in multi-instance mode. If it's not set, it will fall back to single-instance mode using `PAYER_KEYPAIR_PATH`.

### Common Configuration Variables:

These apply whether running in single or multi-instance mode:

- **RPC Endpoints**:
    - `RPC_URL`: Main Solana RPC endpoint. (e.g., `https://api.mainnet-beta.solana.com`)
    - `RPC_URL_TX`: (Optional) RPC endpoint for sending transactions. Defaults to `RPC_URL`.
    - `WSS_RPC_URL`: WebSocket RPC endpoint. (e.g., `wss://api.mainnet-beta.solana.com`)
- **Database**:
    - `DATABASE_NAME`: Name of the MongoDB database. (e.g., `mev_bot_db`)
- **Trading Parameters & Automation (Global for all instances)**:
    - `PROFIT_THRESHOLD_SOL` (Optional, default: `0.02`): Minimum SOL profit to trigger execution.
    - `DIRECT_EXECUTION` (Optional, default: `false`): `true` for direct execution, `false` for TCP dispatch.
- **Logging**:
    - `LOG_LEVEL` (Optional, default: `Info`): Log verbosity (`Error`, `Warn`, `Info`, `Debug`, `Trace`).

### Example `.env` for Multi-Instance:
```env
# Instance 1
BOT_INSTANCE_1_KEYPAIR_PATH=./instance1_wallet.json
BOT_INSTANCE_1_BUDGET_USDT=5.0

# Instance 2
BOT_INSTANCE_2_KEYPAIR_PATH=./instance2_wallet.json
BOT_INSTANCE_2_BUDGET_USDT=2.5

# Common Settings
RPC_URL=https://api.mainnet-beta.solana.com
WSS_RPC_URL=wss://api.mainnet-beta.solana.com
DATABASE_NAME=skyscope_mev_results
PROFIT_THRESHOLD_SOL=0.01
DIRECT_EXECUTION=true
LOG_LEVEL=Info
```

## Setting Up Keypairs for Bot Instances

It is **highly recommended** to use separate, dedicated Solana wallets (keypairs) for each bot instance, funded only with the capital you intend for that instance to use. This isolates risk. **Do NOT use your primary personal wallet seed phrase or keypair directly with any bot.**

**How to Generate a New Keypair File:**

1.  **Install Solana CLI**: If you haven't already, install the Solana Command Line Tools from [https://docs.solana.com/cli/install](https://docs.solana.com/cli/install).
2.  **Generate Keypair**: Open your terminal or command prompt and run:
    ```bash
    solana-keygen new --outfile ~/new_bot_wallet.json
    ```
    - Replace `~/new_bot_wallet.json` with your desired path and filename (e.g., `./instance1_wallet.json` in your project directory).
    - This command will output a public key (your new wallet address) and a seed phrase. **Store this seed phrase securely and offline.** You'll need it if you ever need to recover this keypair. The `.json` file is your keypair file.
3.  **Get Public Key (Wallet Address)**: If you need the public key again from the file:
    ```bash
    solana-keygen pubkey ~/new_bot_wallet.json
    ```
4.  **Fund the New Wallet**: Transfer SOL (and any other tokens the bot might need, like USDC for fees if applicable, though typically SOL is used for fees) to the public key (wallet address) you just generated. Start with a small amount for testing.
5.  **Configure the Bot**: Update your `.env` file with the path to this new keypair file (e.g., `BOT_INSTANCE_1_KEYPAIR_PATH=./instance1_wallet.json`).

Repeat these steps for each bot instance you want to run.

## Performance Optimization

The bot includes several optimization features:

- Batch processing with `get_multiple_accounts` for efficient RPC usage.
- Market filtering based on liquidity thresholds.
- Real-time WebSocket subscriptions for immediate market updates (where applicable).
- MongoDB for persistent storage and performance analysis.
- Error rate limiting for problematic paths to avoid spamming RPCs or DEXs.

## Monitoring

The bot outputs:

- **Real-time Verbose CLI**: Displays current tasks, [Instance ID] tags, simulated profits (in SOL and USDT equivalent), and status messages.
- **Progress Bars**: For long-running tasks like scanning paths.
- **Detailed Logs**: Arbitrage opportunities, successful trades, and errors are logged with timestamps and module information. Log files (`program.log`, `errors.log`) are stored in the `logs/` directory.
- **JSON Trade Files**: Details of potentially profitable trades identified are often saved to JSON files (e.g., in `optimism_transactions/instance_{ID}/`), especially when not using direct execution.
- **MongoDB Integration**: For persistent storage and analysis of arbitrage attempts and results (collections may be instance-specific).

## Disclaimer

This is experimental software. Use at your own risk. Skyscope Sentinel Intelligence and Developer Miss Casey Jay Topojani are not responsible for any funds lost while using this bot. Ensure you understand the risks involved in MEV activities and trading on decentralized exchanges. Always start with small amounts of capital in dedicated wallets.

## License
The license for this software is typically found in a `LICENSE` file in the repository. If not present, assume it is proprietary unless otherwise stated.
```
