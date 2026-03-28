#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tokio::sync::RwLock;
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::aead::{Aead, NewAead, generic_array::GenericArray};
use rand::{rngs::OsRng, RngCore};
use argon2::{self, Config, ThreadMode, Variant, Version};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, read_keypair_file},
    signer::Signer,
};
use solana_client::rpc_client::RpcClient;
use std::str::FromStr;
use std::fs;
use std::path::PathBuf;
use std::io::Read;
use bip39::{Mnemonic, Language};
use hex;

// Session management
struct SessionState {
    authenticated: bool,
    last_activity: Instant,
    keypair: Option<Keypair>,
    wallet_name: Option<String>,
    trading_active: bool,
    trading_strategy: Option<String>,
}

impl SessionState {
    fn new() -> Self {
        Self {
            authenticated: false,
            last_activity: Instant::now(),
            keypair: None,
            wallet_name: None,
            trading_active: false,
            trading_strategy: None,
        }
    }

    fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    fn is_session_expired(&self) -> bool {
        // Session expires after 30 minutes of inactivity
        self.last_activity.elapsed() > Duration::from_secs(30 * 60)
    }
}

// Wallet data structure
#[derive(Serialize, Deserialize)]
struct WalletData {
    balance: f64,
    address: String,
    name: String,
}

// PIN verification result
#[derive(Serialize, Deserialize)]
struct PinVerification {
    success: bool,
    message: String,
}

// Global app state
struct AppState {
    session: Arc<Mutex<SessionState>>,
    pin_hash: Arc<RwLock<Option<String>>>,
    pin_salt: Arc<RwLock<Option<Vec<u8>>>>,
    failed_attempts: Arc<Mutex<u32>>,
    last_attempt_time: Arc<Mutex<Instant>>,
    keystore_path: Arc<RwLock<PathBuf>>,
}

// Initialize the app state
fn init_app_state() -> AppState {
    let app_data_dir = tauri::api::path::app_data_dir(
        &tauri::Config::default().tauri.bundle.identifier
    ).expect("Failed to get app data directory");
    
    // Create app directory if it doesn't exist
    fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");
    
    let keystore_path = app_data_dir.join("keystore.dat");
    
    AppState {
        session: Arc::new(Mutex::new(SessionState::new())),
        pin_hash: Arc::new(RwLock::new(None)),
        pin_salt: Arc::new(RwLock::new(None)),
        failed_attempts: Arc::new(Mutex::new(0)),
        last_attempt_time: Arc::new(Mutex::new(Instant::now())),
        keystore_path: Arc::new(RwLock::new(keystore_path)),
    }
}

// Load PIN hash and salt from config file
async fn load_pin_config(app_state: &AppState) -> Result<(), String> {
    let keystore_path = app_state.keystore_path.read().await.clone();
    
    // If the keystore file doesn't exist, no PIN is set yet
    if !keystore_path.exists() {
        return Ok(());
    }
    
    // Read the keystore file
    let keystore_data = fs::read(&keystore_path).map_err(|e| e.to_string())?;
    
    // The first 32 bytes are the salt, the rest is the encrypted data
    if keystore_data.len() < 32 {
        return Err("Invalid keystore file".to_string());
    }
    
    let salt = keystore_data[0..32].to_vec();
    
    // Read the PIN hash from a separate file
    let pin_hash_path = keystore_path.with_extension("pin");
    if pin_hash_path.exists() {
        let pin_hash = fs::read_to_string(&pin_hash_path).map_err(|e| e.to_string())?;
        
        // Update the app state
        *app_state.pin_hash.write().await = Some(pin_hash);
        *app_state.pin_salt.write().await = Some(salt);
    }
    
    Ok(())
}

// Hash PIN using Argon2
fn hash_pin(pin: &str, salt: &[u8]) -> Result<String, String> {
    let config = Config {
        variant: Variant::Argon2id,
        version: Version::Version13,
        mem_cost: 65536,
        time_cost: 10,
        lanes: 4,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    
    argon2::hash_encoded(pin.as_bytes(), salt, &config)
        .map_err(|e| e.to_string())
}

// Verify PIN
fn verify_pin(pin: &str, hash: &str) -> Result<bool, String> {
    argon2::verify_encoded(hash, pin.as_bytes())
        .map_err(|e| e.to_string())
}

// Generate a new salt
fn generate_salt() -> Vec<u8> {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt.to_vec()
}

// Create encryption key from PIN
fn create_encryption_key(pin: &str, salt: &[u8]) -> Vec<u8> {
    let config = Config {
        variant: Variant::Argon2id,
        version: Version::Version13,
        mem_cost: 65536,
        time_cost: 10,
        lanes: 4,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    
    let mut key = vec![0u8; 32];
    argon2::hash_raw(pin.as_bytes(), salt, &config, &mut key)
        .expect("Failed to derive key");
    
    key
}

// Encrypt data with XChaCha20-Poly1305
fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = XChaCha20Poly1305::new(GenericArray::from_slice(key));
    
    // Generate a random 24-byte nonce
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);
    let nonce = GenericArray::from_slice(&nonce);
    
    // Encrypt the data
    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| e.to_string())?;
    
    // Combine nonce and ciphertext
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

// Decrypt data with XChaCha20-Poly1305
fn decrypt_data(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if encrypted_data.len() < 24 {
        return Err("Invalid encrypted data".to_string());
    }
    
    let cipher = XChaCha20Poly1305::new(GenericArray::from_slice(key));
    
    // Split nonce and ciphertext
    let nonce = GenericArray::from_slice(&encrypted_data[0..24]);
    let ciphertext = &encrypted_data[24..];
    
    // Decrypt the data
    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())
}

// Save wallet to keystore
async fn save_wallet_to_keystore(
    keypair: &Keypair,
    pin: &str,
    name: &str,
    app_state: &AppState,
) -> Result<(), String> {
    // Generate a new salt if needed
    let salt = match &*app_state.pin_salt.read().await {
        Some(s) => s.clone(),
        None => {
            let new_salt = generate_salt();
            *app_state.pin_salt.write().await = Some(new_salt.clone());
            new_salt
        }
    };
    
    // Hash the PIN
    let pin_hash = hash_pin(pin, &salt)?;
    *app_state.pin_hash.write().await = Some(pin_hash.clone());
    
    // Create encryption key
    let key = create_encryption_key(pin, &salt);
    
    // Serialize wallet data
    let wallet_bytes = keypair.to_bytes().to_vec();
    let name_bytes = name.as_bytes().to_vec();
    
    // Combine wallet bytes and name bytes (with a separator)
    let mut data = wallet_bytes;
    data.push(0); // Separator
    data.extend_from_slice(&name_bytes);
    
    // Encrypt the data
    let mut encrypted_data = salt.clone();
    encrypted_data.extend_from_slice(&encrypt_data(&data, &key)?);
    
    // Save to file
    let keystore_path = app_state.keystore_path.read().await.clone();
    fs::write(&keystore_path, &encrypted_data)
        .map_err(|e| e.to_string())?;
    
    // Save PIN hash to a separate file
    let pin_hash_path = keystore_path.with_extension("pin");
    fs::write(&pin_hash_path, &pin_hash)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

// Load wallet from keystore
async fn load_wallet_from_keystore(
    pin: &str,
    app_state: &AppState,
) -> Result<(Keypair, String), String> {
    let keystore_path = app_state.keystore_path.read().await.clone();
    
    // Check if keystore file exists
    if !keystore_path.exists() {
        return Err("No wallet found".to_string());
    }
    
    // Read the keystore file
    let keystore_data = fs::read(&keystore_path)
        .map_err(|e| e.to_string())?;
    
    if keystore_data.len() < 32 {
        return Err("Invalid keystore file".to_string());
    }
    
    // Extract salt and encrypted data
    let salt = &keystore_data[0..32];
    let encrypted_data = &keystore_data[32..];
    
    // Create encryption key
    let key = create_encryption_key(pin, salt);
    
    // Decrypt the data
    let decrypted_data = decrypt_data(encrypted_data, &key)?;
    
    // Find the separator
    let separator_pos = decrypted_data.iter()
        .position(|&b| b == 0)
        .ok_or_else(|| "Invalid wallet data format".to_string())?;
    
    // Extract wallet bytes and name
    let wallet_bytes = &decrypted_data[0..separator_pos];
    let name_bytes = &decrypted_data[(separator_pos + 1)..];
    
    // Convert to keypair and name
    let keypair = Keypair::from_bytes(wallet_bytes)
        .map_err(|e| e.to_string())?;
    
    let name = String::from_utf8(name_bytes.to_vec())
        .map_err(|e| e.to_string())?;
    
    Ok((keypair, name))
}

// Get wallet balance from Solana network
async fn get_wallet_balance(pubkey: &Pubkey) -> Result<f64, String> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());
    
    let balance = client.get_balance(pubkey)
        .map_err(|e| e.to_string())?;
    
    // Convert lamports to SOL
    let sol_balance = balance as f64 / 1_000_000_000.0;
    
    Ok(sol_balance)
}

// Check if session is authenticated
#[tauri::command]
async fn check_authentication(app_state: State<'_, AppState>) -> bool {
    let mut session = app_state.session.lock().unwrap();
    
    if session.is_session_expired() {
        session.authenticated = false;
        session.keypair = None;
        return false;
    }
    
    session.update_activity();
    session.authenticated
}

// Authenticate with PIN
#[tauri::command]
async fn authenticate(pin: String, app_state: State<'_, AppState>) -> Result<bool, String> {
    // Check if too many failed attempts
    {
        let mut failed_attempts = app_state.failed_attempts.lock().unwrap();
        let mut last_attempt_time = app_state.last_attempt_time.lock().unwrap();
        
        if *failed_attempts >= 5 {
            // Reset after 10 minutes
            if last_attempt_time.elapsed() > Duration::from_secs(10 * 60) {
                *failed_attempts = 0;
            } else {
                return Err("Too many failed attempts. Please try again later.".to_string());
            }
        }
        
        *last_attempt_time = Instant::now();
    }
    
    // Load PIN configuration if not already loaded
    if app_state.pin_hash.read().await.is_none() {
        load_pin_config(&app_state).await?;
    }
    
    // Get PIN hash and salt
    let pin_hash = match &*app_state.pin_hash.read().await {
        Some(h) => h.clone(),
        None => return Err("No PIN configured".to_string()),
    };
    
    // Verify PIN
    let is_valid = verify_pin(&pin, &pin_hash)?;
    
    if is_valid {
        // Reset failed attempts
        *app_state.failed_attempts.lock().unwrap() = 0;
        
        // Load wallet from keystore
        match load_wallet_from_keystore(&pin, &app_state).await {
            Ok((keypair, name)) => {
                let mut session = app_state.session.lock().unwrap();
                session.authenticated = true;
                session.keypair = Some(keypair);
                session.wallet_name = Some(name);
                session.update_activity();
                
                Ok(true)
            },
            Err(e) => {
                // PIN is correct but wallet loading failed
                Err(format!("Authentication successful but wallet loading failed: {}", e))
            }
        }
    } else {
        // Increment failed attempts
        *app_state.failed_attempts.lock().unwrap() += 1;
        
        Ok(false)
    }
}

// Get wallet data
#[tauri::command]
async fn get_wallet_data(app_state: State<'_, AppState>) -> Result<WalletData, String> {
    let session = app_state.session.lock().unwrap();
    
    if !session.authenticated {
        return Err("Not authenticated".to_string());
    }
    
    let keypair = session.keypair.as_ref()
        .ok_or_else(|| "No wallet loaded".to_string())?;
    
    let name = session.wallet_name.as_ref()
        .cloned()
        .unwrap_or_else(|| "My Wallet".to_string());
    
    let pubkey = keypair.pubkey();
    let balance = get_wallet_balance(&pubkey).await?;
    
    Ok(WalletData {
        balance,
        address: pubkey.to_string(),
        name,
    })
}

// Import wallet from Solflare JSON
#[tauri::command]
async fn import_solflare_wallet(
    file_path: String,
    password: String,
    pin: String,
    name: String,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    // Read the Solflare JSON file
    let mut file = fs::File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Parse the JSON
    let json: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    // Extract the encrypted private key
    let encrypted_key = json["encrypted_key"].as_str()
        .ok_or_else(|| "Invalid Solflare wallet format".to_string())?;
    
    // TODO: Implement proper Solflare decryption
    // This is a simplified version - actual implementation would need to match Solflare's encryption
    
    // For now, we'll create a dummy keypair from a seed phrase
    let mnemonic = Mnemonic::new(bip39::MnemonicType::Words12, Language::English);
    let seed = mnemonic.to_seed("");
    let keypair = Keypair::from_bytes(&seed[0..32])
        .map_err(|e| format!("Failed to create keypair: {}", e))?;
    
    // Save the wallet to keystore
    save_wallet_to_keystore(&keypair, &pin, &name, &app_state).await?;
    
    // Update session
    let mut session = app_state.session.lock().unwrap();
    session.authenticated = true;
    session.keypair = Some(keypair);
    session.wallet_name = Some(name);
    session.update_activity();
    
    Ok(true)
}

// Import wallet from seed phrase
#[tauri::command]
async fn import_seed_phrase(
    seed_phrase: String,
    pin: String,
    name: String,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    // Parse and validate the seed phrase
    let mnemonic = Mnemonic::from_phrase(&seed_phrase, Language::English)
        .map_err(|e| format!("Invalid seed phrase: {}", e))?;
    
    // Generate seed and keypair
    let seed = mnemonic.to_seed("");
    let keypair = Keypair::from_bytes(&seed[0..32])
        .map_err(|e| format!("Failed to create keypair: {}", e))?;
    
    // Save the wallet to keystore
    save_wallet_to_keystore(&keypair, &pin, &name, &app_state).await?;
    
    // Update session
    let mut session = app_state.session.lock().unwrap();
    session.authenticated = true;
    session.keypair = Some(keypair);
    session.wallet_name = Some(name);
    session.update_activity();
    
    Ok(true)
}

// Import wallet from private key
#[tauri::command]
async fn import_private_key(
    private_key: String,
    pin: String,
    name: String,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    // Parse the private key
    let private_key = private_key.trim();
    
    // Convert hex string to bytes
    let key_bytes = hex::decode(private_key)
        .map_err(|e| format!("Invalid private key: {}", e))?;
    
    if key_bytes.len() != 32 {
        return Err("Invalid private key length".to_string());
    }
    
    // Create keypair
    let keypair = Keypair::from_bytes(&key_bytes)
        .map_err(|e| format!("Failed to create keypair: {}", e))?;
    
    // Save the wallet to keystore
    save_wallet_to_keystore(&keypair, &pin, &name, &app_state).await?;
    
    // Update session
    let mut session = app_state.session.lock().unwrap();
    session.authenticated = true;
    session.keypair = Some(keypair);
    session.wallet_name = Some(name);
    session.update_activity();
    
    Ok(true)
}

// Start trading
#[tauri::command]
async fn start_trading(
    strategy: String,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    let mut session = app_state.session.lock().unwrap();
    
    if !session.authenticated {
        return Err("Not authenticated".to_string());
    }
    
    if session.keypair.is_none() {
        return Err("No wallet loaded".to_string());
    }
    
    // Validate strategy
    let valid_strategies = ["arbitrage", "sandwich", "flashloan", "liquidity"];
    if !valid_strategies.contains(&strategy.as_str()) {
        return Err("Invalid strategy".to_string());
    }
    
    // Start trading
    session.trading_active = true;
    session.trading_strategy = Some(strategy);
    session.update_activity();
    
    // In a real implementation, we would start a background task for trading
    // For now, we just update the session state
    
    Ok(true)
}

// Stop trading
#[tauri::command]
async fn stop_trading(app_state: State<'_, AppState>) -> Result<bool, String> {
    let mut session = app_state.session.lock().unwrap();
    
    if !session.authenticated {
        return Err("Not authenticated".to_string());
    }
    
    // Stop trading
    session.trading_active = false;
    session.update_activity();
    
    // In a real implementation, we would stop the background trading task
    // For now, we just update the session state
    
    Ok(true)
}

fn main() {
    tauri::Builder::default()
        .manage(init_app_state())
        .invoke_handler(tauri::generate_handler![
            check_authentication,
            authenticate,
            get_wallet_data,
            import_solflare_wallet,
            import_seed_phrase,
            import_private_key,
            start_trading,
            stop_trading,
        ])
        .setup(|app| {
            // Load PIN configuration on startup
            let app_state = app.state::<AppState>();
            
            tauri::async_runtime::block_on(async {
                if let Err(e) = load_pin_config(&app_state).await {
                    eprintln!("Failed to load PIN configuration: {}", e);
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
