# Skyscope Solana MEV Bot - An experimental tool from Skyscope Sentinel Intelligence

Developer: Miss Casey Jay Topojani
Business: Skyscope Sentinel Intelligence


## Overview
A high-frequency trading bot designed to identify and exploit arbitrage opportunities across various decentralized exchanges (DEXs) on the Solana blockchain.
It incorporates features like zero-slot MEV, offline signing, advance nonce management, Jito tip integration, and leverages Solana programs for sandwich attack capabilities (where applicable and ethical).

## Features

- **Multi-DEX Support**: Works with Raydium (CLMM and standard pools), Orca Whirlpools, and Meteora DEXs.
- **Real-time Pool Monitoring**: Continuously scans for new liquidity pools.
- **Advanced Arbitrage Detection**: Identifies profitable 1-hop and 2-hop arbitrage paths.
- **Simulation Engine**: Tests potential trades before execution to estimate profitability.
- **Configurable Automated Execution**: Automatically executes profitable trades based on user-defined thresholds, with options for direct on-chain execution or dispatch to an external executor.
- **Performance Tracking**: Records all arbitrage attempts and results, typically to MongoDB.
- **Verbose Real-time CLI**: Offers a modern, user-friendly command-line interface with real-time verbose output, including current tasks and profits (in SOL and estimated USDT equivalent).

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
└── common/ # Shared utilities and constants
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

Key configuration options settable via environment variables:

- **RPC Endpoints**:
    - `RPC_URL`: Main Solana RPC endpoint for fetching data and general operations. (e.g., `https://api.mainnet-beta.solana.com`)
    - `RPC_URL_TX`: (Optional) RPC endpoint specifically for sending transactions. If not set, `RPC_URL` is used. (e.g., `https://api.mainnet-beta.solana.com`)
    - `WSS_RPC_URL`: WebSocket RPC endpoint for real-time updates. (e.g., `wss://api.mainnet-beta.solana.com`)
- **Wallet**:
    - `PAYER_KEYPAIR_PATH`: Filesystem path to the Solana keypair JSON file used for signing transactions. (e.g., `/path/to/your/wallet.json`)
- **Database**:
    - `DATABASE_NAME`: Name of the MongoDB database where the bot stores its findings and results. (e.g., `mev_bot_db`)
- **Trading Parameters & Automation**:
    - `PROFIT_THRESHOLD_SOL` (Optional, default: `0.02`): The minimum estimated profit in SOL required for the bot to consider executing an arbitrage trade. Example: `PROFIT_THRESHOLD_SOL=0.05` for a 0.05 SOL threshold.
    - `DIRECT_EXECUTION` (Optional, default: `false`): Controls how trades are executed.
        - Set to `true` to have the bot attempt to execute profitable trades directly on-chain.
        - If `false` (default), profitable trades are saved to a JSON file, and a message containing the file path is sent to a local TCP server (expected at `127.0.0.1:8080`) for handling by an external execution mechanism.
        Example: `DIRECT_EXECUTION=true`
- **Logging**:
    - `LOG_LEVEL` (Optional, default: `Info`): Controls the verbosity of the bot's own log messages. This helps in monitoring and debugging. Possible values include: `Error`, `Warn`, `Info`, `Debug`, `Trace`. Example: `LOG_LEVEL=Debug`

Certain less frequently changed parameters, such as specific DEX program IDs or internal algorithm constants, are still located within the codebase, primarily in `src/common/constants.rs` and the `src/markets/` directory.

Example `.env` file:
```env
RPC_URL=https://api.mainnet-beta.solana.com
RPC_URL_TX=https://api.mainnet-beta.solana.com
WSS_RPC_URL=wss://api.mainnet-beta.solana.com
PAYER_KEYPAIR_PATH=./my_wallet.json
DATABASE_NAME=skyscope_mev_results
PROFIT_THRESHOLD_SOL=0.03
DIRECT_EXECUTION=true
LOG_LEVEL=Info
```

## Performance Optimization

The bot includes several optimization features:

- Batch processing with `get_multiple_accounts` for efficient RPC usage.
- Market filtering based on liquidity thresholds.
- Real-time WebSocket subscriptions for immediate market updates (where applicable).
- MongoDB for persistent storage and performance analysis.
- Error rate limiting for problematic paths to avoid spamming RPCs or DEXs.

## Monitoring

The bot outputs:

- **Real-time Verbose CLI**: Displays current tasks, simulated profits (in SOL and USDT equivalent), and status messages.
- **Progress Bars**: For long-running tasks like scanning paths.
- **Detailed Logs**: Arbitrage opportunities, successful trades, and errors are logged with timestamps and module information. Log files (`program.log`, `errors.log`) are stored in the `logs/` directory.
- **JSON Trade Files**: Details of potentially profitable trades identified are often saved to JSON files, especially when not using direct execution.
- **MongoDB Integration**: For persistent storage and analysis of arbitrage attempts and results.

## Disclaimer

This is experimental software. Use at your own risk. Skyscope Sentinel Intelligence and Developer Miss Casey Jay Topojani are not responsible for any funds lost while using this bot. Ensure you understand the risks involved in MEV activities and trading on decentralized exchanges.

## License
The license for this software is typically found in a `LICENSE` file in the repository. If not present, assume it is proprietary unless otherwise stated.

```
This Markdown file contains all the documentation you provided about the bot's configuration, performance optimization, monitoring capabilities, and disclaimer.
```
