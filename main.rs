//! Skyscope Solana MEV Bot
//!
//! A secure trading bot for Solana blockchain with PIN protection and secure wallet integration.

use clap::{Parser, Subcommand};
use env_logger::Env;
use log::{error, info, warn};
use std::env;
use std::process;

mod security;
mod keystore;
mod authentication;
mod app;
mod trading;

use crate::app::{App as SkyscopeApp, AppError};
use crate::authentication::{AuthError, Authentication};
use crate::security::Security;

/// Command line arguments for the application
#[derive(Parser)]
#[clap(name = "Skyscope Solana MEV Bot")]
#[clap(author = "Skyscope Sentinel Intelligence")]
#[clap(version = "1.0.0")]
#[clap(about = "A secure trading bot for Solana blockchain with PIN protection")]
struct Cli {
    /// Command to execute
    #[clap(subcommand)]
    command: Option<Commands>,
    
    /// Show funding QR codes for bot instances
    #[clap(long, conflicts_with = "command")]
    show_funding_qr: bool,
    
    /// Launch interactive flow to withdraw funds
    #[clap(long, conflicts_with = "command")]
    withdraw_funds: bool,
}

/// Subcommands for the application
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new wallet
    #[clap(about = "Initialize a new wallet")]
    InitWallet {
        /// Name for the wallet
        #[clap(short, long)]
        name: String,
    },
    
    /// Import an existing wallet
    #[clap(about = "Import an existing wallet")]
    ImportWallet {
        /// Name for the wallet
        #[clap(short, long)]
        name: String,
        
        /// Path to keypair file
        #[clap(short, long)]
        file: String,
    },
    
    /// Start trading with a specific wallet
    #[clap(about = "Start trading with a specific wallet")]
    StartTrading {
        /// Name of the wallet to use
        #[clap(short, long)]
        wallet: String,
    },
}

fn main() {
    // Initialize logging
    init_logging();
    
    info!("Starting Skyscope Solana MEV Bot v1.0.0");
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Handle special command line modes
    if cli.show_funding_qr {
        handle_show_funding_qr();
        return;
    }
    
    if cli.withdraw_funds {
        handle_withdraw_funds();
        return;
    }
    
    // Handle subcommands if present
    if let Some(cmd) = cli.command {
        match handle_command(cmd) {
            Ok(_) => {
                info!("Command executed successfully");
                return;
            }
            Err(e) => {
                error!("Command execution failed: {}", e);
                process::exit(1);
            }
        }
    }
    
    // No special modes or commands, run the main application
    match run_app() {
        Ok(_) => {
            info!("Application exited successfully");
        }
        Err(e) => {
            error!("Application error: {}", e);
            process::exit(1);
        }
    }
}

/// Initialize logging with environment variables
fn init_logging() {
    // Get log level from environment or use default
    let env = Env::default().filter_or("RUST_LOG", "info");
    
    // Initialize logger
    env_logger::Builder::from_env(env)
        .format_timestamp_secs()
        .init();
}

/// Run the main application
fn run_app() -> Result<(), AppError> {
    // Create and run the application
    let mut app = SkyscopeApp::new()?;
    app.run()
}

/// Handle command line subcommands
fn handle_command(cmd: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        Commands::InitWallet { name } => {
            init_wallet(&name)?;
        }
        Commands::ImportWallet { name, file } => {
            import_wallet(&name, &file)?;
        }
        Commands::StartTrading { wallet } => {
            start_trading(&wallet)?;
        }
    }
    
    Ok(())
}

/// Initialize a new wallet
fn init_wallet(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize authentication
    let mut auth = Authentication::new()?;
    
    // Check if PIN is set up
    if auth.needs_setup() {
        println!("First-time setup: You need to create a 4-digit PIN.");
        let pin = prompt_new_pin()?;
        auth.setup_pin(&pin)?;
        println!("PIN set up successfully!");
    } else {
        // Authenticate with PIN
        let pin = prompt_pin()?;
        auth.authenticate(&pin)?;
    }
    
    // Get keystore from authentication
    let mut keystore = auth.keystore()?;
    
    // Generate new keypair
    let pin = prompt_pin()?;
    let pubkey = keystore.generate_keypair(name, &pin)?;
    
    println!("Wallet '{}' created successfully!", name);
    println!("Public Key: {}", pubkey);
    
    Ok(())
}

/// Import an existing wallet
fn import_wallet(name: &str, file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize authentication
    let mut auth = Authentication::new()?;
    
    // Check if PIN is set up
    if auth.needs_setup() {
        println!("First-time setup: You need to create a 4-digit PIN.");
        let pin = prompt_new_pin()?;
        auth.setup_pin(&pin)?;
        println!("PIN set up successfully!");
    } else {
        // Authenticate with PIN
        let pin = prompt_pin()?;
        auth.authenticate(&pin)?;
    }
    
    // Get keystore from authentication
    let mut keystore = auth.keystore()?;
    
    // Import keypair from file
    let pin = prompt_pin()?;
    let pubkey = keystore.import_from_file(name, std::path::Path::new(file), &pin)?;
    
    println!("Wallet '{}' imported successfully!", name);
    println!("Public Key: {}", pubkey);
    
    Ok(())
}

/// Start trading with a specific wallet
fn start_trading(wallet_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize authentication
    let mut auth = Authentication::new()?;
    
    // Check if PIN is set up
    if auth.needs_setup() {
        println!("First-time setup: You need to create a 4-digit PIN.");
        let pin = prompt_new_pin()?;
        auth.setup_pin(&pin)?;
        println!("PIN set up successfully!");
    } else {
        // Authenticate with PIN
        let pin = prompt_pin()?;
        auth.authenticate(&pin)?;
    }
    
    // Get keystore from authentication
    let mut keystore = auth.keystore()?;
    
    // Check if wallet exists
    let pubkey = keystore.get_pubkey(wallet_name)
        .map_err(|_| format!("Wallet '{}' not found", wallet_name))?;
    
    println!("Starting trading with wallet '{}' ({})", wallet_name, pubkey);
    println!("This functionality is not fully implemented in the command line interface.");
    println!("Please use the interactive application for trading.");
    
    Ok(())
}

/// Handle showing funding QR codes
fn handle_show_funding_qr() {
    println!("Showing funding QR codes...");
    
    // Initialize authentication
    match Authentication::new() {
        Ok(mut auth) => {
            // Check if PIN is set up
            if auth.needs_setup() {
                println!("First-time setup: You need to create a 4-digit PIN.");
                let pin = prompt_new_pin().unwrap_or_else(|e| {
                    error!("PIN setup failed: {}", e);
                    process::exit(1);
                });
                
                if let Err(e) = auth.setup_pin(&pin) {
                    error!("PIN setup failed: {}", e);
                    process::exit(1);
                }
                println!("PIN set up successfully!");
            } else {
                // Authenticate with PIN
                let pin = prompt_pin().unwrap_or_else(|e| {
                    error!("PIN prompt failed: {}", e);
                    process::exit(1);
                });
                
                if let Err(e) = auth.authenticate(&pin) {
                    error!("Authentication failed: {}", e);
                    process::exit(1);
                }
            }
            
            // Get keystore from authentication
            match auth.keystore() {
                Ok(keystore) => {
                    // List all wallets
                    match keystore.list_keypairs() {
                        Ok(wallets) => {
                            if wallets.is_empty() {
                                println!("No wallets found. Create or import a wallet first.");
                            } else {
                                println!("Found {} wallet(s):", wallets.len());
                                for (name, pubkey) in wallets {
                                    println!("{} - {}", name, pubkey);
                                    // In a real implementation, we would generate and display QR codes here
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to list wallets: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to access keystore: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            error!("Authentication initialization failed: {}", e);
            process::exit(1);
        }
    }
}

/// Handle withdrawing funds
fn handle_withdraw_funds() {
    println!("Launching withdraw funds flow...");
    
    // Initialize authentication
    match Authentication::new() {
        Ok(mut auth) => {
            // Check if PIN is set up
            if auth.needs_setup() {
                println!("First-time setup: You need to create a 4-digit PIN.");
                let pin = prompt_new_pin().unwrap_or_else(|e| {
                    error!("PIN setup failed: {}", e);
                    process::exit(1);
                });
                
                if let Err(e) = auth.setup_pin(&pin) {
                    error!("PIN setup failed: {}", e);
                    process::exit(1);
                }
                println!("PIN set up successfully!");
            } else {
                // Authenticate with PIN
                let pin = prompt_pin().unwrap_or_else(|e| {
                    error!("PIN prompt failed: {}", e);
                    process::exit(1);
                });
                
                if let Err(e) = auth.authenticate(&pin) {
                    error!("Authentication failed: {}", e);
                    process::exit(1);
                }
            }
            
            println!("Withdraw funds functionality is not fully implemented in the command line interface.");
            println!("Please use the interactive application for withdrawing funds.");
        }
        Err(e) => {
            error!("Authentication initialization failed: {}", e);
            process::exit(1);
        }
    }
}

/// Prompt for PIN
fn prompt_pin() -> Result<String, Box<dyn std::error::Error>> {
    let pin = rpassword::prompt_password("Enter PIN: ")?;
    
    // Validate PIN format
    if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err("PIN must be exactly 4 digits".into());
    }
    
    Ok(pin)
}

/// Prompt for new PIN with confirmation
fn prompt_new_pin() -> Result<String, Box<dyn std::error::Error>> {
    loop {
        let pin = rpassword::prompt_password("Enter a new 4-digit PIN: ")?;
        
        // Validate PIN format
        if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
            println!("PIN must be exactly 4 digits. Please try again.");
            continue;
        }
        
        let confirm_pin = rpassword::prompt_password("Confirm PIN: ")?;
        
        if pin != confirm_pin {
            println!("PINs do not match. Please try again.");
            continue;
        }
        
        return Ok(pin);
    }
}
