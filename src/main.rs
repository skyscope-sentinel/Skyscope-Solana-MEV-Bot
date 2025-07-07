use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path;

use anyhow::Result;
use futures::FutureExt;
use log::info;
use solana_sdk::pubkey::Pubkey;
use tokio::task::JoinSet;
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;
use Skyscope_Solana_MEV_Bot::arbitrage::strategies::{optimism_tx_strategy, run_arbitrage_strategy, sorted_interesting_path_strategy};
use std::sync::Arc;
use clap::Parser;
use Skyscope_Solana_MEV_Bot::arbitrage::strategies::{optimism_tx_strategy, run_arbitrage_strategy, sorted_interesting_path_strategy};
use Skyscope_Solana_MEV_Bot::common::constants::BotInstanceConfig;
use Skyscope_Solana_MEV_Bot::common::database::insert_vec_swap_path_selected_collection;
use Skyscope_Solana_MEV_Bot::common::qr_utils::display_funding_info;
use Skyscope_Solana_MEV_Bot::common::types::InputVec;
use Skyscope_Solana_MEV_Bot::markets::pools::load_all_pools;
use solana_sdk::native_token::sol_to_lamports; // For converting SOL to lamports
use std::io::{self, Write as IoWrite}; // For stdin/stdout
use Skyscope_Solana_MEV_Bot::transactions::create_transaction::{create_ata_extendlut_transaction, ChainType, SendOrSimulate};
use Skyscope_Solana_MEV_Bot::{common::constants::Env, transactions::create_transaction::create_and_send_swap_transaction};
use Skyscope_Solana_MEV_Bot::common::utils::{from_str, get_tokens_infos, setup_logger};
use Skyscope_Solana_MEV_Bot::arbitrage::types::{SwapPathResult, SwapPathSelected, SwapRouteSimulation, TokenInArb, TokenInfos, VecSwapPathSelected};
use Skyscope_Solana_MEV_Bot::markets::types::Dex as DexType; // Renamed to avoid conflict
use rust_socketio::{Payload, asynchronous::{Client, ClientBuilder},};

/// CLI arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    /// Display funding QR codes for configured bot instances and exit
    #[clap(long)]
    show_funding_qr: bool,

    /// Enter interactive fund withdrawal mode
    #[clap(long)]
    withdraw_funds: bool,
}


use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use mongodb::bson::doc;
use mongodb::{Client as MongoDbCLient, options::ClientOptions};

// use MEV_Bot_Solana::common::pools::{load_all_pools, Pool};

#[tokio::main]
async fn main() -> Result<()> {

    //Options
    let simulation_amount = 3500000000; //3.5 SOL
    // let simulation_amount = 1000000000; //1 SOL
    // let simulation_amount = 2000000000; //1 SOL

    let massive_strategie: bool = true;
    let best_strategie: bool = true;
    let optimism_strategie: bool = true;

    //massive_strategie options
    let fetch_new_pools = false;
            // Restrict USDC/SOL pools to 2 markets
    let restrict_sol_usdc = true;

    //best_strategie options
    // let mut path_best_strategie: String = format!("best_paths_selected/SOL-SOLLY.json");
    let mut path_best_strategie: String = format!("best_paths_selected/ultra_strategies/0-SOL-SOLLY-1-SOL-SPIKE-2-SOL-AMC-GME.json");
    
    
    //Optism tx to send
    let optimism_path: String = "optimism_transactions/11-6-2024-SOL-SOLLY-SOL-0.json".to_string();

    // //Send message to Rust execution program
    // let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    // let message = optimism_path.as_bytes();
    // stream.write_all(message).await?;
    // info!("🛜  Sent: {} tx to executor", String::from_utf8_lossy(message));

    let mut inputs_vec = vec![
        InputVec{
            tokens_to_arb: vec![
                TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
                TokenInArb{address: String::from("4Cnk9EPnW5ixfLZatCPJjDB1PUtcRpVVgTQukm9epump"), symbol: String::from("DADDY-ANSEM")},
 
            ],
            include_1hop: true,
            include_2hop: true,
            numbers_of_best_paths: 4,
            // When we have more than 3 tokens it's better to desactivate caused by timeout on multiples getProgramAccounts calls
            get_fresh_pools_bool: false
        },
        InputVec{
            tokens_to_arb: vec![
                TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
                TokenInArb{address: String::from("2J5uSgqgarWoh7QDBmHSDA3d7UbfBKDZsdy1ypTSpump"), symbol: String::from("DADDY-TATE")},

            ],
            include_1hop: true,
            include_2hop: true,
            numbers_of_best_paths: 4,
            // When we have more than 3 tokens it's better to desactivate caused by timeout on multiples getProgramAccounts calls
            get_fresh_pools_bool: false
        },
        InputVec{
            tokens_to_arb: vec![
                TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
                TokenInArb{address: String::from("BX9yEgW8WkoWV8SvqTMMCynkQWreRTJ9ZS81dRXYnnR9"), symbol: String::from("SPIKE")},

            ],
            include_1hop: true,
            include_2hop: true,
            numbers_of_best_paths: 2,
            // When we have more than 3 tokens it's better to desactivate caused by timeout on multiples getProgramAccounts calls
            get_fresh_pools_bool: false
        },
        //////////////
        //////////////
        //////////////
        //////////////
        //////////////
        //////////////
        InputVec{
            tokens_to_arb: vec![
                TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
                TokenInArb{address: String::from("9jaZhJM6nMHTo4hY9DGabQ1HNuUWhJtm7js1fmKMVpkN"), symbol: String::from("AMC")},
                TokenInArb{address: String::from("8wXtPeU6557ETkp9WHFY1n1EcU6NxDvbAggHGsMYiHsB"), symbol: String::from("GME")},
                // TokenInArb{address: String::from("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), symbol: String::from("USDC")},
                // TokenInArb{address: String::from("5BKTP1cWao5dhr8tkKcfPW9mWkKtuheMEAU6nih2jSX"), symbol: String::from("NoHat")},
            ],
            include_1hop: true,
            include_2hop: true,
            numbers_of_best_paths: 4,
            // When we have more than 3 tokens it's better to desactivate caused by timeout on multiples getProgramAccounts calls
            get_fresh_pools_bool: false
        },
        // InputVec{
        //     tokens_to_arb: vec![
        //         TokenInArb{address: String::from("So11111111111111111111111111111111111111112"), symbol: String::from("SOL")}, // Base token here
        //         TokenInArb{address: String::from("8NH3AfwkizHmbVd83SSxc2YbsFmFL4m2BeepvL6upump"), symbol: String::from("TOPG")},
        //     ],
        //     include_1hop: true,
        //     include_2hop: true,
        //     numbers_of_best_paths: 2,
        //     get_fresh_pools_bool: false
        // },
    ];

    dotenv::dotenv().ok();
    setup_logger().unwrap();

    info!("Starting Skyscope Solana MEV Bot");
    info!("Developed by Miss Casey Jay Topojani for Skyscope Sentinel Intelligence");

    let cli_args = CliArgs::parse();
    let env = Env::new();

    if env.bot_instances.is_empty() {
        info!("⚠️ No bot instances configured.");
        info!("   Please set either BOT_INSTANCE_1_KEYPAIR_PATH (for multi-instance mode)");
        info!("   or PAYER_KEYPAIR_PATH (for single-instance mode) in your .env file.");
        info!("   Refer to the 'Quick Start' or 'Setting Up Secure Keypairs' section in README.md for detailed instructions.");
        info!("Exiting.");
        return Ok(());
    }

    if cli_args.show_funding_qr {
        info!("Displaying funding information for configured instances...");
        for instance_conf in &env.bot_instances {
            // Pass &env to the now async function
            if let Err(e) = display_funding_info(&env, instance_conf.id, &instance_conf.payer_keypair_path, instance_conf.budget_usdt).await {
                log::error!("[Instance {}] Failed to display funding info: {}", instance_conf.id, e);
            }
        }
        info!("Exiting after displaying funding QR codes.");
        return Ok(());
    }

    if cli_args.withdraw_funds {
        // Pass a reference to env
        if let Err(e) = interactive_withdrawal_flow(&env).await {
            log::error!("Error during interactive withdrawal: {}", e);
        }
        info!("Exiting after withdrawal attempt / cancellation.");
        return Ok(());
    }

    // Continue with normal bot operation if --show-funding-qr or --withdraw-funds is not present
    info!("⚠️⚠️ New fresh pools fetched on METEORA and RAYDIUM are excluded because a lot of time there have very low liquidity, potentially can be used on subscribe log strategy");
    info!("⚠️⚠️ Liquidity is fetch to API and can be outdated on Radyium Pool");

    let mut set: JoinSet<()> = JoinSet::new();
    
    // // The first token is the base token (here SOL)
    // This is used for global_tokens_infos_arc, ensure it covers all tokens any instance might need.
    let all_tokens_from_inputs_global: Vec<TokenInArb> = inputs_vec.clone().into_iter().flat_map(|input| input.tokens_to_arb).collect();

    info!("Open Socket IO channel...");
    // `env` is already initialized above

    if env.bot_instances.is_empty() { // This check is technically redundant due to the one above, but kept for safety.
        info!("⚠️ No bot instances configured.");
        info!("   Please set either BOT_INSTANCE_1_KEYPAIR_PATH (for multi-instance mode)");
        info!("   or PAYER_KEYPAIR_PATH (for single-instance mode) in your .env file.");
        info!("   Refer to the 'Quick Start' or 'Setting Up Secure Keypairs' section in README.md for detailed instructions.");
        info!("Exiting.");
        return Ok(());
    }
    info!("🤖 Found {} bot instance(s) to manage.", env.bot_instances.len());

    // Global data fetching (once)
    let dexs_arc = if massive_strategie || best_strategie { // best_strategie might also need dexs
        info!("🏊 Launching global pool fetching...");
        let dexs = load_all_pools(fetch_new_pools).await;
        info!("🏊 {} Global DEXs data loaded.", dexs.len());
        Some(std::sync::Arc::new(dexs))
    } else {
        None
    };

    let all_tokens_from_inputs: Vec<TokenInArb> = inputs_vec.clone().into_iter().flat_map(|input| input.tokens_to_arb).collect();
    let global_tokens_infos_arc = if massive_strategie || best_strategie { // best_strategie needs token_infos
        info!("🪙 Fetching global token infos for all specified input tokens...");
        let infos = get_tokens_infos(all_tokens_from_inputs.clone()).await;
        info!("🪙 {} Global token infos loaded.", infos.len());
        Some(std::sync::Arc::new(infos))
    } else {
        None
    };

    // Socket.IO client setup (once)
    // Note: If strategies need to send instance-specific data over socket.io, the client might need to be cloned
    // or managed differently. For now, assuming it's for general non-instance-specific comms if used by strategies.
    let callback = |payload: Payload, socket: Client| {
        async move {
            match payload {
                Payload::String(data) => println!("Received: {}", data), // Consider using log::info!
                Payload::Binary(bin_data) => println!("Received bytes: {:#?}", bin_data), // Consider using log::info!
                Payload::Text(data) => println!("Received Text: {:?}", data), // Consider using log::info!
            }
        }
        .boxed()
    };
    
    let mut socket = ClientBuilder::new("http://localhost:3000") // TODO: Make configurable
        .namespace("/")
        .on("connection", callback)
        .on("error", |err, _| {
            async move { eprintln!("SocketIO Error: {:#?}", err) }.boxed() // Consider log::error!
        })
        .on("orca_quote", callback) // Example event
        .on("orca_quote_res", callback) // Example event
        .connect()
        .await
        .expect("SocketIO connection failed"); // Consider graceful error handling


    for instance_config in &env.bot_instances {
        info!("---------- Starting operations for Bot Instance ID: {} ----------", instance_config.id);
        info!("   Keypair Path: {}", instance_config.payer_keypair_path);
        info!("   Budget: {:.2} USDT", instance_config.budget_usdt);

        // It's important that strategies called below are adapted to use instance_config.payer_keypair_path for transactions
        // and respect instance_config.budget_usdt. This will be part of "Strategy Adaptation" step.

        if massive_strategie {
            if let Some(ref dexs_arc_inner) = dexs_arc {
                if let Some(ref global_tokens_infos_inner) = global_tokens_infos_arc {
                    info!("📈 [Instance {}] Launching arbitrage process (massive_strategie)...", instance_config.id);
                    let mut instance_specific_best_paths: Vec<String> = Vec::new(); // Paths generated by this instance

                    for input_iter in inputs_vec.clone() {
                        let current_tokens_infos: HashMap<String, TokenInfos> = input_iter.tokens_to_arb.iter()
                            .filter_map(|token_in_arb| {
                                global_tokens_infos_inner.get(&token_in_arb.address).map(|info| (token_in_arb.address.clone(), info.clone()))
                            })
                            .collect();

                        if current_tokens_infos.len() != input_iter.tokens_to_arb.len() {
                            log::warn!("[Instance {}] Not all token infos found for an input_vec using symbols {:?}, skipping.", instance_config.id, input_iter.tokens_to_arb.iter().map(|t|&t.symbol).collect::<Vec<_>>());
                            continue;
                        }

                        info!("[Instance {}] Running massive_strategie for tokens: {:?}", instance_config.id, input_iter.tokens_to_arb.iter().map(|t| &t.symbol).collect::<Vec<_>>());

                        // TODO: Adapt run_arbitrage_strategy to take instance_config
                        let result = run_arbitrage_strategy(
                            simulation_amount,
                            input_iter.get_fresh_pools_bool,
                            restrict_sol_usdc,
                            input_iter.include_1hop,
                            input_iter.include_2hop,
                            input_iter.numbers_of_best_paths,
                            dexs_arc_inner.to_vec(),
                            input_iter.tokens_to_arb.clone(),
                            current_tokens_infos.clone()
                            // instance_config.clone() // This will be added
                        ).await;

                        match result {
                            Ok((path_for_best_strategie_local, swap_path_selected)) => {
                                // Store path relative to this instance if needed, or use a naming convention
                                // For now, this path is local to this input_iter processing.
                                instance_specific_best_paths.push(path_for_best_strategie_local.clone());
                                if !path_for_best_strategie_local.is_empty() { // Assuming empty means no good path found / saved
                                     path_best_strategie = path_for_best_strategie_local; // This will be overwritten by last successful one. Needs better handling.
                                }

                                if let Some(profit) = swap_path_selected.profit {
                                    let profit_sol = profit as f64 / 1_000_000_000.0;
                                    match Skyscope_Solana_MEV_Bot::common::utils::get_sol_usdt_price().await {
                                        Ok(usdt_price) => {
                                            let profit_usdt = profit_sol * usdt_price;
                                            info!("📈 [Instance {}] Massive strategy found potential profit: {:.6} SOL (~${:.2} USDT)", instance_config.id, profit_sol, profit_usdt);
                                        }
                                        Err(e) => {
                                            info!("📈 [Instance {}] Massive strategy found potential profit: {:.6} SOL (failed to fetch USDT price: {})", instance_config.id, profit_sol, e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("[Instance {}] Error running massive_strategie for input {:?}: {:?}", instance_config.id, input_iter.tokens_to_arb.iter().map(|t|&t.symbol).collect::<Vec<_>>(), e);
                            }
                        }
                    } // end loop input_iter

                    // The ultra_strat logic needs significant rework if it's to be instance-aware or aggregate instance results.
                    // For now, it's commented out or will use the last `path_best_strategie`.
                    /*
                    if inputs_vec.clone().len() > 1 && !instance_specific_best_paths.is_empty() {
                        // This logic needs to be instance-specific for file names and DB entries
                        info!("[Instance {}] Processing ultra strategies...", instance_config.id);
                        // ... (Adapted ultra_strat logic here using instance_specific_best_paths) ...
                        // path_best_strategie = instance_specific_ultra_strat_path; // update for this instance
                    }
                    */
                } else {
                    info!("[Instance {}] Skipping massive_strategie due to missing global data (DEXs or Token Infos).", instance_config.id);
                }
            } else {
                 info!("[Instance {}] Skipping massive_strategie as it's not enabled.", instance_config.id);
            }
        } // end if massive_strategie

        if best_strategie {
            if let Some(ref global_tokens_infos_inner) = global_tokens_infos_arc {
                // `path_best_strategie` is problematic here as it's globally scoped and overwritten.
                // This strategy should ideally operate on paths generated by *this* instance,
                // or a well-defined shared path if that's the design.
                // Using the global `path_best_strategie` for now, acknowledging this limitation.
                if !path_best_strategie.is_empty() {
                    info!("[Instance {}] Running best_strategie using path: {}...", instance_config.id, path_best_strategie);
                    // TODO: Adapt sorted_interesting_path_strategy for instance_config
                    let _ = sorted_interesting_path_strategy(
                        simulation_amount,
                        path_best_strategie.clone(),
                        all_tokens_from_inputs.clone(), // This should ideally be instance specific tokens or based on the path
                        global_tokens_infos_inner.as_ref().clone()
                        // instance_config.clone() // This will be added
                    ).await;
                } else {
                    info!("[Instance {}] Skipping best_strategie as no best_path_file was determined.", instance_config.id);
                }
            } else {
                info!("[Instance {}] Skipping best_strategie due to missing global Token Infos.", instance_config.id);
            }
        } // end if best_strategie

        if optimism_strategie {
            // Similar to best_strategie, optimism_path is global.
            // If this strategy should be run by each instance with its own funds, it's fine.
            // If the optimism_path is specific to one instance's state, this needs adjustment.
            if !optimism_path.is_empty(){
                info!("[Instance {}] Running optimism_tx_strategy with path: {}...", instance_config.id, optimism_path);
                // TODO: Adapt optimism_tx_strategy for instance_config
                let _ = optimism_tx_strategy(
                    optimism_path.clone()
                    // instance_config.clone() // This will be added
                ).await;
            } else {
                info!("[Instance {}] Skipping optimism_strategie as no optimism_path was provided.", instance_config.id);
            }
        } // end if optimism_strategie

        info!("---------- Finished operations for Bot Instance ID: {} ----------", instance_config.id);
    } // end loop over bot_instances

    while let Some(res) = set.join_next().await {
        info!("JoinSet task completed: {:?}", res);
    }

    info!("🏁 Skyscope Solana MEV Bot finished all configured operations.");
use Skyscope_Solana_MEV_Bot::transactions::transfer_sol::transfer_sol_manager;

// ... (other use statements) ...

async fn interactive_withdrawal_flow(env: &Env) -> Result<()> {
    info!("Entering interactive fund withdrawal mode...");
    if env.bot_instances.is_empty() {
        info!("No bot instances configured. Nothing to withdraw from.");
        return Ok(());
    }

    println!("\nAvailable Bot Instances for Withdrawal:");
    for (idx, instance) in env.bot_instances.iter().enumerate() {
        println!("{}. Instance ID: {} (Keypair: {})", idx + 1, instance.id, instance.payer_keypair_path);
    }

    let selected_idx: usize = loop {
        print!("\nSelect instance number to withdraw from (or 0 to cancel): ");
        io::stdout().flush()?;
        let mut selected_idx_str = String::new();
        io::stdin().read_line(&mut selected_idx_str)?;
        match selected_idx_str.trim().parse::<usize>() {
            Ok(0) => {
                println!("Withdrawal cancelled.");
                return Ok(());
            }
            Ok(num) if num > 0 && num <= env.bot_instances.len() => break num - 1,
            _ => {
                println!("Invalid selection. Please enter a number from the list or 0 to cancel.");
            }
        };
    };

    let instance_config = &env.bot_instances[selected_idx];
    info!("[Instance {}] Selected for withdrawal.", instance_config.id);

    let keypair = solana_sdk::signature::read_keypair_file(&instance_config.payer_keypair_path)
        .map_err(|e| anyhow::anyhow!("Failed to read keypair for instance {}: {}", instance_config.id, e))?;
    let pubkey = keypair.pubkey();

    let current_sol_balance = Skyscope_Solana_MEV_Bot::common::budget_manager::get_sol_balance(&env.rpc_url, &pubkey).await?;
    let sol_price_usdt = Skyscope_Solana_MEV_Bot::common::utils::get_sol_usdt_price().await.unwrap_or(0.0);
    let balance_usdt = current_sol_balance * sol_price_usdt;
    println!("\nInstance {} ({}) Balance: {:.9} SOL (~${:.2} USDT)", instance_config.id, pubkey, current_sol_balance, balance_usdt);

    if current_sol_balance < 0.000005 { // Minimum for a transaction
        println!("Insufficient balance to perform a withdrawal (less than 0.000005 SOL).");
        return Ok(());
    }

    let dest_pubkey: Pubkey = loop {
        print!("\nEnter destination Solana address: ");
        io::stdout().flush()?;
        let mut dest_address_str = String::new();
        io::stdin().read_line(&mut dest_address_str)?;
        match dest_address_str.trim().parse::<Pubkey>() {
            Ok(pk) => break pk,
            Err(_) => {
                println!("Invalid destination address. Please try again.");
            }
        };
    };

    let amount_to_transfer_sol: f64 = loop {
        println!("\nSelect amount to transfer:");
        println!("  1. 25%");
        println!("  2. 50%");
        println!("  3. 75%");
        println!("  4. 100% (leaves minimal SOL for account rent, if possible)");
        println!("  5. Custom SOL amount");
        print!("Enter your choice (1-5): ");
        io::stdout().flush()?;
        let mut amount_choice_str = String::new();
        io::stdin().read_line(&mut amount_choice_str)?;

        match amount_choice_str.trim() {
            "1" => break current_sol_balance * 0.25,
            "2" => break current_sol_balance * 0.50,
            "3" => break current_sol_balance * 0.75,
            "4" => {
                let rent_buffer = 0.000005; // Rough estimate for lamports for fees, actual rent is complex.
                break if current_sol_balance > rent_buffer { current_sol_balance - rent_buffer } else { current_sol_balance };
            },
            "5" => {
                print!("\nEnter SOL amount to transfer: ");
                io::stdout().flush()?;
                let mut custom_amount_str = String::new();
                io::stdin().read_line(&mut custom_amount_str)?;
                match custom_amount_str.trim().parse::<f64>() {
                    Ok(amt) if amt > 0.0 && amt <= current_sol_balance => break amt,
                    Ok(amt) if amt > current_sol_balance => {
                        println!("Custom amount exceeds balance. Max is {:.9} SOL.", current_sol_balance);
                        // Continue loop
                    }
                    _ => {
                        println!("Invalid custom amount. Please enter a positive number.");
                        // Continue loop
                    }
                }
            }
            _ => {
                println!("Invalid choice. Please enter a number between 1 and 5.");
                // Continue loop
            }
        }
    };

    if amount_to_transfer_sol <= 0.0 { // Should be caught by loops above, but as a safeguard
        println!("Transfer amount must be positive. Withdrawal cancelled.");
        return Ok(());
    }
     if amount_to_transfer_sol > current_sol_balance {
        println!("Transfer amount {:.9} SOL exceeds current balance {:.9} SOL. Withdrawal cancelled.", amount_to_transfer_sol, current_sol_balance);
        return Ok(());
    }

    let amount_usdt = amount_to_transfer_sol * sol_price_usdt;
    println!("\nConfirm Transfer Details:");
    println!("  From:    Bot Instance {} ({})", instance_config.id, pubkey);
    println!("  To:      {}", dest_pubkey);
    println!("  Amount:  {:.9} SOL (~${:.2} USDT)", amount_to_transfer_sol, amount_usdt);

    print!("\nType 'yes' to confirm and proceed with the transfer: ");
    io::stdout().flush()?;
    let mut confirmation_str = String::new();
    io::stdin().read_line(&mut confirmation_str)?;

    if confirmation_str.trim().to_lowercase() == "yes" {
        info!("[Instance {}] Initiating transfer of {:.9} SOL to {}...", instance_config.id, amount_to_transfer_sol, dest_pubkey);

        match transfer_sol_manager(&env.rpc_url, &keypair, &dest_pubkey, sol_to_lamports(amount_to_transfer_sol)).await {
            Ok(signature) => {
                info!("[Instance {}] Transfer successful! Signature: {}", instance_config.id, signature);
                println!("\n✅ Transfer successful!");
                println!("   Signature: {}", signature);
                println!("   Transferred {:.9} SOL from {} to {}.", amount_to_transfer_sol, pubkey, dest_pubkey);
            }
            Err(e) => {
                log::error!("[Instance {}] Transfer failed: {}", instance_config.id, e);
                println!("\n❌ Transfer failed: {}", e);
            }
        }
    } else {
        println!("\nWithdrawal cancelled by user.");
    }

    Ok(())
}