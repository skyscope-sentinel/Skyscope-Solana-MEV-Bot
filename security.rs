use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Constants for security settings
const MAX_PIN_ATTEMPTS: u8 = 5;
const BASE_DELAY_MS: u64 = 500;
const CONFIG_DIR: &str = ".skyscope";
const SECURITY_CONFIG_FILE: &str = "security_config.json";

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Failed to serialize/deserialize security config: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("PIN must be exactly 4 digits")]
    InvalidPinFormat,
    
    #[error("Invalid PIN")]
    InvalidPin,
    
    #[error("Too many failed attempts, try again later")]
    TooManyAttempts,
    
    #[error("PIN not set up yet")]
    PinNotSetup,
    
    #[error("PIN already set up")]
    PinAlreadySetup,
    
    #[error("Failed to hash PIN: {0}")]
    HashingError(String),
}

#[derive(Serialize, Deserialize, Debug)]
struct SecurityConfig {
    pin_hash: String,
    failed_attempts: u8,
    last_attempt_time: Option<u64>,
    salt: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            pin_hash: String::new(),
            failed_attempts: 0,
            last_attempt_time: None,
            salt: SaltString::generate(&mut OsRng).to_string(),
        }
    }
}

pub struct Security {
    config: SecurityConfig,
    config_path: PathBuf,
}

impl Security {
    /// Initialize the security module
    pub fn new() -> Result<Self, SecurityError> {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        let config_dir = home_dir.join(CONFIG_DIR);
        
        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        let config_path = config_dir.join(SECURITY_CONFIG_FILE);
        let config = if config_path.exists() {
            // Load existing config
            let mut file = File::open(&config_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(&contents)?
        } else {
            // Create new default config
            let config = SecurityConfig::default();
            let json = serde_json::to_string_pretty(&config)?;
            let mut file = File::create(&config_path)?;
            file.write_all(json.as_bytes())?;
            config
        };
        
        Ok(Self { config, config_path })
    }
    
    /// Check if PIN has been set up
    pub fn is_pin_setup(&self) -> bool {
        !self.config.pin_hash.is_empty()
    }
    
    /// Validate that PIN is 4 digits
    fn validate_pin_format(pin: &str) -> Result<(), SecurityError> {
        if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
            return Err(SecurityError::InvalidPinFormat);
        }
        Ok(())
    }
    
    /// Set up a new PIN (only works if PIN hasn't been set up yet)
    pub fn setup_pin(&mut self, pin: &str) -> Result<(), SecurityError> {
        // Check if PIN is already set up
        if self.is_pin_setup() {
            return Err(SecurityError::PinAlreadySetup);
        }
        
        // Validate PIN format
        Self::validate_pin_format(pin)?;
        
        // Hash the PIN with Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(pin.as_bytes(), &salt)
            .map_err(|e| SecurityError::HashingError(e.to_string()))?
            .to_string();
        
        // Update config
        self.config.pin_hash = password_hash;
        self.config.salt = salt.to_string();
        self.config.failed_attempts = 0;
        self.config.last_attempt_time = None;
        
        // Save config
        self.save_config()?;
        
        Ok(())
    }
    
    /// Change an existing PIN
    pub fn change_pin(&mut self, current_pin: &str, new_pin: &str) -> Result<(), SecurityError> {
        // First verify the current PIN
        self.verify_pin(current_pin)?;
        
        // Validate new PIN format
        Self::validate_pin_format(new_pin)?;
        
        // Hash the new PIN with Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(new_pin.as_bytes(), &salt)
            .map_err(|e| SecurityError::HashingError(e.to_string()))?
            .to_string();
        
        // Update config
        self.config.pin_hash = password_hash;
        self.config.salt = salt.to_string();
        self.config.failed_attempts = 0;
        self.config.last_attempt_time = None;
        
        // Save config
        self.save_config()?;
        
        Ok(())
    }
    
    /// Verify a PIN against the stored hash
    pub fn verify_pin(&mut self, pin: &str) -> Result<(), SecurityError> {
        // Check if PIN is set up
        if !self.is_pin_setup() {
            return Err(SecurityError::PinNotSetup);
        }
        
        // Check for too many failed attempts
        if self.config.failed_attempts >= MAX_PIN_ATTEMPTS {
            if let Some(last_attempt) = self.config.last_attempt_time {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                // Calculate backoff time based on number of attempts (exponential backoff)
                let backoff_seconds = BASE_DELAY_MS * (1 << self.config.failed_attempts) / 1000;
                
                if now - last_attempt < backoff_seconds {
                    return Err(SecurityError::TooManyAttempts);
                }
            }
        }
        
        // Parse the stored hash
        let parsed_hash = PasswordHash::new(&self.config.pin_hash)
            .map_err(|e| SecurityError::HashingError(e.to_string()))?;
        
        // Verify the PIN
        let result = Argon2::default().verify_password(pin.as_bytes(), &parsed_hash);
        
        match result {
            Ok(_) => {
                // Reset failed attempts on success
                self.config.failed_attempts = 0;
                self.config.last_attempt_time = None;
                self.save_config()?;
                Ok(())
            }
            Err(_) => {
                // Increment failed attempts
                self.config.failed_attempts += 1;
                self.config.last_attempt_time = Some(
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );
                self.save_config()?;
                
                // Add delay to prevent brute force
                let delay_ms = BASE_DELAY_MS * (1 << self.config.failed_attempts.min(10));
                thread::sleep(Duration::from_millis(delay_ms));
                
                Err(SecurityError::InvalidPin)
            }
        }
    }
    
    /// Reset failed attempts counter
    pub fn reset_failed_attempts(&mut self) -> Result<(), SecurityError> {
        self.config.failed_attempts = 0;
        self.config.last_attempt_time = None;
        self.save_config()?;
        Ok(())
    }
    
    /// Save the security configuration to disk
    fn save_config(&self) -> Result<(), SecurityError> {
        let json = serde_json::to_string_pretty(&self.config)?;
        let mut file = File::create(&self.config_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
    
    /// Get the current number of failed attempts
    pub fn get_failed_attempts(&self) -> u8 {
        self.config.failed_attempts
    }
    
    /// Check if the security config file exists
    pub fn config_exists() -> bool {
        if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join(CONFIG_DIR).join(SECURITY_CONFIG_FILE);
            config_path.exists()
        } else {
            false
        }
    }
    
    /// Delete the security config (for testing or reset purposes)
    pub fn delete_config() -> Result<(), SecurityError> {
        if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join(CONFIG_DIR).join(SECURITY_CONFIG_FILE);
            if config_path.exists() {
                fs::remove_file(config_path)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pin_validation() {
        // Valid PIN
        assert!(Security::validate_pin_format("1234").is_ok());
        
        // Invalid PINs
        assert!(Security::validate_pin_format("123").is_err()); // Too short
        assert!(Security::validate_pin_format("12345").is_err()); // Too long
        assert!(Security::validate_pin_format("123a").is_err()); // Non-digit
    }
    
    // Note: More comprehensive tests would be added here
    // but are omitted for brevity
}
