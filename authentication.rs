use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use thiserror::Error;

use crate::keystore::{Keystore, KeystoreError};
use crate::security::{Security, SecurityError};

// Constants for authentication
const SESSION_TIMEOUT_MINUTES: u64 = 15;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Keystore error: {0}")]
    Keystore(#[from] KeystoreError),
    
    #[error("Authentication required")]
    AuthenticationRequired,
    
    #[error("PIN setup required")]
    PinSetupRequired,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// Represents the current authentication state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuthState {
    /// PIN has not been set up yet
    NeedsSetup,
    
    /// User is not authenticated
    Unauthenticated,
    
    /// User is authenticated
    Authenticated,
}

/// Represents an authenticated session
struct Session {
    /// When the session was last active
    last_active: Instant,
    
    /// When the session expires
    expires_at: Instant,
}

impl Session {
    /// Create a new session
    fn new() -> Self {
        let now = Instant::now();
        Self {
            last_active: now,
            expires_at: now + Duration::from_mins(SESSION_TIMEOUT_MINUTES),
        }
    }
    
    /// Check if the session is expired
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
    
    /// Update the session activity time
    fn update_activity(&mut self) {
        self.last_active = Instant::now();
    }
    
    /// Extend the session timeout
    fn extend(&mut self) {
        self.expires_at = Instant::now() + Duration::from_mins(SESSION_TIMEOUT_MINUTES);
    }
}

/// Main authentication manager for the application
pub struct Authentication {
    /// Security module for PIN management
    security: Security,
    
    /// Keystore module for wallet management
    keystore: Option<Keystore>,
    
    /// Current authentication state
    state: AuthState,
    
    /// Current session (if authenticated)
    session: Option<Session>,
}

impl Authentication {
    /// Initialize the authentication system
    pub fn new() -> Result<Self, AuthError> {
        // Initialize security module
        let security = Security::new()?;
        
        // Determine initial state
        let state = if security.is_pin_setup() {
            AuthState::Unauthenticated
        } else {
            AuthState::NeedsSetup
        };
        
        Ok(Self {
            security,
            keystore: None,
            state,
            session: None,
        })
    }
    
    /// Get the current authentication state
    pub fn state(&self) -> AuthState {
        self.state
    }
    
    /// Check if PIN setup is required (first run)
    pub fn needs_setup(&self) -> bool {
        self.state == AuthState::NeedsSetup
    }
    
    /// Check if the user is authenticated
    pub fn is_authenticated(&self) -> bool {
        // Check if we have an active session
        if let Some(session) = &self.session {
            if !session.is_expired() {
                return true;
            }
        }
        
        false
    }
    
    /// Set up a new PIN (only works on first run)
    pub fn setup_pin(&mut self, pin: &str) -> Result<(), AuthError> {
        if self.state != AuthState::NeedsSetup {
            return Err(AuthError::InvalidState(
                "PIN is already set up".to_string(),
            ));
        }
        
        // Set up the PIN
        self.security.setup_pin(pin)?;
        
        // Initialize keystore
        let keystore = Keystore::new(self.security.clone())?;
        self.keystore = Some(keystore);
        
        // Update state and create session
        self.state = AuthState::Authenticated;
        self.session = Some(Session::new());
        
        Ok(())
    }
    
    /// Authenticate with a PIN
    pub fn authenticate(&mut self, pin: &str) -> Result<(), AuthError> {
        if self.state == AuthState::NeedsSetup {
            return Err(AuthError::PinSetupRequired);
        }
        
        // Verify PIN
        self.security.verify_pin(pin)?;
        
        // Initialize keystore if not already done
        if self.keystore.is_none() {
            self.keystore = Some(Keystore::new(self.security.clone())?);
        }
        
        // Update state and create session
        self.state = AuthState::Authenticated;
        self.session = Some(Session::new());
        
        Ok(())
    }
    
    /// Log out the current session
    pub fn logout(&mut self) {
        self.state = AuthState::Unauthenticated;
        self.session = None;
    }
    
    /// Check if the session is valid and update activity
    fn check_session(&mut self) -> Result<(), AuthError> {
        if self.state != AuthState::Authenticated {
            return Err(AuthError::AuthenticationRequired);
        }
        
        // Check if session is expired
        if let Some(session) = &mut self.session {
            if session.is_expired() {
                self.logout();
                return Err(AuthError::SessionExpired);
            }
            
            // Update activity time
            session.update_activity();
            Ok(())
        } else {
            self.logout();
            Err(AuthError::AuthenticationRequired)
        }
    }
    
    /// Extend the current session timeout
    pub fn extend_session(&mut self) -> Result<(), AuthError> {
        self.check_session()?;
        
        if let Some(session) = &mut self.session {
            session.extend();
        }
        
        Ok(())
    }
    
    /// Change the current PIN
    pub fn change_pin(&mut self, current_pin: &str, new_pin: &str) -> Result<(), AuthError> {
        self.check_session()?;
        self.security.change_pin(current_pin, new_pin)?;
        Ok(())
    }
    
    /// Get the keystore (requires authentication)
    pub fn keystore(&mut self) -> Result<&mut Keystore, AuthError> {
        self.check_session()?;
        
        self.keystore.as_mut().ok_or_else(|| {
            AuthError::InvalidState("Keystore not initialized".to_string())
        })
    }
    
    /// Reset failed attempts counter
    pub fn reset_failed_attempts(&mut self) -> Result<(), AuthError> {
        self.security.reset_failed_attempts()?;
        Ok(())
    }
    
    /// Get the current number of failed attempts
    pub fn get_failed_attempts(&self) -> u8 {
        self.security.get_failed_attempts()
    }
}

/// Helper extension for Duration
trait DurationExt {
    fn from_mins(minutes: u64) -> Self;
}

impl DurationExt for Duration {
    fn from_mins(minutes: u64) -> Self {
        Duration::from_secs(minutes * 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: More comprehensive tests would be added here
    // but are omitted for brevity
}
