# Skyscope Solana MEV Bot (v1.0)

A secure, modular, high-performance trading bot that automatically captures Maximal Extractable Value (MEV) opportunities on the Solana blockchain.  
The bot combines advanced arbitrage techniques from the open-source ecosystem with enterprise-grade security—every launch is gated by a 4-digit PIN and your wallets are stored only in locally-encrypted keystore files.

---

## Key Features
1. **Multi-DEX MEV Engine**  
   • Atomic flash-loan arbitrage (Raydium, Orca, Jupiter, Meteora)  
   • Sandwich trading, liquidity sniping & classic 1-/2-hop arbitrage  
   • Real-time mempool monitoring (Yellowstone gRPC)  
   • Jito bundle & priority-fee support  
2. **Concurrent Wallet Instances** – run several isolated bots, each with its own encrypted keypair and budget.  
3. **Rich CLI & TUI** – interactive terminal app plus command-line sub-commands (`--show-funding-qr`, `--withdraw-funds`, etc.).  
4. **Adaptive Risk Management** – max loss %, max trade size, token / pool blacklists, DEX priority weights.  
5. **Comprehensive Logs & Metrics** – JSON & human-readable logs, trade history, P/L, success ratio.  

---

## Security Highlights 🔒
| Layer | Mechanism |
|-------|-----------|
| App Launch | 4-digit **PIN protection** (Argon2 hashed, salted, exponential back-off after failures). |
| Keystore | Wallets are stored **encrypted with XChaCha20-Poly1305**, unlockable only with the PIN-derived key. |
| No Seed Storage | Seed phrases are never written to disk; you can import a seed once and it is immediately converted into an encrypted keypair. |
| Session Control | Automatic session expiry (default 15 min) & manual logout. |
| Optional Hardware | Works with Ledger-exported JSON keypairs or external Sign-In-With-Solana providers. |

---

## Installation & First-Time Setup

```bash
# 1. Install Rust & Solana-CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# 2. Clone & build
git clone https://github.com/skyscope-sentinel/Skyscope-Solana-MEV-Bot.git
cd Skyscope-Solana-MEV-Bot
cargo build --release
```

### Initial Launch
```bash
cargo run --release
```
The bot will:
1. Ask you to create a **4-digit PIN** (first run only).  
2. Generate the local keystore directory: `~/.skyscope/keystore`.  
3. Walk you through creating or importing your first wallet.

---

## Usage Guide

### Interactive Mode (recommended)
```bash
cargo run --release
```
• Navigate menus to manage wallets, configure trading parameters, and start/stop the bot.  
• Live dashboard shows running time, trades executed, and total profit in SOL.

### Quick CLI Tasks
| Task | Command |
|------|---------|
| Display QR codes for funding wallets | `cargo run --release -- --show-funding-qr` |
| Withdraw funds interactively | `cargo run --release -- --withdraw-funds` |
| Init a wallet non-interactively | `cargo run --release init-wallet --name mybot` |
| Import a keypair file | `cargo run --release import-wallet --name mybot --file ~/mykeypair.json` |

---

## Trading Capabilities

* **Strategies** – MEV arbitrage, sandwich, flash-loan, liquidity sniping.  
* **DEX Coverage** – Raydium, Orca, Jupiter, Meteora (plug-in adapters).  
* **Flash-Loans** – Optional, executed only on supporting DEXs (Orca, Meteora).  
* **Concurrency** – Up to 4 concurrent trade threads per instance, adjustable.  
* **Priority Fees** – Auto-tuned based on recent slot congestion & profit target.  

---

## Configuration Options (`.env` + in-app)

| Variable | Description | Example |
|----------|-------------|---------|
| `RPC_URL` | Primary RPC endpoint | `https://api.mainnet-beta.solana.com` |
| `BOT_INSTANCE_1_KEYPAIR_PATH` | Path to encrypted keystore file | `~/.skyscope/keystore/trader1.keystore` |
| Trading Params (UI) | Amount in SOL, max slippage %, max concurrent trades, strategy, DEX list |
| Risk | Max loss %, max trade size (SOL), token/pool blacklist |
| Session | Timeout minutes (5-60) |
| DEX Priority | 1-10 weight per DEX |

_Edit these via the Settings menus or by editing the generated `.env`._

---

## Security Best Practices
1. **Never reuse your trading PIN elsewhere.**  
2. **Back-up keystore files** plus note the wallet public keys; losing the keystore without a seed backup = funds lost.  
3. Keep your system clock synced; Solana rejects signatures outside its slot window.  
4. Prefer a dedicated machine / VPS; disable unnecessary services & keep OS patched.  
5. Test on **devnet** with small amounts before mainnet deployment.  
6. Monitor logs; Unexpected errors or rapid balance changes → stop the bot and investigate.  

---

## License
MIT License © 2025 Skyscope Sentinel Intelligence.

## Disclaimer
This software is **experimental & provided “as-is”**. Trading cryptocurrencies involves substantial risk; past performance is not indicative of future results. The authors are **not liable for losses** arising from the use or misuse of this code.  
By running the bot you agree to comply with all applicable laws and exchange terms of service in your jurisdiction.
