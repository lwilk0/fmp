//! This module provides functionality for managing a vault.
//! It includes functions for creating a vault, adding accounts, printing entries in the vault and more.

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

use crate::crypto::{lock_memory, secure_overwrite, unlock_memory};
use anyhow::Error;
use dirs::data_dir;
use gpgme::{Context, Protocol};
use secrecy::{ExposeSecret, SecretBox, SecretString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{File, create_dir_all, read_dir, read_to_string, rename},
    io::{BufReader, Read, Write},
    path::PathBuf,
};
use zeroize::Zeroize;

/// A secure string wrapper specifically for clipboard operations
pub struct SecureClipboardString {
    inner: String,
}

impl SecureClipboardString {
    fn new(mut password: String) -> Self {
        lock_memory(password.as_bytes());
        Self { inner: password }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl Drop for SecureClipboardString {
    fn drop(&mut self) {
        // Unlock memory before zeroization
        unlock_memory(self.inner.as_bytes());

        // The ZeroizeOnDrop will handle the actual zeroization
        // but we add extra security measures
        unsafe {
            let bytes = self.inner.as_bytes_mut();
            secure_overwrite(bytes);
        }
    }
}

impl std::ops::Deref for SecureClipboardString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

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
        use std::cmp::Ordering;

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

/// Represents a comprehensive account with all fields supported by the GUI
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub name: String,
    pub account_type: String,
    pub website: String,
    pub username: String,
    pub password: SecurePassword,
    pub notes: String,
    pub additional_fields: HashMap<String, String>,
    pub created_at: String,
    pub modified_at: String,
}

impl Default for Account {
    fn default() -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            name: String::new(),
            account_type: "Password Account".to_string(),
            website: String::new(),
            username: String::new(),
            password: SecurePassword::empty(),
            notes: String::new(),
            additional_fields: HashMap::new(),
            created_at: now.clone(),
            modified_at: now,
        }
    }
}

impl Account {
    pub fn new(name: String) -> Self {
        let mut account = Self::default();
        account.name = name;
        account
    }

    pub fn update_modified_time(&mut self) {
        self.modified_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }
}

/// Represents the locations of various files and directories within a vault.
pub struct Locations {
    pub fmp: PathBuf,
    pub vault: PathBuf,
    pub backup: PathBuf,
    pub account: PathBuf,
    pub recipient: PathBuf,
    pub data: PathBuf,
    pub totp: PathBuf,
    pub gate: PathBuf,
}

impl Locations {
    /// Creates a new `Locations` instance with paths based on the provided vault and account names.
    ///
    /// # Arguments
    /// * `vault_name` - The name of the vault.
    /// * `account_name` - The name of the account.
    ///
    /// # Returns
    /// * `Result<Self, Error>` - Returns a `Locations` instance on success, or an error on failure.
    ///
    /// # Errors
    /// * If the vault or account paths cannot be created, an error is returned.
    pub fn new(vault_name: &str, account_name: &str) -> Locations {
        let base = data_dir().unwrap_or_else(|| PathBuf::from("."));
        let fmp = base.join("fmp");

        let vault = fmp.join("vaults").join(vault_name);
        let backup = fmp.join("backups");
        let account = vault.join(account_name);
        let recipient = vault.join("recipient");
        let data = account.join("data.gpg");
        let totp = vault.join("totp.gpg");
        let gate = vault.join("gate.gpg");

        Self {
            fmp,
            vault,
            backup,
            account,
            recipient,
            data,
            totp,
            gate,
        }
    }

    /// Initializes the vault by creating the necessary directories and files.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
    ///
    /// # Errors
    /// * If the vault directory cannot be created or if the recipient file cannot be created, an error is returned.
    pub fn initialize_vault(&self) -> Result<(), Error> {
        create_dir_all(&self.vault)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.vault, std::fs::Permissions::from_mode(0o700))?;
        }
        let recipient_file = File::create(&self.recipient)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            recipient_file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Creates an account directory within the vault.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
    ///
    /// # Errors
    /// * If the account directory cannot be created, an error is returned.
    pub fn create_account_directory(&self) -> Result<(), Error> {
        create_dir_all(&self.account)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.account, std::fs::Permissions::from_mode(0o700))?;
        }

        Ok(())
    }

    /// Checks if a vault with the specified name exists.
    ///
    /// # Arguments
    /// * `vault_name` - The name of the vault to check.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns a "OK(())" if the vault exists, or an error on failure.
    pub fn does_vault_exist(&self) -> Result<(), Error> {
        if !self.vault.exists() {
            return Err(anyhow::anyhow!(
                "Vault `{:?}` does not exist. Check for typos or create it.",
                self.vault
            ));
        }

        Ok(())
    }

    /// Checks if an account with the specified name exists within the vault.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns `Ok(())` if the account exists, or an error on failure.
    ///
    /// # Errors
    /// * If the account directory does not exist, an error is returned.
    pub fn does_account_exist(&self) -> Result<(), Error> {
        if !self.account.exists() {
            return Err(anyhow::anyhow!(
                "Account `{:?}` does not exist. Check for typos or create it.",
                self.account
            ));
        }

        Ok(())
    }
}

pub struct Store {
    pub ctx: Context,
    pub locations: Locations,
}

impl Store {
    /// Creates a new "Store" instance with a GPGME context and locations based on the provided vault and account names.
    ///
    /// # Arguments
    /// * `vault_name` - The name of the vault.
    /// * `account_name` - The name of the account.
    ///
    /// # Returns
    /// * `Result<Self, Error>` - Returns a "Store" instance on success, or an error on failure.
    ///
    /// # Errors
    /// * If the GPGME context cannot be created or if the locations cannot be initialized, an error is returned.
    pub fn new(vault_name: &str, account_name: &str) -> Result<Self, Error> {
        let ctx = Context::from_protocol(Protocol::OpenPgp)?;

        let locations = Locations::new(vault_name, account_name);

        Ok(Self { ctx, locations })
    }

    /// Encrypts an `Account` struct and writes it to the data.gpg file in the vault.
    ///
    /// # Arguments
    /// * `account` - An `Account` struct containing all account data to be encrypted and stored.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
    ///
    /// # Errors
    /// * If the recipient cannot be found, if the file cannot be created, or if encryption fails.
    pub fn encrypt_account_to_file(&mut self, account: &Account) -> Result<(), Error> {
        // Serialize the account to JSON
        let json_data = serde_json::to_string(account)?;
        let mut data = json_data.into_bytes();

        lock_memory(data.as_slice());

        let recipient = read_to_string(&self.locations.recipient)?
            .trim()
            .to_string();
        let recipient_key = self.ctx.get_key(&recipient).map_err(|e| {
            anyhow::anyhow!(
                "Failed to find recipient `{}` for encryption. Error: {}",
                recipient,
                e
            )
        })?;

        let mut file = File::create(&self.locations.data)?;

        #[cfg(unix)]
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;

        let mut output = Vec::new();
        self.ctx
            .encrypt([&recipient_key], &data, &mut output)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to encrypt data for recipient `{}`. Error: {}",
                    recipient,
                    e
                )
            })?;

        file.write_all(&output)?;

        data.zeroize();

        Ok(())
    }

    /// Decrypts data from data.gpg file in the vault and returns an `Account` struct.
    ///
    /// # Returns
    /// * `Result<Account, Error>` - Returns an `Account` struct containing all decrypted account data on success, or an error on failure.
    ///
    /// # Errors
    /// * If the file cannot be opened, if decryption fails, or if JSON parsing fails.
    pub fn decrypt_account_from_file(&mut self) -> Result<Account, Error> {
        let mut encrypted_data = Vec::new();
        let file = File::open(&self.locations.data)?;

        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut encrypted_data)?;

        let mut output = Vec::new();
        self.ctx
            .decrypt(&encrypted_data, &mut output)
            .map_err(|e| anyhow::anyhow!("Failed to decrypt data. Error: {}", e))?;

        // Lock the decrypted buffer to reduce swap risk
        lock_memory(&output);

        // Try to parse as JSON first (new format)
        let account = if let Ok(json_str) = String::from_utf8(output.clone()) {
            if let Ok(account) = serde_json::from_str::<Account>(&json_str) {
                account
            } else {
                // Fallback to old format parsing
                self.parse_legacy_format(&output)?
            }
        } else {
            return Err(anyhow::anyhow!(
                "Failed to convert decrypted data to string"
            ));
        };

        // Zeroize decrypted buffer
        output.zeroize();

        Ok(account)
    }

    /// Parses legacy format (username:password) into an Account struct
    fn parse_legacy_format(&self, data: &[u8]) -> Result<Account, Error> {
        // Parse "username:password" using the first ':' only
        let sep = data
            .iter()
            .position(|&b| b == b':')
            .ok_or_else(|| anyhow::anyhow!("Decrypted data is malformed: missing separator"))?;

        let username_bytes = &data[..sep];
        let password_bytes = &data[sep + 1..];

        let username = String::from_utf8(username_bytes.to_vec())?;
        let mut password = String::from_utf8(password_bytes.to_vec())?;

        let mut account = Account::default();
        account.username = username;
        account.password = SecurePassword::new(password.clone());
        account.name = "Migrated Account".to_string();

        // Zeroize the temporary password string
        password.zeroize();

        Ok(account)
    }
}

/// Renames a directory from `old_path` to `new_path`.
///
/// # Arguments
/// * `old_path` - The current path of the directory to be renamed.
/// * `new_path` - The new path for the directory.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
/// # Errors
/// * If the directory at `old_path` does not exist, or if the renaming operation fails, an error is returned.
pub fn rename_directory(old_path: &PathBuf, new_path: &PathBuf) -> Result<(), Error> {
    if old_path.exists() {
        rename(old_path, new_path)?;
    } else {
        return Err(anyhow::anyhow!(
            "The directory `{}` does not exist.",
            old_path.display()
        ));
    }

    Ok(())
}

/// Reads all directories in the specified directory and returns their names as a vector of strings.
///
/// # Arguments
/// * `directory` - The path to the directory to read.
///
/// # Returns
/// * `Result<Vec<String>, Error>` - Returns a vector of directory names on success, or an error on failure.
///
/// # Errors
/// * If reading the directory fails, or if the file type cannot be determined, an error is returned.
pub fn read_directory(directory: &PathBuf) -> Result<Vec<String>, Error> {
    let mut directories = Vec::new();

    for entry in read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let account_name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("Failed to convert file name to string."))?;
            directories.push(account_name);
        }
    }

    Ok(directories)
}

/// Retrieves the full account details for a specific account in a vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault containing the account.
/// * `account_name` - The name of the account to retrieve details for.
///
/// # Returns
/// * `Result<Account, Error>` - Returns an `Account` struct containing all decrypted account data on success, or an error on failure.
///
/// # Errors
/// * If the vault does not exist, if the account does not exist, or if decryption fails.
pub fn get_full_account_details(vault_name: &str, account_name: &str) -> Result<Account, Error> {
    let mut store = Store::new(vault_name, account_name)?;
    store.locations.does_vault_exist()?;
    store.locations.does_account_exist()?;

    let account = store.decrypt_account_from_file()?;
    Ok(account)
}

/// Creates a new account in the specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault to create the account in.
/// * `account` - The account data to encrypt and store.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault does not exist, if the account directory cannot be created, or if encryption fails.
pub fn create_account(vault_name: &str, account: &Account) -> Result<(), Error> {
    let mut store = Store::new(vault_name, &account.name)?;
    store.locations.does_vault_exist()?;
    store.locations.create_account_directory()?;
    store.encrypt_account_to_file(account)?;
    Ok(())
}

/// Updates an existing account in the specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault containing the account.
/// * `account` - The updated account data to encrypt and store.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault does not exist, if the account does not exist, or if encryption fails.
pub fn update_account(vault_name: &str, account: &Account) -> Result<(), Error> {
    let mut store = Store::new(vault_name, &account.name)?;
    store.locations.does_vault_exist()?;
    store.locations.does_account_exist()?;
    store.encrypt_account_to_file(account)?;
    Ok(())
}

/// Creates a new vault with the specified name and recipient.
///
/// # Arguments
/// * `vault_name` - The name of the vault to create.
/// * `recipient` - The GPG recipient ID for encryption.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault directory cannot be created or if the recipient file cannot be written.
pub fn create_vault(vault_name: &str, recipient: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, "");
    locations.initialize_vault()?;

    // Write recipient to file
    let mut recipient_file = File::create(&locations.recipient)?;
    recipient_file.write_all(recipient.as_bytes())?;

    // Create a gate file for GPG warm-up
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let recipient_key = ctx.get_key(recipient).map_err(|e| {
        anyhow::anyhow!(
            "Failed to find recipient `{}` for encryption. Error: {}",
            recipient,
            e
        )
    })?;

    let gate_data = b"gate";
    let mut output = Vec::new();
    ctx.encrypt([&recipient_key], &gate_data[..], &mut output)?;

    let mut gate_file = File::create(&locations.gate)?;
    #[cfg(unix)]
    gate_file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    gate_file.write_all(&output)?;

    Ok(())
}

/// Attempt to decrypt the vault's gate file to warm up GPG (triggers passphrase prompt).
pub fn warm_up_gpg(vault_name: &str) -> Result<(), Error> {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let locations = Locations::new(vault_name, "");
    let mut encrypted = Vec::new();
    let mut out = Vec::new();

    let file = File::open(&locations.gate)?;
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut encrypted)?;

    ctx.decrypt(&encrypted, &mut out)
        .map_err(|e| anyhow::anyhow!("Failed to decrypt warm-up file. Error: {}", e))?;
    Ok(())
}
