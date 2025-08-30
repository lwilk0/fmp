//! GPG encryption and decryption operations for account data.

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

use crate::crypto::lock_memory;
use crate::models::Account;
use crate::security::SecurePassword;
use crate::storage::Locations;
use anyhow::Error;
use gpgme::{Context, Protocol};
use std::fs::{File, read_to_string};
use std::io::{BufReader, Read, Write};
use zeroize::Zeroize;

/// Handles GPG encryption and decryption operations for account data
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
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }

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
