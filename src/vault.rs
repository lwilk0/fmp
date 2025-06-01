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

use crate::crypto::lock_memory;
use anyhow::Error;
use dirs::data_dir;
use gpgme::{Context, Protocol};
use secrecy::{ExposeSecret, SecretBox};
use std::{
    fs::{File, create_dir_all, read_dir, read_to_string, rename},
    io::{BufReader, Read, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};
use zeroize::Zeroize;

/// Represents an accounts data with a username and a securely stored password.
#[derive(Default)]
pub struct UserPass {
    pub username: String,
    pub password: SecretBox<Vec<u8>>,
}

/// Represents the locations of various files and directories within a vault.
pub struct Locations {
    pub fmp_location: PathBuf,
    pub vault_location: PathBuf,
    pub backup_location: PathBuf,
    pub account_location: PathBuf,
    pub recipient_location: PathBuf,
    pub data_location: PathBuf,
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
    pub fn new(vault_name: &str, account_name: &str) -> Result<Self, Error> {
        let fmp_location = data_dir()
            .expect("Failed to find data directory")
            .join("fmp");

        let vault_location = fmp_location.join("vaults").join(vault_name);
        let backup_location = fmp_location.join("backups");
        let account_location = vault_location.join(account_name);
        let recipient_location = vault_location.join("recipient");
        let data_location = account_location.join("data.gpg");

        Ok(Self {
            fmp_location,
            vault_location,
            backup_location,
            account_location,
            recipient_location,
            data_location,
        })
    }

    /// Initializes the vault by creating the necessary directories and files.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
    ///
    /// # Errors
    /// * If the vault directory cannot be created or if the recipient file cannot be created, an error is returned.
    pub fn initialize_vault(&self) -> Result<(), Error> {
        create_dir_all(&self.vault_location)?;
        File::create(&self.recipient_location)?;

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
        create_dir_all(&self.account_location)?;

        Ok(())
    }

    /// Checks if a vault with the specified name exists.
    ///
    /// # Arguments
    /// * `vault_name` - The name of the vault to check.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns a `OK(())` if the vault exists, or an error on failure.
    pub fn does_vault_exist(&self) -> Result<(), Error> {
        if !self.vault_location.exists() {
            return Err(anyhow::anyhow!(
                "Vault `{:?}` does not exist. Check for typos or run `fmp -c` to create it.",
                self.vault_location
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
        if !self.account_location.exists() {
            return Err(anyhow::anyhow!(
                "Account `{:?}` does not exist. Check for typos or run `fmp -a` to create it.",
                self.account_location
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
    /// Creates a new `Store` instance with a GPGME context and locations based on the provided vault and account names.
    ///
    /// # Arguments
    /// * `vault_name` - The name of the vault.
    /// * `account_name` - The name of the account.
    ///
    /// # Returns
    /// * `Result<Self, Error>` - Returns a `Store` instance on success, or an error on failure.
    ///
    /// # Errors
    /// * If the GPGME context cannot be created or if the locations cannot be initialized, an error is returned.
    pub fn new(vault_name: &str, account_name: &str) -> Result<Self, Error> {
        let ctx = Context::from_protocol(Protocol::OpenPgp)?;

        let locations = Locations::new(vault_name, account_name)?;

        Ok(Self { ctx, locations })
    }

    /// Encrypts a `UserPass` struct and writes it to the data.gpg file in the vault.
    ///
    /// # Arguments
    /// * `userpass` - A `UserPass` struct containing the username and password to be encrypted and stored.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
    ///
    /// # Errors
    /// * If the recipient cannot be found, if the file cannot be created, or if encryption fails.
    pub fn encrypt_to_file(&mut self, userpass: &UserPass) -> Result<(), Error> {
        let mut data: Vec<u8> = Vec::new();

        data.extend_from_slice(userpass.username.as_bytes());
        data.push(b':');
        data.extend_from_slice(userpass.password.expose_secret());

        lock_memory(data.as_slice());

        let recipient = read_to_string(&self.locations.recipient_location)?;
        let recipient_key = &self.ctx.get_key(&recipient).map_err(|e| {
            anyhow::anyhow!(
                "Failed to find recipient `{}` for encryption. Error: {}",
                recipient,
                e
            )
        })?;

        let mut file = File::create(&self.locations.data_location)?;

        #[cfg(unix)]
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;

        #[cfg(windows)]
        {
            use windows::Win32::Security::{
                GetSecurityDescriptorDacl, InitializeSecurityDescriptor, SECURITY_DESCRIPTOR,
                SetSecurityDescriptorDacl,
            };
            use windows::Win32::Storage::FileSystem::{
                DACL_SECURITY_INFORMATION, SetFileSecurityW,
            };

            let mut security_descriptor = SECURITY_DESCRIPTOR::default();
            unsafe {
                InitializeSecurityDescriptor(&mut security_descriptor as *mut _ as *mut c_void, 1)?;
                SetSecurityDescriptorDacl(
                    &mut security_descriptor as *mut _ as *mut c_void,
                    true.into(),
                    None,
                    false.into(),
                )?;
                SetFileSecurityW(
                    self.locations.data_location.as_os_str(),
                    DACL_SECURITY_INFORMATION,
                    &security_descriptor as *const _ as *const c_void,
                )?;
            }
        }

        let mut output = Vec::new();
        self.ctx
            .encrypt([recipient_key], &data, &mut output)
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

    /// Decrypts data from the data.gpg file in the vault and returns a `UserPass` struct.
    ///
    /// # Returns
    /// * `Result<UserPass, Error>` - Returns a `UserPass` struct containing the decrypted username and password on success, or an error on failure.
    ///
    /// # Errors
    /// * If the recipient cannot be found, if the file cannot be opened, or if decryption fails.
    pub fn decrypt_from_file(&mut self) -> Result<UserPass, Error> {
        let mut encrypted_data = Vec::new();
        let file = File::open(&self.locations.data_location)?;

        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut encrypted_data)?;

        let mut output = Vec::new();

        lock_memory(&output);

        self.ctx
            .decrypt(&encrypted_data, &mut output)
            .map_err(|e| anyhow::anyhow!("Failed to decrypt data. Error: {}", e))?;

        let mut output_split: Vec<Vec<u8>> =
            output.split(|&b| b == b':').map(|s| s.to_vec()).collect();

        lock_memory(&output_split.concat());

        output.zeroize();

        let username = String::from_utf8(output_split[0].clone())?;

        let output_as_userpass = UserPass {
            username,
            password: SecretBox::new(Box::new(output_split[1].to_vec())),
        };

        lock_memory(output_as_userpass.password.expose_secret());

        for slice in &mut output_split {
            let mut slice_vec = slice.to_vec();
            slice_vec.zeroize();
        }

        output_split.zeroize();

        Ok(output_as_userpass)
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

/// Retrieves the account details for a specific account in a vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault containing the account.
/// * `account_name` - The name of the account to retrieve details for.
///
/// # Returns
/// * `Result<UserPass, Error>` - Returns a `UserPass` struct containing the username and decrypted password on success, or an error on failure.
///
/// # Errors
/// * If the vault does not exist, if the account does not exist, or if decryption fails.
pub fn get_account_details(vault_name: &str, account_name: &str) -> Result<UserPass, Error> {
    let mut store = Store::new(vault_name, account_name)?;

    let userpass = Store::decrypt_from_file(&mut store)?;

    lock_memory(userpass.password.expose_secret());

    Ok(UserPass {
        username: userpass.username,
        password: userpass.password,
    })
}

#[cfg(test)]
#[path = "tests/vault_tests.rs"]
mod vault_tests;
