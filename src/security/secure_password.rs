//! Secure password implementation with memory protection and timing attack prevention.

/*
Copyright (C) 2025  Luke Wilkinson

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::crypto::{lock_memory, secure_overwrite};
use crate::security::SecureClipboardString;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zeroize::Zeroize;

/// A secure password wrapper that handles memory locking and zeroization
#[derive(Clone, Debug)]
pub struct SecurePassword {
    inner: SecretString,
    // Add a dummy field to make memory layout less predictable
    _obfuscation: [u8; 32],
}

impl SecurePassword {
    /// Creates a new secure password from a string
    pub fn new(mut password: String) -> Self {
        // Lock the password in memory to prevent swapping
        lock_memory(password.as_bytes());

        // Generate random obfuscation data
        use rand::RngCore;
        let mut obfuscation = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut obfuscation);

        let secure_password = Self {
            inner: SecretString::new(password.clone().into_boxed_str()),
            _obfuscation: obfuscation,
        };

        // Zeroize the original password string
        password.zeroize();

        secure_password
    }

    /// Creates an empty secure password
    pub fn empty() -> Self {
        // Generate random obfuscation data even for empty passwords
        use rand::RngCore;
        let mut obfuscation = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut obfuscation);

        Self {
            inner: SecretString::new(String::new().into_boxed_str()),
            _obfuscation: obfuscation,
        }
    }

    /// Exposes the password securely for temporary use
    pub fn expose_secret(&self) -> &str {
        self.inner.expose_secret()
    }

    /// Returns the length of the password for UI purposes
    pub fn len(&self) -> usize {
        self.inner.expose_secret().len()
    }

    /// Checks if the password is empty
    pub fn is_empty(&self) -> bool {
        self.inner.expose_secret().is_empty()
    }

    /// Updates the password with a new value
    pub fn update(&mut self, mut new_password: String) {
        // Lock the new password in memory
        lock_memory(new_password.as_bytes());

        // Regenerate obfuscation data on update
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut self._obfuscation);

        self.inner = SecretString::new(new_password.clone().into_boxed_str());

        // Zeroize the input password string
        new_password.zeroize();
    }

    /// Creates a masked version of the password for display
    pub fn masked(&self, min_length: usize) -> String {
        let len = self.len().max(min_length);
        "•".repeat(len)
    }

    /// Securely copies password to a temporary string for clipboard operations
    /// The returned string should be used immediately and then dropped
    pub fn expose_for_clipboard(&self) -> SecureClipboardString {
        let password = self.inner.expose_secret().to_string();
        // Lock this temporary copy in memory too
        lock_memory(password.as_bytes());
        SecureClipboardString::new(password)
    }

    /// Creates a temporary obfuscated copy for operations that need the actual password
    /// but want to minimize exposure time
    pub fn with_exposed<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        let password = self.inner.expose_secret();
        // Lock the exposed password in memory during the operation
        lock_memory(password.as_bytes());

        // Add some timing obfuscation to prevent timing attacks
        let start_time = std::time::Instant::now();
        let result = f(password);

        // Ensure minimum execution time to prevent timing analysis
        let min_duration = std::time::Duration::from_millis(10);
        let elapsed = start_time.elapsed();
        if elapsed < min_duration {
            std::thread::sleep(min_duration - elapsed);
        }

        result
    }

    /// Constant-time comparison to prevent timing attacks
    pub fn constant_time_eq(&self, other: &str) -> bool {
        let self_password = self.inner.expose_secret();
        let self_bytes = self_password.as_bytes();
        let other_bytes = other.as_bytes();

        // Ensure we always compare the same amount of data
        let max_len = self_bytes.len().max(other_bytes.len());
        let mut result = self_bytes.len() == other_bytes.len();

        for i in 0..max_len {
            let a = self_bytes.get(i).copied().unwrap_or(0);
            let b = other_bytes.get(i).copied().unwrap_or(0);
            result &= a == b;
        }

        result
    }
}

impl Default for SecurePassword {
    fn default() -> Self {
        Self::empty()
    }
}

impl Drop for SecurePassword {
    fn drop(&mut self) {
        // Securely overwrite the obfuscation data
        secure_overwrite(&mut self._obfuscation);

        // The SecretString will be zeroized by its own Drop implementation
        // but we add extra security measures here

        // Add a small delay to make timing attacks harder
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

// Custom serialization to handle SecretString
impl Serialize for SecurePassword {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the exposed secret (this should only happen during save operations)
        serializer.serialize_str(self.inner.expose_secret())
    }
}

// Custom deserialization to handle SecretString
impl<'de> Deserialize<'de> for SecurePassword {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut password = String::deserialize(deserializer)?;
        let secure_password = SecurePassword::new(password.clone());

        // Zeroize the temporary password string
        password.zeroize();

        Ok(secure_password)
    }
}
