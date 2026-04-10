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

use crate::{
    crypto::{lock_memory, secure_overwrite},
    security::SecureClipboardString,
};
use rand::{RngCore, rng};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zeroize::Zeroize;

/// A secure password wrapper that handles memory locking and zeroization
#[derive(Debug)]
pub struct SecurePassword {
    inner_secret: SecretString,
    // Add a dummy field to make memory layout less predictable
    obfuscation_data: [u8; 32],
}

// Fancy schmancy clone that (fingers crossed) does not leak memory
impl Clone for SecurePassword {
    fn clone(&self) -> Self {
        let password_str = self.inner_secret.expose_secret().to_string();
        Self::new(password_str)
    }
}

impl SecurePassword {
    /// Creates a new secure password from a string
    pub fn new(password_input: String) -> Self {
        lock_memory(password_input.as_bytes());

        let mut obfuscation_buffer = [0u8; 32];
        rng().fill_bytes(&mut obfuscation_buffer);

        let secure_password_instance = Self {
            inner_secret: SecretString::new(password_input.into_boxed_str()),
            obfuscation_data: obfuscation_buffer,
        };

        secure_password_instance
    }

    /// Creates an empty secure password
    pub fn empty() -> Self {
        let mut obfuscation_buffer = [0u8; 32];
        rng().fill_bytes(&mut obfuscation_buffer);

        Self {
            inner_secret: SecretString::new(String::new().into_boxed_str()),
            obfuscation_data: obfuscation_buffer,
        }
    }

    /// Returns the length of the password for UI purposes
    pub fn len(&self) -> usize {
        self.inner_secret.expose_secret().len()
    }

    /// Updates the password with a new value
    pub fn update(&mut self, new_password_input: &str) {
        // Regenerate obfuscation data on update
        rng().fill_bytes(&mut self.obfuscation_data);

        self.inner_secret = SecretString::new(new_password_input.to_string().into_boxed_str());
    }

    /// Creates a masked version of the password for display
    pub fn masked(&self, min_length: usize) -> String {
        let len = self.len().max(min_length);
        "•".repeat(len)
    }

    /// Securely copies password to a temporary string for clipboard operations
    /// The returned string should be used immediately and then dropped
    pub fn expose_for_clipboard(&self) -> SecureClipboardString {
        let password_copy = self.inner_secret.expose_secret().to_string();

        SecureClipboardString::new(password_copy)
    }

    /// Creates a temporary obfuscated copy for operations that need the actual password
    /// but want to minimize exposure time
    pub fn with_exposed<F, R>(&self, operation_function: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        let exposed_password = self.inner_secret.expose_secret();
        operation_function(exposed_password)
    }
}

impl Default for SecurePassword {
    fn default() -> Self {
        Self::empty()
    }
}

impl Drop for SecurePassword {
    fn drop(&mut self) {
        secure_overwrite(&mut self.obfuscation_data);
        self.inner_secret.zeroize();
    }
}

// Custom serialization to handle SecretString
impl Serialize for SecurePassword {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the exposed secret (this should only happen during save operations)
        serializer.serialize_str(self.inner_secret.expose_secret())
    }
}

// Custom deserialization to handle SecretString
impl<'de> Deserialize<'de> for SecurePassword {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let password_data = String::deserialize(deserializer)?;
        Ok(SecurePassword::new(password_data))
    }
}
