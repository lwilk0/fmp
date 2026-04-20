//! File system path management for vault operations.

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

use crate::storage::filesystem::validate_path_new;
use anyhow::Error;
use dirs::data_dir;
use std::fs::{File, create_dir_all};
use std::path::PathBuf;

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
    /// * `Locations` - Returns a `Locations` instance.
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
        if validate_path_new(&self.vault) {
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
        } else {
            Err(anyhow::anyhow!("Invalid vault directory path"))
        }
    }

    /// Creates an account directory within the vault.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
    ///
    /// # Errors
    /// * If the account directory cannot be created, an error is returned.
    pub fn create_account_directory(&self) -> Result<(), Error> {
        if validate_path_new(&self.account) {
            create_dir_all(&self.account)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&self.account, std::fs::Permissions::from_mode(0o700))?;
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid account directory path"))
        }
    }

    /// Checks if a vault with the specified name exists.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns `Ok(())` if the vault exists, or an error on failure.
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
