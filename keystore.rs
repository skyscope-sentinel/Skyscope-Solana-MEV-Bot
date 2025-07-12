use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use thiserror::Error;

use crate::security::{Security, SecurityError};

// Constants for keystore settings
const KEYSTORE_DIR: &str = ".skyscope/keystore";
const KEYSTORE_FILE_EXT: &str = "keystore";
const NONCE_SIZE: usize = 24; // Size of XChaCha20Poly1305 nonce

#[derive(Error, Debug)]
pub enum KeystoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Failed to serialize/deserialize keystore data: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Encryption/Decryption error")]
    CryptoError,
    
    #[error("Invalid keypair data")]
    InvalidKeypair,
    
    #[error("Keypair not found: {0}")]
    KeypairNotFound(String),
    
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParseError(String),
    
    #[error("Invalid seed phrase")]
    InvalidSeedPhrase,
    
    #[error("PIN required for this operation")]
    PinRequired,
    
    #[error("Keystore already exists for this name")]
    KeystoreExists,
}

#[derive(Serialize, Deserialize, Debug)]
struct KeystoreData {
    name: String,
    pubkey: String,
    encrypted_keypair: Vec<u8>,
    nonce: Vec<u8>,
    created_at: u64,
    last_used: Option<u64>,
}

pub struct Keystore {
    security: Security,
    keystore_dir: PathBuf,
}

impl Keystore {
    /// Initialize the keystore
    pub fn new(security: Security) -> Result<Self, KeystoreError> {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        let keystore_dir = home_dir.join(KEYSTORE_DIR);
        
        // Create keystore directory if it doesn't exist
        if !keystore_dir.exists() {
            fs::create_dir_all(&keystore_dir)?;
        }
        
        Ok(Self { security, keystore_dir })
    }
    
    /// Generate a new keypair and store it encrypted
    pub fn generate_keypair(&mut self, name: &str, pin: &str) -> Result<Pubkey, KeystoreError> {
        // Verify PIN first
        self.security.verify_pin(pin)?;
        
        // Check if a keystore with this name already exists
        let keystore_path = self.get_keystore_path(name);
        if keystore_path.exists() {
            return Err(KeystoreError::KeystoreExists);
        }
        
        // Generate a new Solana keypair
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey();
        
        // Encrypt and store the keypair
        self.store_keypair(name, &keypair, pin)?;
        
        Ok(pubkey)
    }
    
    /// Import an existing keypair from bytes
    pub fn import_keypair_bytes(&mut self, name: &str, keypair_bytes: &[u8], pin: &str) 
        -> Result<Pubkey, KeystoreError> 
    {
        // Verify PIN first
        self.security.verify_pin(pin)?;
        
        // Check if a keystore with this name already exists
        let keystore_path = self.get_keystore_path(name);
        if keystore_path.exists() {
            return Err(KeystoreError::KeystoreExists);
        }
        
        // Create keypair from bytes
        let keypair = Keypair::from_bytes(keypair_bytes)
            .map_err(|_| KeystoreError::InvalidKeypair)?;
        let pubkey = keypair.pubkey();
        
        // Encrypt and store the keypair
        self.store_keypair(name, &keypair, pin)?;
        
        Ok(pubkey)
    }
    
    /// Import keypair from a Solana JSON keypair file
    pub fn import_from_file(&mut self, name: &str, file_path: &Path, pin: &str) 
        -> Result<Pubkey, KeystoreError> 
    {
        // Read the keypair file
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Parse the keypair
        let keypair_bytes: Vec<u8> = serde_json::from_str(&contents)?;
        self.import_keypair_bytes(name, &keypair_bytes, pin)
    }
    
    /// Import keypair from a BIP39 seed phrase
    pub fn import_from_seed_phrase(&mut self, name: &str, seed_phrase: &str, passphrase: Option<&str>, pin: &str) 
        -> Result<Pubkey, KeystoreError> 
    {
        // Verify PIN first
        self.security.verify_pin(pin)?;
        
        // Check if a keystore with this name already exists
        let keystore_path = self.get_keystore_path(name);
        if keystore_path.exists() {
            return Err(KeystoreError::KeystoreExists);
        }
        
        // Create keypair from seed phrase
        let seed = match passphrase {
            Some(pass) => bip39::Seed::new(
                &bip39::Mnemonic::from_phrase(seed_phrase, bip39::Language::English)
                    .map_err(|_| KeystoreError::InvalidSeedPhrase)?,
                pass,
            ),
            None => bip39::Seed::new(
                &bip39::Mnemonic::from_phrase(seed_phrase, bip39::Language::English)
                    .map_err(|_| KeystoreError::InvalidSeedPhrase)?,
                "",
            ),
        };
        
        // Derive keypair using ed25519-dalek
        let seed_bytes = seed.as_bytes();
        let keypair = keypair_from_seed(&seed_bytes[..32])
            .map_err(|_| KeystoreError::InvalidSeedPhrase)?;
        
        let pubkey = keypair.pubkey();
        
        // Encrypt and store the keypair
        self.store_keypair(name, &keypair, pin)?;
        
        Ok(pubkey)
    }
    
    /// Get a keypair by name (requires PIN for decryption)
    pub fn get_keypair(&mut self, name: &str, pin: &str) -> Result<Keypair, KeystoreError> {
        // Verify PIN first
        self.security.verify_pin(pin)?;
        
        // Get keystore path
        let keystore_path = self.get_keystore_path(name);
        if !keystore_path.exists() {
            return Err(KeystoreError::KeypairNotFound(name.to_string()));
        }
        
        // Read keystore file
        let mut file = File::open(&keystore_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Parse keystore data
        let keystore_data: KeystoreData = serde_json::from_str(&contents)?;
        
        // Derive encryption key from PIN
        let encryption_key = self.derive_encryption_key(pin);
        
        // Create nonce
        let nonce = XNonce::from_slice(&keystore_data.nonce);
        
        // Create cipher
        let cipher = XChaCha20Poly1305::new(&encryption_key.into());
        
        // Decrypt keypair
        let keypair_bytes = cipher
            .decrypt(nonce, keystore_data.encrypted_keypair.as_ref())
            .map_err(|_| KeystoreError::CryptoError)?;
        
        // Create keypair from decrypted bytes
        let keypair = Keypair::from_bytes(&keypair_bytes)
            .map_err(|_| KeystoreError::InvalidKeypair)?;
        
        // Update last used timestamp
        self.update_last_used(name)?;
        
        Ok(keypair)
    }
    
    /// Get public key by name (doesn't require PIN)
    pub fn get_pubkey(&self, name: &str) -> Result<Pubkey, KeystoreError> {
        // Get keystore path
        let keystore_path = self.get_keystore_path(name);
        if !keystore_path.exists() {
            return Err(KeystoreError::KeypairNotFound(name.to_string()));
        }
        
        // Read keystore file
        let mut file = File::open(&keystore_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Parse keystore data
        let keystore_data: KeystoreData = serde_json::from_str(&contents)?;
        
        // Parse pubkey
        Pubkey::from_str(&keystore_data.pubkey)
            .map_err(|e| KeystoreError::PubkeyParseError(e.to_string()))
    }
    
    /// List all available keypairs in the keystore
    pub fn list_keypairs(&self) -> Result<Vec<(String, Pubkey)>, KeystoreError> {
        let mut keypairs = Vec::new();
        
        // Read directory entries
        for entry in fs::read_dir(&self.keystore_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Check if it's a file with the correct extension
            if path.is_file() && path.extension().map_or(false, |ext| ext == KEYSTORE_FILE_EXT) {
                // Read keystore file
                let mut file = File::open(&path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                
                // Parse keystore data
                let keystore_data: KeystoreData = serde_json::from_str(&contents)?;
                
                // Parse pubkey
                let pubkey = Pubkey::from_str(&keystore_data.pubkey)
                    .map_err(|e| KeystoreError::PubkeyParseError(e.to_string()))?;
                
                keypairs.push((keystore_data.name, pubkey));
            }
        }
        
        Ok(keypairs)
    }
    
    /// Delete a keypair from the keystore
    pub fn delete_keypair(&mut self, name: &str, pin: &str) -> Result<(), KeystoreError> {
        // Verify PIN first
        self.security.verify_pin(pin)?;
        
        // Get keystore path
        let keystore_path = self.get_keystore_path(name);
        if !keystore_path.exists() {
            return Err(KeystoreError::KeypairNotFound(name.to_string()));
        }
        
        // Delete the file
        fs::remove_file(keystore_path)?;
        
        Ok(())
    }
    
    /// Sign data with a keypair (requires PIN)
    pub fn sign_data(&mut self, name: &str, data: &[u8], pin: &str) -> Result<Vec<u8>, KeystoreError> {
        // Get the keypair (this will verify the PIN)
        let keypair = self.get_keypair(name, pin)?;
        
        // Sign the data
        Ok(keypair.sign_message(data).as_ref().to_vec())
    }
    
    /// Export keypair as bytes (requires PIN)
    pub fn export_keypair_bytes(&mut self, name: &str, pin: &str) -> Result<Vec<u8>, KeystoreError> {
        // Get the keypair (this will verify the PIN)
        let keypair = self.get_keypair(name, pin)?;
        
        // Return keypair bytes
        Ok(keypair.to_bytes().to_vec())
    }
    
    /// Export keypair to a file (requires PIN)
    pub fn export_to_file(&mut self, name: &str, file_path: &Path, pin: &str) -> Result<(), KeystoreError> {
        // Get keypair bytes (this will verify the PIN)
        let keypair_bytes = self.export_keypair_bytes(name, pin)?;
        
        // Write to file
        let mut file = File::create(file_path)?;
        let json = serde_json::to_string(&keypair_bytes)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
    
    /// Store a keypair encrypted with the PIN
    fn store_keypair(&mut self, name: &str, keypair: &Keypair, pin: &str) -> Result<(), KeystoreError> {
        // Derive encryption key from PIN
        let encryption_key = self.derive_encryption_key(pin);
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);
        
        // Create cipher
        let cipher = XChaCha20Poly1305::new(&encryption_key.into());
        
        // Encrypt keypair
        let keypair_bytes = keypair.to_bytes();
        let encrypted_keypair = cipher
            .encrypt(nonce, keypair_bytes.as_ref())
            .map_err(|_| KeystoreError::CryptoError)?;
        
        // Create keystore data
        let keystore_data = KeystoreData {
            name: name.to_string(),
            pubkey: keypair.pubkey().to_string(),
            encrypted_keypair,
            nonce: nonce_bytes.to_vec(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_used: None,
        };
        
        // Serialize and save to file
        let json = serde_json::to_string_pretty(&keystore_data)?;
        let keystore_path = self.get_keystore_path(name);
        let mut file = File::create(keystore_path)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
    
    /// Update the last_used timestamp for a keypair
    fn update_last_used(&self, name: &str) -> Result<(), KeystoreError> {
        // Get keystore path
        let keystore_path = self.get_keystore_path(name);
        if !keystore_path.exists() {
            return Err(KeystoreError::KeypairNotFound(name.to_string()));
        }
        
        // Read keystore file
        let mut file = File::open(&keystore_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Parse keystore data
        let mut keystore_data: KeystoreData = serde_json::from_str(&contents)?;
        
        // Update last_used timestamp
        keystore_data.last_used = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        
        // Serialize and save to file
        let json = serde_json::to_string_pretty(&keystore_data)?;
        let mut file = File::create(keystore_path)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
    
    /// Derive encryption key from PIN using SHA-256
    fn derive_encryption_key(&self, pin: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(pin.as_bytes());
        hasher.finalize().into()
    }
    
    /// Get the path for a keystore file
    fn get_keystore_path(&self, name: &str) -> PathBuf {
        self.keystore_dir.join(format!("{}.{}", name, KEYSTORE_FILE_EXT))
    }
}

// Helper function to create a Solana keypair from seed bytes
fn keypair_from_seed(seed: &[u8]) -> Result<Keypair, &'static str> {
    if seed.len() < 32 {
        return Err("Seed is too short");
    }
    
    let secret = ed25519_dalek::SecretKey::from_bytes(&seed[..32])
        .map_err(|_| "Invalid seed")?;
    
    let public = ed25519_dalek::PublicKey::from(&secret);
    let keypair = ed25519_dalek::Keypair { secret, public };
    
    let keypair_bytes = keypair.to_bytes();
    Keypair::from_bytes(&keypair_bytes).map_err(|_| "Failed to create keypair")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    // Note: More comprehensive tests would be added here
    // but are omitted for brevity
}
