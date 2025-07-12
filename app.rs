use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use console::{style, Term};
use dialoguer::{Input, Password, Select};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use thiserror::Error;

use crate::authentication::{AuthError, AuthState, Authentication};
use crate::keystore::{Keystore, KeystoreError};
use crate::security::{Security, SecurityError};
use crate::trading::{TradingConfig, TradingEngine, TradingError};

// Constants
const APP_NAME: &str = "Skyscope Solana MEV Bot";
const APP_VERSION: &str = "1.0.0";
const DEFAULT_RPC_URL: &str = "https://api.mainnet-beta.solana.com";
const SESSION_REFRESH_INTERVAL: u64 = 5; // minutes

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
    
    #[error("Keystore error: {0}")]
    Keystore(#[from] KeystoreError),
    
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Trading error: {0}")]
    Trading(#[from] TradingError),
    
    #[error("RPC error: {0}")]
    Rpc(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("User cancelled operation")]
    UserCancelled,
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Operation not supported: {0}")]
    NotSupported(String),
}

/// The main application struct
pub struct App {
    /// Authentication manager
    auth: Authentication,
    
    /// RPC client for Solana
    rpc_client: Option<RpcClient>,
    
    /// Trading engine
    trading_engine: Option<Arc<Mutex<TradingEngine>>>,
    
    /// Terminal for user interaction
    term: Term,
    
    /// Last session refresh time
    last_refresh: Instant,
}

impl App {
    /// Create a new App instance
    pub fn new() -> Result<Self, AppError> {
        // Initialize authentication
        let auth = Authentication::new()?;
        
        Ok(Self {
            auth,
            rpc_client: None,
            trading_engine: None,
            term: Term::stdout(),
            last_refresh: Instant::now(),
        })
    }
    
    /// Run the application
    pub fn run(&mut self) -> Result<(), AppError> {
        self.print_banner()?;
        
        // Check if this is first run (PIN setup required)
        if self.auth.needs_setup() {
            self.handle_first_run()?;
        } else {
            self.handle_authentication()?;
        }
        
        // Main application loop
        self.main_loop()
    }
    
    /// Print application banner
    fn print_banner(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.term.write_line(&format!(
            "{} v{}",
            style(APP_NAME).bold().cyan(),
            style(APP_VERSION).bold()
        ))?;
        self.term.write_line("Secure Solana MEV Trading Bot")?;
        self.term.write_line(&format!("{}", style("─".repeat(50)).dim()))?;
        Ok(())
    }
    
    /// Handle first run setup (PIN creation)
    fn handle_first_run(&mut self) -> Result<(), AppError> {
        self.term.write_line("\nWelcome to the first-time setup!")?;
        self.term.write_line("You need to create a 4-digit PIN to secure your wallet and trading operations.")?;
        self.term.write_line("This PIN will be required each time you launch the application.")?;
        
        // Get PIN
        let pin = self.prompt_new_pin()?;
        
        // Set up PIN
        self.auth.setup_pin(&pin)?;
        
        self.term.write_line(&format!(
            "\n{} PIN successfully created!",
            style("✓").green()
        ))?;
        self.term.write_line("You will need this PIN every time you launch the application.")?;
        self.term.write_line("Please make sure to remember it, as there is no recovery option.")?;
        
        // Pause for user to read
        self.prompt_continue()?;
        
        Ok(())
    }
    
    /// Handle authentication for returning users
    fn handle_authentication(&mut self) -> Result<(), AppError> {
        self.term.write_line("\nWelcome back!")?;
        self.term.write_line("Please enter your 4-digit PIN to continue:")?;
        
        // Allow up to 3 attempts
        for attempt in 1..=3 {
            let pin = Password::new()
                .with_prompt("PIN")
                .report(false)
                .interact_on(&self.term)?;
            
            match self.auth.authenticate(&pin) {
                Ok(_) => {
                    self.term.write_line(&format!(
                        "\n{} Authentication successful!",
                        style("✓").green()
                    ))?;
                    return Ok(());
                }
                Err(e) => {
                    if attempt < 3 {
                        self.term.write_line(&format!(
                            "{} Authentication failed: {}. Attempts remaining: {}",
                            style("✗").red(),
                            e,
                            3 - attempt
                        ))?;
                    } else {
                        self.term.write_line(&format!(
                            "{} Too many failed attempts. Exiting...",
                            style("✗").red()
                        ))?;
                        return Err(AppError::Auth(e));
                    }
                }
            }
        }
        
        Err(AppError::UserCancelled)
    }
    
    /// Main application loop
    fn main_loop(&mut self) -> Result<(), AppError> {
        // Initialize RPC client
        self.init_rpc_client()?;
        
        loop {
            // Check and refresh session if needed
            self.check_session()?;
            
            // Display main menu
            self.term.clear_screen()?;
            self.print_banner()?;
            
            let options = vec![
                "Wallet Management",
                "Start Trading Bot",
                "Trading Settings",
                "View Trading History",
                "Security Settings",
                "Exit",
            ];
            
            let selection = Select::new()
                .with_prompt("Select an option")
                .items(&options)
                .default(0)
                .interact_on(&self.term)?;
            
            match selection {
                0 => self.wallet_management_menu()?,
                1 => self.start_trading()?,
                2 => self.trading_settings_menu()?,
                3 => self.view_trading_history()?,
                4 => self.security_settings_menu()?,
                5 => break,
                _ => unreachable!(),
            }
        }
        
        self.term.write_line("Thank you for using Skyscope Solana MEV Bot!")?;
        Ok(())
    }
    
    /// Initialize the RPC client
    fn init_rpc_client(&mut self) -> Result<(), AppError> {
        // Get RPC URL from environment or use default
        let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string());
        
        // Create RPC client
        self.rpc_client = Some(RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        ));
        
        Ok(())
    }
    
    /// Check if session is valid and refresh if needed
    fn check_session(&mut self) -> Result<(), AppError> {
        // Check if session refresh is needed
        let now = Instant::now();
        if now.duration_since(self.last_refresh) > Duration::from_secs(SESSION_REFRESH_INTERVAL * 60) {
            // Extend session
            self.auth.extend_session()?;
            self.last_refresh = now;
        }
        
        Ok(())
    }
    
    /// Wallet management menu
    fn wallet_management_menu(&mut self) -> Result<(), AppError> {
        loop {
            self.term.clear_screen()?;
            self.print_banner()?;
            self.term.write_line("\nWallet Management")?;
            
            let options = vec![
                "Create New Wallet",
                "Import Existing Wallet",
                "List Wallets",
                "View Wallet Details",
                "Export Wallet",
                "Delete Wallet",
                "Back to Main Menu",
            ];
            
            let selection = Select::new()
                .with_prompt("Select an option")
                .items(&options)
                .default(0)
                .interact_on(&self.term)?;
            
            match selection {
                0 => self.create_wallet()?,
                1 => self.import_wallet_menu()?,
                2 => self.list_wallets()?,
                3 => self.view_wallet_details()?,
                4 => self.export_wallet()?,
                5 => self.delete_wallet()?,
                6 => break,
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    /// Create a new wallet
    fn create_wallet(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nCreate New Wallet")?;
        
        // Get wallet name
        let name = Input::<String>::new()
            .with_prompt("Enter a name for this wallet")
            .interact_on(&self.term)?;
        
        // Get PIN for wallet operations
        let pin = Password::new()
            .with_prompt("Enter your PIN to confirm")
            .report(false)
            .interact_on(&self.term)?;
        
        // Create wallet
        let keystore = self.auth.keystore()?;
        let pubkey = keystore.generate_keypair(&name, &pin)?;
        
        self.term.write_line(&format!(
            "\n{} Wallet '{}' created successfully!",
            style("✓").green(),
            name
        ))?;
        self.term.write_line(&format!("Public Key: {}", pubkey))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Import wallet menu
    fn import_wallet_menu(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nImport Wallet")?;
        
        let options = vec![
            "Import from Keypair File",
            "Import from Seed Phrase",
            "Back",
        ];
        
        let selection = Select::new()
            .with_prompt("Select import method")
            .items(&options)
            .default(0)
            .interact_on(&self.term)?;
        
        match selection {
            0 => self.import_from_file()?,
            1 => self.import_from_seed_phrase()?,
            2 => return Ok(()),
            _ => unreachable!(),
        }
        
        Ok(())
    }
    
    /// Import wallet from keypair file
    fn import_from_file(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nImport from Keypair File")?;
        
        // Get wallet name
        let name = Input::<String>::new()
            .with_prompt("Enter a name for this wallet")
            .interact_on(&self.term)?;
        
        // Get file path
        let file_path = Input::<String>::new()
            .with_prompt("Enter path to keypair file")
            .interact_on(&self.term)?;
        
        // Get PIN for wallet operations
        let pin = Password::new()
            .with_prompt("Enter your PIN to confirm")
            .report(false)
            .interact_on(&self.term)?;
        
        // Import wallet
        let keystore = self.auth.keystore()?;
        let pubkey = keystore.import_from_file(&name, Path::new(&file_path), &pin)?;
        
        self.term.write_line(&format!(
            "\n{} Wallet '{}' imported successfully!",
            style("✓").green(),
            name
        ))?;
        self.term.write_line(&format!("Public Key: {}", pubkey))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Import wallet from seed phrase
    fn import_from_seed_phrase(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nImport from Seed Phrase")?;
        
        // Get wallet name
        let name = Input::<String>::new()
            .with_prompt("Enter a name for this wallet")
            .interact_on(&self.term)?;
        
        // Get seed phrase
        let seed_phrase = Password::new()
            .with_prompt("Enter your seed phrase")
            .report(false)
            .interact_on(&self.term)?;
        
        // Get optional passphrase
        let use_passphrase = dialoguer::Confirm::new()
            .with_prompt("Do you use a BIP39 passphrase?")
            .default(false)
            .interact_on(&self.term)?;
        
        let passphrase = if use_passphrase {
            Some(
                Password::new()
                    .with_prompt("Enter your BIP39 passphrase")
                    .report(false)
                    .interact_on(&self.term)?,
            )
        } else {
            None
        };
        
        // Get PIN for wallet operations
        let pin = Password::new()
            .with_prompt("Enter your PIN to confirm")
            .report(false)
            .interact_on(&self.term)?;
        
        // Import wallet
        let keystore = self.auth.keystore()?;
        let pubkey = keystore.import_from_seed_phrase(
            &name,
            &seed_phrase,
            passphrase.as_deref(),
            &pin,
        )?;
        
        self.term.write_line(&format!(
            "\n{} Wallet '{}' imported successfully!",
            style("✓").green(),
            name
        ))?;
        self.term.write_line(&format!("Public Key: {}", pubkey))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// List all wallets
    fn list_wallets(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nWallet List")?;
        
        let keystore = self.auth.keystore()?;
        let wallets = keystore.list_keypairs()?;
        
        if wallets.is_empty() {
            self.term.write_line("No wallets found. Create or import a wallet first.")?;
        } else {
            self.term.write_line(&format!("Found {} wallet(s):", wallets.len()))?;
            for (i, (name, pubkey)) in wallets.iter().enumerate() {
                self.term.write_line(&format!(
                    "{}. {} - {}",
                    i + 1,
                    style(name).bold(),
                    pubkey
                ))?;
            }
        }
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// View wallet details
    fn view_wallet_details(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nWallet Details")?;
        
        // Get wallet list
        let keystore = self.auth.keystore()?;
        let wallets = keystore.list_keypairs()?;
        
        if wallets.is_empty() {
            self.term.write_line("No wallets found. Create or import a wallet first.")?;
            self.prompt_continue()?;
            return Ok(());
        }
        
        // Create wallet selection list
        let wallet_names: Vec<String> = wallets.iter().map(|(name, _)| name.clone()).collect();
        
        let selection = Select::new()
            .with_prompt("Select a wallet")
            .items(&wallet_names)
            .default(0)
            .interact_on(&self.term)?;
        
        let wallet_name = &wallet_names[selection];
        let pubkey = keystore.get_pubkey(wallet_name)?;
        
        // Display wallet details
        self.term.write_line(&format!("\nWallet Name: {}", style(wallet_name).bold()))?;
        self.term.write_line(&format!("Public Key: {}", pubkey))?;
        
        // Get balance if RPC client is available
        if let Some(rpc_client) = &self.rpc_client {
            match rpc_client.get_balance(&pubkey) {
                Ok(balance) => {
                    let sol_balance = balance as f64 / 1_000_000_000.0;
                    self.term.write_line(&format!("Balance: {} SOL", sol_balance))?;
                }
                Err(err) => {
                    self.term.write_line(&format!(
                        "Failed to get balance: {}",
                        style(err.to_string()).red()
                    ))?;
                }
            }
        }
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Export wallet
    fn export_wallet(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nExport Wallet")?;
        
        // Get wallet list
        let keystore = self.auth.keystore()?;
        let wallets = keystore.list_keypairs()?;
        
        if wallets.is_empty() {
            self.term.write_line("No wallets found. Create or import a wallet first.")?;
            self.prompt_continue()?;
            return Ok(());
        }
        
        // Create wallet selection list
        let wallet_names: Vec<String> = wallets.iter().map(|(name, _)| name.clone()).collect();
        
        let selection = Select::new()
            .with_prompt("Select a wallet to export")
            .items(&wallet_names)
            .default(0)
            .interact_on(&self.term)?;
        
        let wallet_name = &wallet_names[selection];
        
        // Get export path
        let export_path = Input::<String>::new()
            .with_prompt("Enter export file path")
            .interact_on(&self.term)?;
        
        // Get PIN for wallet operations
        let pin = Password::new()
            .with_prompt("Enter your PIN to confirm")
            .report(false)
            .interact_on(&self.term)?;
        
        // Export wallet
        keystore.export_to_file(wallet_name, Path::new(&export_path), &pin)?;
        
        self.term.write_line(&format!(
            "\n{} Wallet '{}' exported successfully to '{}'",
            style("✓").green(),
            wallet_name,
            export_path
        ))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Delete wallet
    fn delete_wallet(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nDelete Wallet")?;
        
        // Get wallet list
        let keystore = self.auth.keystore()?;
        let wallets = keystore.list_keypairs()?;
        
        if wallets.is_empty() {
            self.term.write_line("No wallets found. Create or import a wallet first.")?;
            self.prompt_continue()?;
            return Ok(());
        }
        
        // Create wallet selection list
        let wallet_names: Vec<String> = wallets.iter().map(|(name, _)| name.clone()).collect();
        
        let selection = Select::new()
            .with_prompt("Select a wallet to delete")
            .items(&wallet_names)
            .default(0)
            .interact_on(&self.term)?;
        
        let wallet_name = &wallet_names[selection];
        
        // Confirm deletion
        let confirm = dialoguer::Confirm::new()
            .with_prompt(&format!(
                "Are you sure you want to delete wallet '{}'? This action cannot be undone.",
                wallet_name
            ))
            .default(false)
            .interact_on(&self.term)?;
        
        if !confirm {
            self.term.write_line("Operation cancelled.")?;
            self.prompt_continue()?;
            return Ok(());
        }
        
        // Get PIN for wallet operations
        let pin = Password::new()
            .with_prompt("Enter your PIN to confirm")
            .report(false)
            .interact_on(&self.term)?;
        
        // Delete wallet
        keystore.delete_keypair(wallet_name, &pin)?;
        
        self.term.write_line(&format!(
            "\n{} Wallet '{}' deleted successfully",
            style("✓").green(),
            wallet_name
        ))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Start trading
    fn start_trading(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nStart Trading Bot")?;
        
        // Get wallet list
        let keystore = self.auth.keystore()?;
        let wallets = keystore.list_keypairs()?;
        
        if wallets.is_empty() {
            self.term.write_line("No wallets found. Create or import a wallet first.")?;
            self.prompt_continue()?;
            return Ok(());
        }
        
        // Create wallet selection list
        let wallet_names: Vec<String> = wallets.iter().map(|(name, _)| name.clone()).collect();
        
        let selection = Select::new()
            .with_prompt("Select a wallet for trading")
            .items(&wallet_names)
            .default(0)
            .interact_on(&self.term)?;
        
        let wallet_name = &wallet_names[selection];
        
        // Get PIN for wallet operations
        let pin = Password::new()
            .with_prompt("Enter your PIN to confirm")
            .report(false)
            .interact_on(&self.term)?;
        
        // Get keypair
        let keypair = keystore.get_keypair(wallet_name, &pin)?;
        
        // Initialize trading engine if not already done
        if self.trading_engine.is_none() {
            self.init_trading_engine()?;
        }
        
        // Start trading
        let trading_engine = self.trading_engine.as_ref().unwrap().clone();
        
        self.term.write_line(&format!(
            "\n{} Starting trading bot with wallet '{}'...",
            style("✓").green(),
            wallet_name
        ))?;
        
        // Configure trading parameters
        let config = self.configure_trading_parameters()?;
        
        // Start the trading engine
        {
            let mut engine = trading_engine.lock().unwrap();
            engine.start(keypair, config)?;
        }
        
        // Display trading UI
        self.display_trading_ui(trading_engine, wallet_name)?;
        
        Ok(())
    }
    
    /// Initialize the trading engine
    fn init_trading_engine(&mut self) -> Result<(), AppError> {
        // Check if RPC client is initialized
        if self.rpc_client.is_none() {
            self.init_rpc_client()?;
        }
        
        // Create trading engine
        let rpc_client = self.rpc_client.as_ref().unwrap().clone();
        let trading_engine = TradingEngine::new(rpc_client)?;
        
        self.trading_engine = Some(Arc::new(Mutex::new(trading_engine)));
        
        Ok(())
    }
    
    /// Configure trading parameters
    fn configure_trading_parameters(&mut self) -> Result<TradingConfig, AppError> {
        self.term.write_line("\nConfigure Trading Parameters")?;
        
        // Get trading amount
        let trading_amount = Input::<f64>::new()
            .with_prompt("Enter trading amount in SOL (min 0.05)")
            .validate_with(|input: &f64| -> Result<(), &str> {
                if *input < 0.05 {
                    Err("Trading amount must be at least 0.05 SOL")
                } else {
                    Ok(())
                }
            })
            .interact_on(&self.term)?;
        
        // Get max slippage
        let max_slippage = Input::<f64>::new()
            .with_prompt("Enter maximum slippage percentage (0.1-5.0)")
            .default(1.0)
            .validate_with(|input: &f64| -> Result<(), &str> {
                if *input < 0.1 || *input > 5.0 {
                    Err("Slippage must be between 0.1% and 5.0%")
                } else {
                    Ok(())
                }
            })
            .interact_on(&self.term)?;
        
        // Get DEX selection
        let dex_options = vec!["All DEXs", "Raydium", "Orca", "Jupiter", "Meteora"];
        let dex_selection = Select::new()
            .with_prompt("Select DEXs to trade on")
            .items(&dex_options)
            .default(0)
            .interact_on(&self.term)?;
        
        // Get strategy selection
        let strategy_options = vec![
            "MEV Arbitrage",
            "Sandwich Trading",
            "Flashloan Arbitrage",
            "Liquidity Sniping",
        ];
        let strategy_selection = Select::new()
            .with_prompt("Select trading strategy")
            .items(&strategy_options)
            .default(0)
            .interact_on(&self.term)?;
        
        // Create trading config
        let config = TradingConfig {
            amount_sol: trading_amount,
            max_slippage_percent: max_slippage,
            dex_selection: dex_selection,
            strategy: strategy_selection,
            use_flashloans: strategy_selection == 2, // Enable flashloans for flashloan arbitrage
            max_concurrent_trades: 2,
            priority_fee_lamports: 10000,
        };
        
        Ok(config)
    }
    
    /// Display trading UI
    fn display_trading_ui(&mut self, trading_engine: Arc<Mutex<TradingEngine>>, wallet_name: &str) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line(&format!(
            "\nTrading Bot Active - Wallet: {}",
            style(wallet_name).bold()
        ))?;
        
        self.term.write_line("\nPress Ctrl+C to stop trading and return to menu")?;
        self.term.write_line(&format!("{}", style("─".repeat(50)).dim()))?;
        
        // Trading statistics placeholders
        let mut trades_executed = 0;
        let mut profit_sol = 0.0;
        
        // Trading loop
        let start_time = Instant::now();
        let mut last_update = Instant::now();
        
        loop {
            // Check if it's time to update the UI (every second)
            if Instant::now().duration_since(last_update) >= Duration::from_secs(1) {
                // Get trading stats
                let stats = {
                    let engine = trading_engine.lock().unwrap();
                    engine.get_statistics()
                };
                
                trades_executed = stats.trades_executed;
                profit_sol = stats.profit_sol;
                
                // Update display
                self.term.move_cursor_to(0, 8)?;
                self.term.clear_line()?;
                self.term.write_line(&format!(
                    "Running Time: {}s",
                    Instant::now().duration_since(start_time).as_secs()
                ))?;
                
                self.term.clear_line()?;
                self.term.write_line(&format!(
                    "Trades Executed: {}",
                    trades_executed
                ))?;
                
                self.term.clear_line()?;
                self.term.write_line(&format!(
                    "Profit: {} SOL",
                    style(format!("{:.6}", profit_sol)).green()
                ))?;
                
                self.term.clear_line()?;
                self.term.write_line(&format!(
                    "Current Status: {}",
                    style("Scanning for opportunities...").cyan()
                ))?;
                
                // Update last update time
                last_update = Instant::now();
            }
            
            // Check for Ctrl+C
            if self.term.is_key_pressed()? {
                if let Some(key) = self.term.read_key()? {
                    if key == console::Key::Char('c') && self.term.is_ctrl_down() {
                        break;
                    }
                }
            }
            
            // Small sleep to prevent CPU hogging
            std::thread::sleep(Duration::from_millis(100));
        }
        
        // Stop trading
        {
            let mut engine = trading_engine.lock().unwrap();
            engine.stop()?;
        }
        
        self.term.write_line(&format!(
            "\n{} Trading stopped. Summary:",
            style("✓").green()
        ))?;
        self.term.write_line(&format!("Total Trades: {}", trades_executed))?;
        self.term.write_line(&format!("Total Profit: {} SOL", profit_sol))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Trading settings menu
    fn trading_settings_menu(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nTrading Settings")?;
        
        // Initialize trading engine if not already done
        if self.trading_engine.is_none() {
            self.init_trading_engine()?;
        }
        
        let options = vec![
            "Configure RPC Endpoints",
            "Set Default Trading Parameters",
            "Configure Risk Management",
            "Configure DEX Priority",
            "Back to Main Menu",
        ];
        
        let selection = Select::new()
            .with_prompt("Select an option")
            .items(&options)
            .default(0)
            .interact_on(&self.term)?;
        
        match selection {
            0 => self.configure_rpc_endpoints()?,
            1 => self.configure_default_trading_params()?,
            2 => self.configure_risk_management()?,
            3 => self.configure_dex_priority()?,
            4 => return Ok(()),
            _ => unreachable!(),
        }
        
        Ok(())
    }
    
    /// Configure RPC endpoints
    fn configure_rpc_endpoints(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nConfigure RPC Endpoints")?;
        
        // Get main RPC URL
        let main_rpc_url = Input::<String>::new()
            .with_prompt("Enter main RPC URL")
            .default(DEFAULT_RPC_URL.to_string())
            .interact_on(&self.term)?;
        
        // Get fallback RPC URL
        let use_fallback = dialoguer::Confirm::new()
            .with_prompt("Do you want to configure a fallback RPC?")
            .default(false)
            .interact_on(&self.term)?;
        
        let fallback_rpc_url = if use_fallback {
            Some(
                Input::<String>::new()
                    .with_prompt("Enter fallback RPC URL")
                    .interact_on(&self.term)?,
            )
        } else {
            None
        };
        
        // Update RPC client
        self.rpc_client = Some(RpcClient::new_with_commitment(
            main_rpc_url,
            CommitmentConfig::confirmed(),
        ));
        
        // Update trading engine if initialized
        if let Some(trading_engine) = &self.trading_engine {
            let mut engine = trading_engine.lock().unwrap();
            engine.update_rpc_client(self.rpc_client.as_ref().unwrap().clone())?;
            
            if let Some(fallback_url) = fallback_rpc_url {
                engine.set_fallback_rpc(fallback_url)?;
            }
        }
        
        self.term.write_line(&format!(
            "\n{} RPC endpoints updated successfully!",
            style("✓").green()
        ))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Configure default trading parameters
    fn configure_default_trading_params(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nConfigure Default Trading Parameters")?;
        
        // Get default parameters
        let config = self.configure_trading_parameters()?;
        
        // Update trading engine
        if let Some(trading_engine) = &self.trading_engine {
            let mut engine = trading_engine.lock().unwrap();
            engine.set_default_config(config)?;
        }
        
        self.term.write_line(&format!(
            "\n{} Default trading parameters updated successfully!",
            style("✓").green()
        ))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Configure risk management
    fn configure_risk_management(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nConfigure Risk Management")?;
        
        // Get max loss percentage
        let max_loss_percent = Input::<f64>::new()
            .with_prompt("Enter maximum loss percentage before stopping (0.1-50)")
            .default(10.0)
            .validate_with(|input: &f64| -> Result<(), &str> {
                if *input < 0.1 || *input > 50.0 {
                    Err("Value must be between 0.1% and 50%")
                } else {
                    Ok(())
                }
            })
            .interact_on(&self.term)?;
        
        // Get max trade size
        let max_trade_size_sol = Input::<f64>::new()
            .with_prompt("Enter maximum trade size in SOL")
            .default(1.0)
            .validate_with(|input: &f64| -> Result<(), &str> {
                if *input <= 0.0 {
                    Err("Value must be greater than 0")
                } else {
                    Ok(())
                }
            })
            .interact_on(&self.term)?;
        
        // Update trading engine
        if let Some(trading_engine) = &self.trading_engine {
            let mut engine = trading_engine.lock().unwrap();
            engine.set_risk_parameters(max_loss_percent, max_trade_size_sol)?;
        }
        
        self.term.write_line(&format!(
            "\n{} Risk management parameters updated successfully!",
            style("✓").green()
        ))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Configure DEX priority
    fn configure_dex_priority(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nConfigure DEX Priority")?;
        
        // DEX options
        let dex_options = vec!["Raydium", "Orca", "Jupiter", "Meteora"];
        
        // Get priority for each DEX
        let mut priorities = Vec::new();
        
        for dex in &dex_options {
            let priority = Input::<u8>::new()
                .with_prompt(&format!("Enter priority for {} (1-10, 10 is highest)", dex))
                .default(5)
                .validate_with(|input: &u8| -> Result<(), &str> {
                    if *input < 1 || *input > 10 {
                        Err("Priority must be between 1 and 10")
                    } else {
                        Ok(())
                    }
                })
                .interact_on(&self.term)?;
            
            priorities.push((*dex, priority));
        }
        
        // Update trading engine
        if let Some(trading_engine) = &self.trading_engine {
            let mut engine = trading_engine.lock().unwrap();
            for (dex, priority) in priorities {
                engine.set_dex_priority(dex, priority)?;
            }
        }
        
        self.term.write_line(&format!(
            "\n{} DEX priorities updated successfully!",
            style("✓").green()
        ))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// View trading history
    fn view_trading_history(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nTrading History")?;
        
        // Check if trading engine is initialized
        if self.trading_engine.is_none() {
            self.term.write_line("No trading history available yet.")?;
            self.prompt_continue()?;
            return Ok(());
        }
        
        // Get trading history
        let history = {
            let engine = self.trading_engine.as_ref().unwrap().lock().unwrap();
            engine.get_trading_history()
        };
        
        if history.is_empty() {
            self.term.write_line("No trading history available yet.")?;
        } else {
            self.term.write_line(&format!("Found {} trade(s):", history.len()))?;
            
            for (i, trade) in history.iter().enumerate() {
                self.term.write_line(&format!(
                    "{}. {} - {} SOL - {}",
                    i + 1,
                    trade.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    if trade.profit_sol >= 0.0 {
                        style(format!("+{:.6}", trade.profit_sol)).green()
                    } else {
                        style(format!("{:.6}", trade.profit_sol)).red()
                    },
                    trade.description
                ))?;
            }
            
            // Show summary
            let total_profit: f64 = history.iter().map(|t| t.profit_sol).sum();
            self.term.write_line(&format!(
                "\nTotal Profit: {}",
                if total_profit >= 0.0 {
                    style(format!("+{:.6} SOL", total_profit)).green()
                } else {
                    style(format!("{:.6} SOL", total_profit)).red()
                }
            ))?;
        }
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Security settings menu
    fn security_settings_menu(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nSecurity Settings")?;
        
        let options = vec![
            "Change PIN",
            "Configure Session Timeout",
            "Back to Main Menu",
        ];
        
        let selection = Select::new()
            .with_prompt("Select an option")
            .items(&options)
            .default(0)
            .interact_on(&self.term)?;
        
        match selection {
            0 => self.change_pin()?,
            1 => self.configure_session_timeout()?,
            2 => return Ok(()),
            _ => unreachable!(),
        }
        
        Ok(())
    }
    
    /// Change PIN
    fn change_pin(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nChange PIN")?;
        
        // Get current PIN
        let current_pin = Password::new()
            .with_prompt("Enter your current PIN")
            .report(false)
            .interact_on(&self.term)?;
        
        // Get new PIN
        let new_pin = self.prompt_new_pin()?;
        
        // Change PIN
        self.auth.change_pin(&current_pin, &new_pin)?;
        
        self.term.write_line(&format!(
            "\n{} PIN changed successfully!",
            style("✓").green()
        ))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Configure session timeout
    fn configure_session_timeout(&mut self) -> Result<(), AppError> {
        self.term.clear_screen()?;
        self.print_banner()?;
        self.term.write_line("\nConfigure Session Timeout")?;
        
        // Get session timeout
        let timeout_minutes = Input::<u64>::new()
            .with_prompt("Enter session timeout in minutes (5-60)")
            .default(SESSION_REFRESH_INTERVAL)
            .validate_with(|input: &u64| -> Result<(), &str> {
                if *input < 5 || *input > 60 {
                    Err("Timeout must be between 5 and 60 minutes")
                } else {
                    Ok(())
                }
            })
            .interact_on(&self.term)?;
        
        // Update session timeout
        // Note: In a real implementation, we would store this in a config file
        
        self.term.write_line(&format!(
            "\n{} Session timeout updated to {} minutes!",
            style("✓").green(),
            timeout_minutes
        ))?;
        
        self.prompt_continue()?;
        Ok(())
    }
    
    /// Prompt for a new PIN (with validation)
    fn prompt_new_pin(&mut self) -> Result<String, AppError> {
        loop {
            let pin = Password::new()
                .with_prompt("Enter a 4-digit PIN")
                .report(false)
                .interact_on(&self.term)?;
            
            // Validate PIN format
            if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
                self.term.write_line(&format!(
                    "{} PIN must be exactly 4 digits",
                    style("✗").red()
                ))?;
                continue;
            }
            
            // Confirm PIN
            let confirm_pin = Password::new()
                .with_prompt("Confirm PIN")
                .report(false)
                .interact_on(&self.term)?;
            
            if pin != confirm_pin {
                self.term.write_line(&format!(
                    "{} PINs do not match",
                    style("✗").red()
                ))?;
                continue;
            }
            
            return Ok(pin);
        }
    }
    
    /// Prompt user to press any key to continue
    fn prompt_continue(&mut self) -> Result<(), AppError> {
        self.term.write_line("\nPress any key to continue...")?;
        self.term.read_key()?;
        Ok(())
    }
}

/// Dummy TradingEngine and related types for compilation
/// In a real implementation, these would be in separate modules
pub struct TradingEngine {
    // Fields would be here
}

impl TradingEngine {
    pub fn new(rpc_client: RpcClient) -> Result<Self, TradingError> {
        // Implementation would be here
        Ok(Self {})
    }
    
    pub fn start(&mut self, keypair: Keypair, config: TradingConfig) -> Result<(), TradingError> {
        // Implementation would be here
        Ok(())
    }
    
    pub fn stop(&mut self) -> Result<(), TradingError> {
        // Implementation would be here
        Ok(())
    }
    
    pub fn get_statistics(&self) -> TradingStatistics {
        // Implementation would be here
        TradingStatistics {
            trades_executed: 0,
            profit_sol: 0.0,
        }
    }
    
    pub fn update_rpc_client(&mut self, rpc_client: RpcClient) -> Result<(), TradingError> {
        // Implementation would be here
        Ok(())
    }
    
    pub fn set_fallback_rpc(&mut self, fallback_url: String) -> Result<(), TradingError> {
        // Implementation would be here
        Ok(())
    }
    
    pub fn set_default_config(&mut self, config: TradingConfig) -> Result<(), TradingError> {
        // Implementation would be here
        Ok(())
    }
    
    pub fn set_risk_parameters(&mut self, max_loss_percent: f64, max_trade_size_sol: f64) -> Result<(), TradingError> {
        // Implementation would be here
        Ok(())
    }
    
    pub fn set_dex_priority(&mut self, dex: &str, priority: u8) -> Result<(), TradingError> {
        // Implementation would be here
        Ok(())
    }
    
    pub fn get_trading_history(&self) -> Vec<TradeRecord> {
        // Implementation would be here
        Vec::new()
    }
}

#[derive(Debug, Clone)]
pub struct TradingConfig {
    pub amount_sol: f64,
    pub max_slippage_percent: f64,
    pub dex_selection: usize,
    pub strategy: usize,
    pub use_flashloans: bool,
    pub max_concurrent_trades: usize,
    pub priority_fee_lamports: u64,
}

#[derive(Debug, Clone)]
pub struct TradingStatistics {
    pub trades_executed: usize,
    pub profit_sol: f64,
}

#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub profit_sol: f64,
    pub description: String,
}

#[derive(Error, Debug)]
pub enum TradingError {
    #[error("RPC error: {0}")]
    Rpc(String),
    
    #[error("Trading already in progress")]
    AlreadyRunning,
    
    #[error("Trading not started")]
    NotRunning,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
}
