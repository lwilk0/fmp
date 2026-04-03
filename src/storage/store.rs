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
    pub gpg_context: Context,
    pub storage_locations: Locations,
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
        let gpg_context = Context::from_protocol(Protocol::OpenPgp)?;
        let storage_locations = Locations::new(vault_name, account_name);

        Ok(Self {
            gpg_context,
            storage_locations,
        })
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
    pub fn encrypt_account_to_file(&mut self, account_data: &Account) -> Result<(), Error> {
        // Serialize the account to JSON
        let serialized_json = serde_json::to_string(account_data)?;
        let mut account_bytes = serialized_json.into_bytes();

        lock_memory(account_bytes.as_slice());

        let recipient_id = read_to_string(&self.storage_locations.recipient)?
            .trim()
            .to_string();
        let recipient_key = self.gpg_context.get_key(&recipient_id).map_err(|error| {
            anyhow::anyhow!(
                "Failed to find recipient `{}` for encryption. Error: {}",
                recipient_id,
                error
            )
        })?;

        let mut output_file = File::create(&self.storage_locations.data)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            output_file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }

        let mut encrypted_output = Vec::new();
        self.gpg_context
            .encrypt([&recipient_key], &account_bytes, &mut encrypted_output)
            .map_err(|encryption_error| {
                anyhow::anyhow!(
                    "Failed to encrypt data for recipient `{}`. Error: {}",
                    recipient_id,
                    encryption_error
                )
            })?;

        output_file.write_all(&encrypted_output)?;

        account_bytes.zeroize();

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
        let mut encrypted_file_data = Vec::new();
        let input_file = File::open(&self.storage_locations.data)?;

        let mut file_reader = BufReader::new(input_file);
        file_reader.read_to_end(&mut encrypted_file_data)?;

        let mut decrypted_output = Vec::new();
        self.gpg_context
            .decrypt(&encrypted_file_data, &mut decrypted_output)
            .map_err(|decryption_error| {
                anyhow::anyhow!("Failed to decrypt data. Error: {}", decryption_error)
            })?;

        // Lock the decrypted buffer to reduce swap risk
        lock_memory(&decrypted_output);

        // Try to parse as JSON first
        let account_data = if let Ok(json_string) = String::from_utf8(decrypted_output.clone()) {
            if let Ok(parsed_account) = serde_json::from_str::<Account>(&json_string) {
                parsed_account
            } else {
                // Fallback to old format parsing
                Self::parse_legacy_format(&decrypted_output)?
            }
        } else {
            return Err(anyhow::anyhow!(
                "Failed to convert decrypted data to string"
            ));
        };

        decrypted_output.zeroize();

        Ok(account_data)
    }

    /// Parses legacy format (username:password) into an Account struct
    fn parse_legacy_format(legacy_data: &[u8]) -> Result<Account, Error> {
        let separator_position = legacy_data
            .iter()
            .position(|&byte| byte == b':')
            .ok_or_else(|| anyhow::anyhow!("Decrypted data is malformed: missing separator"))?;

        let username_bytes = &legacy_data[..separator_position];
        let password_bytes = &legacy_data[separator_position + 1..];

        let mut parsed_username = String::from_utf8(username_bytes.to_vec())?;
        let mut parsed_password = String::from_utf8(password_bytes.to_vec())?;

        let migrated_account = Account {
            username: parsed_username.clone(),
            password: SecurePassword::new(parsed_password.clone()),
            name: "Migrated Account".to_string(),
            ..Default::default()
        };

        parsed_username.zeroize();
        parsed_password.zeroize();

        Ok(migrated_account)
    }
}

pub fn get_recipient_key(locations: &Locations) -> Result<(Context, gpgme::Key), Error> {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let recipient = std::fs::read_to_string(&locations.recipient)?
        .trim()
        .to_string();
    let recipient_key = ctx.get_key(&recipient).map_err(|e| {
        anyhow::anyhow!(
            "Failed to find recipient `{}` for encryption. Error: {}",
            recipient,
            e
        )
    })?;

    Ok((ctx, recipient_key))
}
