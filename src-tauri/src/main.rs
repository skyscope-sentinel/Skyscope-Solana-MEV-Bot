//! Skyscope Solana MEV Bot - Tauri Application
//!
//! This is the entry point for the Tauri application that provides a GUI for the
//! Skyscope Solana MEV Bot. It integrates the existing Rust backend with a React frontend.

use log::{debug, error, info, warn, LevelFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{
    api::path::{home_dir, app_dir},
    AppHandle, Manager, State, Window,
};
use tauri_plugin_store::StoreBuilder;
use tokio::sync::RwLock;
use tokio::task;
use tokio::time::sleep;

// Import our existing modules
mod authentication;
mod keystore;
mod security;
mod trading;
mod app;

// Re-export the modules we need
use authentication::{Authentication, AuthError};
use keystore::{Keystore, KeystoreError};
use security::{Security, SecurityError};
use trading::{Trading, TradingError, TradingStats, TradingStrategy};

// Define application state
struct AppState {
    auth: Arc<Mutex<Option<Authentication>>>,
    keystore: Arc<Mutex<Option<Keystore>>>,
    trading: Arc<RwLock<Option<Trading>>>,
    session_start: Arc<Mutex<Option<Instant>>>,
    session_timeout: Arc<Mutex<Duration>>,
    is_trading: Arc<Mutex<bool>>,
    trading_stats: Arc<RwLock<TradingStats>>,
    sol_to_usd_rate: Arc<Mutex<f64>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            auth: Arc::new(Mutex::new(None)),
            keystore: Arc::new(Mutex::new(None)),
            trading: Arc::new(RwLock::new(None)),
            session_start: Arc::new(Mutex::new(None)),
            session_timeout: Arc::new(Mutex::new(Duration::from_secs(15 * 60))), // 15 minutes default
            is_trading: Arc::new(Mutex::new(false)),
            trading_stats: Arc::new(RwLock::new(TradingStats::default())),
            sol_to_usd_rate: Arc::new(Mutex::new(150.75)), // Default SOL to USD rate
        }
    }
}

// Error type for Tauri commands
#[derive(Debug, thiserror::Error)]
enum CommandError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
    
    #[error("Keystore error: {0}")]
    Keystore(#[from] KeystoreError),
    
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Trading error: {0}")]
    Trading(#[from] TradingError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Not authenticated")]
    NotAuthenticated,
    
    #[error("Invalid PIN")]
    InvalidPin,
    
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    
    #[error("Trading already started")]
    TradingAlreadyStarted,
    
    #[error("Trading not started")]
    TradingNotStarted,
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("System error: {0}")]
    System(String),
}

// Implement Serialize for CommandError to send to frontend
impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Type alias for Result with CommandError
type CommandResult<T> = Result<T, CommandError>;

// Wallet information for the frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
struct WalletInfo {
    name: String,
    pubkey: String,
    balance: Option<f64>,
}

// Trading status information for the frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
struct TradingStatusInfo {
    is_trading: bool,
    wallet: Option<String>,
    strategy: Option<String>,
    start_time: Option<String>,
    duration: Option<u64>, // in seconds
}

// Trading statistics for the frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
struct TradingStatsInfo {
    balance: f64,
    profit_loss: f64,
    trades_executed: u64,
    success_rate: f64,
    avg_profit_per_trade: f64,
    last_trade_time: Option<String>,
    last_trade_profit: Option<f64>,
}

// Settings for the frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
struct SettingsInfo {
    trading_amount: f64,
    max_slippage: f64,
    strategy: String,
    session_timeout_minutes: u64,
    auto_start_trading: bool,
}

// Helper function to check if session is valid
fn check_session(state: &AppState) -> CommandResult<()> {
    let session_start = state.session_start.lock().unwrap();
    let session_timeout = state.session_timeout.lock().unwrap();
    
    if let Some(start) = *session_start {
        if start.elapsed() > *session_timeout {
            return Err(CommandError::SessionExpired);
        }
    } else {
        return Err(CommandError::NotAuthenticated);
    }
    
    Ok(())
}

// Helper function to update session start time
fn update_session(state: &AppState) {
    let mut session_start = state.session_start.lock().unwrap();
    *session_start = Some(Instant::now());
}

// Helper function to get app data directory
fn get_app_data_dir() -> CommandResult<PathBuf> {
    let app_data_dir = app_dir(
        tauri::Config::default().tauri.bundle.identifier.clone(),
    )
    .ok_or_else(|| CommandError::System("Failed to get app data directory".to_string()))?;
    
    // Create directory if it doesn't exist
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)?;
    }
    
    Ok(app_data_dir)
}

// Helper function to get keystore directory
fn get_keystore_dir() -> CommandResult<PathBuf> {
    let home = home_dir()
        .ok_or_else(|| CommandError::System("Failed to get home directory".to_string()))?;
    
    let keystore_dir = home.join(".skyscope").join("keystore");
    
    // Create directory if it doesn't exist
    if !keystore_dir.exists() {
        fs::create_dir_all(&keystore_dir)?;
    }
    
    Ok(keystore_dir)
}

// Command to check if PIN is set up
#[tauri::command]
async fn is_pin_setup() -> CommandResult<bool> {
    let auth = Authentication::new().map_err(CommandError::Auth)?;
    Ok(auth.needs_setup())
}

// Command to set up a new PIN
#[tauri::command]
async fn setup_pin(
    pin: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> CommandResult<bool> {
    // Validate PIN
    if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(CommandError::InvalidParameters("PIN must be exactly 4 digits".to_string()));
    }
    
    // Create authentication
    let mut auth = Authentication::new().map_err(CommandError::Auth)?;
    
    // Set up PIN
    auth.setup_pin(&pin).map_err(CommandError::Auth)?;
    
    // Store authentication in state
    {
        let mut auth_state = state.auth.lock().unwrap();
        *auth_state = Some(auth);
    }
    
    // Initialize keystore
    let keystore = {
        let auth_state = state.auth.lock().unwrap();
        auth_state.as_ref().unwrap().keystore().map_err(CommandError::Auth)?
    };
    
    // Store keystore in state
    {
        let mut keystore_state = state.keystore.lock().unwrap();
        *keystore_state = Some(keystore);
    }
    
    // Update session
    update_session(&state);
    
    // Start session timeout monitor
    start_session_monitor(app_handle, state.session_timeout.clone(), state.session_start.clone());
    
    Ok(true)
}

// Command to verify an existing PIN
#[tauri::command]
async fn verify_pin(
    pin: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> CommandResult<bool> {
    // Validate PIN
    if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(CommandError::InvalidParameters("PIN must be exactly 4 digits".to_string()));
    }
    
    // Create authentication
    let mut auth = Authentication::new().map_err(CommandError::Auth)?;
    
    // Authenticate with PIN
    auth.authenticate(&pin).map_err(|e| {
        match e {
            AuthError::InvalidPin => CommandError::InvalidPin,
            _ => CommandError::Auth(e),
        }
    })?;
    
    // Store authentication in state
    {
        let mut auth_state = state.auth.lock().unwrap();
        *auth_state = Some(auth);
    }
    
    // Initialize keystore
    let keystore = {
        let auth_state = state.auth.lock().unwrap();
        auth_state.as_ref().unwrap().keystore().map_err(CommandError::Auth)?
    };
    
    // Store keystore in state
    {
        let mut keystore_state = state.keystore.lock().unwrap();
        *keystore_state = Some(keystore);
    }
    
    // Update session
    update_session(&state);
    
    // Start session timeout monitor
    start_session_monitor(app_handle, state.session_timeout.clone(), state.session_start.clone());
    
    Ok(true)
}

// Command to list all wallets
#[tauri::command]
async fn list_wallets(state: State<'_, AppState>) -> CommandResult<Vec<WalletInfo>> {
    // Check session
    check_session(&state)?;
    
    // Get keystore
    let keystore_state = state.keystore.lock().unwrap();
    let keystore = keystore_state.as_ref().ok_or(CommandError::NotAuthenticated)?;
    
    // List keypairs
    let keypairs = keystore.list_keypairs().map_err(CommandError::Keystore)?;
    
    // Convert to wallet info
    let wallets = keypairs
        .into_iter()
        .map(|(name, pubkey)| WalletInfo {
            name,
            pubkey,
            balance: None, // We'll fetch balances separately
        })
        .collect();
    
    Ok(wallets)
}

// Command to create a new wallet
#[tauri::command]
async fn create_wallet(
    name: String,
    pin: String,
    state: State<'_, AppState>,
) -> CommandResult<WalletInfo> {
    // Check session
    check_session(&state)?;
    
    // Validate PIN
    if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(CommandError::InvalidParameters("PIN must be exactly 4 digits".to_string()));
    }
    
    // Get keystore
    let mut keystore_state = state.keystore.lock().unwrap();
    let keystore = keystore_state.as_mut().ok_or(CommandError::NotAuthenticated)?;
    
    // Generate keypair
    let pubkey = keystore.generate_keypair(&name, &pin).map_err(CommandError::Keystore)?;
    
    // Create wallet info
    let wallet_info = WalletInfo {
        name,
        pubkey,
        balance: None,
    };
    
    Ok(wallet_info)
}

// Command to import wallet from keypair file
#[tauri::command]
async fn import_from_file(
    name: String,
    file_path: String,
    pin: String,
    state: State<'_, AppState>,
) -> CommandResult<WalletInfo> {
    // Check session
    check_session(&state)?;
    
    // Validate PIN
    if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(CommandError::InvalidParameters("PIN must be exactly 4 digits".to_string()));
    }
    
    // Get keystore
    let mut keystore_state = state.keystore.lock().unwrap();
    let keystore = keystore_state.as_mut().ok_or(CommandError::NotAuthenticated)?;
    
    // Import keypair from file
    let pubkey = keystore
        .import_from_file(&name, Path::new(&file_path), &pin)
        .map_err(CommandError::Keystore)?;
    
    // Create wallet info
    let wallet_info = WalletInfo {
        name,
        pubkey,
        balance: None,
    };
    
    Ok(wallet_info)
}

// Command to import wallet from seed phrase
#[tauri::command]
async fn import_from_seed_phrase(
    name: String,
    seed_phrase: String,
    passphrase: String,
    pin: String,
    state: State<'_, AppState>,
) -> CommandResult<WalletInfo> {
    // Check session
    check_session(&state)?;
    
    // Validate PIN
    if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(CommandError::InvalidParameters("PIN must be exactly 4 digits".to_string()));
    }
    
    // Get keystore
    let mut keystore_state = state.keystore.lock().unwrap();
    let keystore = keystore_state.as_mut().ok_or(CommandError::NotAuthenticated)?;
    
    // Import keypair from seed phrase
    let pubkey = keystore
        .import_from_seed_phrase(&name, &seed_phrase, &passphrase, &pin)
        .map_err(CommandError::Keystore)?;
    
    // Create wallet info
    let wallet_info = WalletInfo {
        name,
        pubkey,
        balance: None,
    };
    
    Ok(wallet_info)
}

// Command to get wallet balance
#[tauri::command]
async fn get_wallet_balance(
    wallet_name: String,
    state: State<'_, AppState>,
) -> CommandResult<f64> {
    // Check session
    check_session(&state)?;
    
    // Get keystore
    let keystore_state = state.keystore.lock().unwrap();
    let keystore = keystore_state.as_ref().ok_or(CommandError::NotAuthenticated)?;
    
    // Get public key
    let pubkey = keystore
        .get_pubkey(&wallet_name)
        .map_err(|_| CommandError::WalletNotFound(wallet_name))?;
    
    // In a real implementation, we would fetch the balance from the Solana blockchain
    // For now, we'll return a mock balance
    let balance = 0.05;
    
    Ok(balance)
}

// Command to start trading
#[tauri::command]
async fn start_trading(
    wallet: String,
    strategy: Option<String>,
    amount: Option<f64>,
    max_slippage: Option<f64>,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> CommandResult<bool> {
    // Check session
    check_session(&state)?;
    
    // Check if trading is already started
    {
        let is_trading = state.is_trading.lock().unwrap();
        if *is_trading {
            return Err(CommandError::TradingAlreadyStarted);
        }
    }
    
    // Get keystore
    let keystore_state = state.keystore.lock().unwrap();
    let keystore = keystore_state.as_ref().ok_or(CommandError::NotAuthenticated)?;
    
    // Get public key
    let pubkey = keystore
        .get_pubkey(&wallet)
        .map_err(|_| CommandError::WalletNotFound(wallet.clone()))?;
    
    // Parse strategy
    let strategy = match strategy.as_deref() {
        Some("MEV Arbitrage") | None => TradingStrategy::MevArbitrage,
        Some("Sandwich Trading") => TradingStrategy::SandwichTrading,
        Some("Flashloan Arbitrage") => TradingStrategy::FlashloanArbitrage,
        Some("Liquidity Sniping") => TradingStrategy::LiquiditySniping,
        Some(s) => return Err(CommandError::InvalidParameters(format!("Invalid strategy: {}", s))),
    };
    
    // Create trading instance
    let trading = Trading::new(
        &pubkey,
        strategy,
        amount.unwrap_or(0.1),
        max_slippage.unwrap_or(1.0),
    ).map_err(CommandError::Trading)?;
    
    // Store trading instance in state
    {
        let mut trading_state = state.trading.write().await;
        *trading_state = Some(trading);
    }
    
    // Set trading flag
    {
        let mut is_trading = state.is_trading.lock().unwrap();
        *is_trading = true;
    }
    
    // Start trading in background
    let trading_stats = state.trading_stats.clone();
    let is_trading = state.is_trading.clone();
    let sol_to_usd_rate = state.sol_to_usd_rate.clone();
    
    task::spawn(async move {
        let mut balance = 0.05;
        let mut profit = 0.0;
        let mut trades = 0;
        
        while {
            let is_trading_guard = is_trading.lock().unwrap();
            *is_trading_guard
        } {
            // Simulate trading (in a real implementation, this would execute actual trades)
            let random_profit = (rand::random::<f64>() * 0.002) - 0.0005; // Random profit between -0.0005 and 0.0015 SOL
            profit += random_profit;
            balance += random_profit;
            trades += 1;
            
            // Update trading stats
            {
                let mut stats = trading_stats.write().await;
                stats.balance = balance;
                stats.profit_loss = profit;
                stats.trades_executed = trades;
                stats.success_rate = if trades > 0 {
                    (trades as f64 - (profit.is_sign_negative() as u64) as f64) / trades as f64 * 100.0
                } else {
                    0.0
                };
                stats.avg_profit_per_trade = if trades > 0 {
                    profit / trades as f64
                } else {
                    0.0
                };
                stats.last_trade_time = Some(chrono::Local::now().to_rfc3339());
                stats.last_trade_profit = Some(random_profit);
            }
            
            // Emit trading update event
            let sol_to_usd = {
                let rate = sol_to_usd_rate.lock().unwrap();
                *rate
            };
            
            let _ = app_handle.emit_all(
                "trading_update",
                TradingStatsInfo {
                    balance,
                    profit_loss: profit,
                    trades_executed: trades,
                    success_rate: if trades > 0 {
                        (trades as f64 - (profit.is_sign_negative() as u64) as f64) / trades as f64 * 100.0
                    } else {
                        0.0
                    },
                    avg_profit_per_trade: if trades > 0 {
                        profit / trades as f64
                    } else {
                        0.0
                    },
                    last_trade_time: Some(chrono::Local::now().to_rfc3339()),
                    last_trade_profit: Some(random_profit),
                },
            );
            
            // Sleep for a bit
            sleep(Duration::from_secs(3)).await;
        }
    });
    
    Ok(true)
}

// Command to stop trading
#[tauri::command]
async fn stop_trading(state: State<'_, AppState>) -> CommandResult<bool> {
    // Check session
    check_session(&state)?;
    
    // Check if trading is started
    {
        let is_trading = state.is_trading.lock().unwrap();
        if !*is_trading {
            return Err(CommandError::TradingNotStarted);
        }
    }
    
    // Set trading flag to false
    {
        let mut is_trading = state.is_trading.lock().unwrap();
        *is_trading = false;
    }
    
    Ok(true)
}

// Command to get trading status
#[tauri::command]
async fn get_trading_status(state: State<'_, AppState>) -> CommandResult<TradingStatusInfo> {
    // Check session
    check_session(&state)?;
    
    // Get trading status
    let is_trading = {
        let is_trading = state.is_trading.lock().unwrap();
        *is_trading
    };
    
    // If not trading, return simple status
    if !is_trading {
        return Ok(TradingStatusInfo {
            is_trading: false,
            wallet: None,
            strategy: None,
            start_time: None,
            duration: None,
        });
    }
    
    // Get trading instance
    let trading_state = state.trading.read().await;
    let trading = trading_state.as_ref().ok_or(CommandError::TradingNotStarted)?;
    
    // Get trading info
    let wallet = trading.wallet_pubkey().to_string();
    let strategy = match trading.strategy() {
        TradingStrategy::MevArbitrage => "MEV Arbitrage",
        TradingStrategy::SandwichTrading => "Sandwich Trading",
        TradingStrategy::FlashloanArbitrage => "Flashloan Arbitrage",
        TradingStrategy::LiquiditySniping => "Liquidity Sniping",
    };
    let start_time = trading.start_time().to_rfc3339();
    let duration = trading.duration().as_secs();
    
    Ok(TradingStatusInfo {
        is_trading: true,
        wallet: Some(wallet),
        strategy: Some(strategy.to_string()),
        start_time: Some(start_time),
        duration: Some(duration),
    })
}

// Command to get trading stats
#[tauri::command]
async fn get_trading_stats(state: State<'_, AppState>) -> CommandResult<TradingStatsInfo> {
    // Check session
    check_session(&state)?;
    
    // Get trading stats
    let stats = state.trading_stats.read().await;
    
    Ok(TradingStatsInfo {
        balance: stats.balance,
        profit_loss: stats.profit_loss,
        trades_executed: stats.trades_executed,
        success_rate: stats.success_rate,
        avg_profit_per_trade: stats.avg_profit_per_trade,
        last_trade_time: stats.last_trade_time.clone(),
        last_trade_profit: stats.last_trade_profit,
    })
}

// Command to get settings
#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> CommandResult<SettingsInfo> {
    // Check session
    check_session(&state)?;
    
    // Get session timeout
    let session_timeout = {
        let timeout = state.session_timeout.lock().unwrap();
        timeout.as_secs() / 60
    };
    
    // In a real implementation, we would load settings from a config file
    // For now, we'll return default settings
    Ok(SettingsInfo {
        trading_amount: 0.1,
        max_slippage: 1.0,
        strategy: "MEV Arbitrage".to_string(),
        session_timeout_minutes: session_timeout,
        auto_start_trading: false,
    })
}

// Command to update settings
#[tauri::command]
async fn update_settings(
    settings: SettingsInfo,
    state: State<'_, AppState>,
) -> CommandResult<bool> {
    // Check session
    check_session(&state)?;
    
    // Update session timeout
    {
        let mut timeout = state.session_timeout.lock().unwrap();
        *timeout = Duration::from_secs(settings.session_timeout_minutes * 60);
    }
    
    // In a real implementation, we would save settings to a config file
    
    Ok(true)
}

// Command to logout
#[tauri::command]
async fn logout(state: State<'_, AppState>) -> CommandResult<bool> {
    // Clear authentication
    {
        let mut auth_state = state.auth.lock().unwrap();
        *auth_state = None;
    }
    
    // Clear keystore
    {
        let mut keystore_state = state.keystore.lock().unwrap();
        *keystore_state = None;
    }
    
    // Clear session
    {
        let mut session_start = state.session_start.lock().unwrap();
        *session_start = None;
    }
    
    // Stop trading if active
    {
        let mut is_trading = state.is_trading.lock().unwrap();
        *is_trading = false;
    }
    
    Ok(true)
}

// Command to get SOL to USD rate
#[tauri::command]
async fn get_sol_to_usd_rate(state: State<'_, AppState>) -> CommandResult<f64> {
    let rate = {
        let rate = state.sol_to_usd_rate.lock().unwrap();
        *rate
    };
    
    Ok(rate)
}

// Command to update SOL to USD rate
#[tauri::command]
async fn update_sol_to_usd_rate(
    rate: f64,
    state: State<'_, AppState>,
) -> CommandResult<bool> {
    {
        let mut sol_to_usd = state.sol_to_usd_rate.lock().unwrap();
        *sol_to_usd = rate;
    }
    
    Ok(true)
}

// Function to start session monitor
fn start_session_monitor(
    app_handle: AppHandle,
    session_timeout: Arc<Mutex<Duration>>,
    session_start: Arc<Mutex<Option<Instant>>>,
) {
    task::spawn(async move {
        loop {
            // Check if session is active
            let is_active = {
                let session_start = session_start.lock().unwrap();
                session_start.is_some()
            };
            
            if !is_active {
                // Session is not active, no need to monitor
                break;
            }
            
            // Check if session has expired
            let has_expired = {
                let session_start = session_start.lock().unwrap();
                let session_timeout = session_timeout.lock().unwrap();
                
                if let Some(start) = *session_start {
                    start.elapsed() > *session_timeout
                } else {
                    false
                }
            };
            
            if has_expired {
                // Session has expired, emit event
                let _ = app_handle.emit_all("session_expired", ());
                
                // Clear session
                {
                    let mut session_start = session_start.lock().unwrap();
                    *session_start = None;
                }
                
                break;
            }
            
            // Sleep for a bit
            sleep(Duration::from_secs(10)).await;
        }
    });
}

fn main() {
    // Initialize logging
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .format_timestamp_secs()
        .init();
    
    info!("Starting Skyscope Solana MEV Bot v1.0.0");
    
    // Create application state
    let app_state = AppState::new();
    
    // Build Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Authentication commands
            is_pin_setup,
            setup_pin,
            verify_pin,
            logout,
            
            // Wallet commands
            list_wallets,
            create_wallet,
            import_from_file,
            import_from_seed_phrase,
            get_wallet_balance,
            
            // Trading commands
            start_trading,
            stop_trading,
            get_trading_status,
            get_trading_stats,
            
            // Settings commands
            get_settings,
            update_settings,
            
            // Utility commands
            get_sol_to_usd_rate,
            update_sol_to_usd_rate,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
