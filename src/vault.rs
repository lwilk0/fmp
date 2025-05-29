//! This file provides functionality for managing a vault.
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

use crate::crypto::{decrypt_variable, encrypt_variable};
use anyhow::Error;
use dirs::data_dir;
use gpgme::{Context, Protocol};
use libc::c_void;
use log::warn;
use prettytable::{Table, row};
use secrecy::{ExposeSecret, SecretBox};
use std::{
    fs::{File, create_dir_all, read_dir, read_to_string, rename},
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};
use zeroize::Zeroize;

const NULL: &str = "null";

/// Represents an accounts data with a username and a securely stored password.
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
        if vault_name.is_empty() || account_name.is_empty() {
            return Err(anyhow::anyhow!(
                "Vault name and account name cannot be empty."
            ));
        }

        let fmp_location = data_dir()
            .expect("Failed to find data directory")
            .join("fmp");

        let vault_location = fmp_location.join(vault_name);
        let backup_location = fmp_location.join("backup");
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
            Err(anyhow::anyhow!(
                "Account `{:?}` does not exist. Check for typos or run `fmp -a` to create it.",
                self.account_location
            ))
        } else {
            Ok(())
        }
    }

    /// Finds all account names within a vault.
    ///
    /// # Returns
    /// * `Result<Vec<String>, Error>` - Returns a vector of account names on success, or an error on failure.
    ///
    /// # Errors
    /// * If reading the directory fails or if the file names cannot be converted to strings, an error is returned.
    pub fn find_account_names(&self) -> Result<Vec<String>, Error> {
        let mut account_names = Vec::new();

        for entry in read_dir(&self.vault_location)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let account_name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| anyhow::anyhow!("Failed to convert file name to string."))?;
                account_names.push(account_name);
            }
        }

        Ok(account_names)
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
    pub fn encrypt_to_file(&mut self, userpass: UserPass) -> Result<(), Error> {
        let mut data: Vec<u8> = Vec::new();

        data.extend_from_slice(userpass.username.as_bytes());
        data.push(b':');

        // Access the password securely
        let mut decrypted_password =
            decrypt_variable(&mut self.ctx, userpass.password.expose_secret().as_slice())?;
        data.extend_from_slice(&decrypted_password);

        decrypted_password.zeroize();

        #[cfg(unix)]
        unsafe {
            libc::mlock(data.as_ptr() as *const c_void, data.len())
        };

        #[cfg(windows)]
        unsafe {
            use windows::Win32::System::Memory::VirtualLock;
            VirtualLock(data.as_ptr() as *const c_void, data.len());
        }

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

            // Set file permissions to restrict access on Windows
            let mut security_descriptor = SECURITY_DESCRIPTOR::default();
            unsafe {
                InitializeSecurityDescriptor(
                    &mut security_descriptor as *mut _ as *mut c_void,
                    1, // SECURITY_DESCRIPTOR_REVISION
                )?;
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
        let recipient = read_to_string(&self.locations.recipient_location)?;

        let mut file = File::open(&self.locations.data_location)?;
        let mut encrypted_data = Vec::new();

        file.read_to_end(&mut encrypted_data)?;

        let mut output = Vec::new();
        self.ctx
            .decrypt(&encrypted_data, &mut output)
            .map_err(|e| anyhow::anyhow!("Failed to decrypt data. Error: {}", e))?;

        let mut output_split: Vec<&[u8]> = output.split(|&b| b == b':').collect();

        let username = String::from_utf8(output_split[0].to_vec())?;

        let mut password_bytes = output_split[1].to_vec(); // NOTE: This is done to avoid converting the password to a string, which could expose sensitive data in memory
        let encrypted_password = encrypt_variable(&mut self.ctx, &mut password_bytes, &recipient)?;

        password_bytes.zeroize();

        let output_as_userpass = UserPass {
            username,
            password: SecretBox::new(Box::new(encrypted_password)),
        };

        #[cfg(unix)]
        unsafe {
            libc::mlock(
                output_as_userpass.password.expose_secret().as_ptr() as *const c_void,
                output_as_userpass.password.expose_secret().len(),
            )
        };

        #[cfg(windows)]
        unsafe {
            use windows::Win32::System::Memory::VirtualLock;
            VirtualLock(
                output_as_userpass.password.expose_secret().as_ptr() as *const c_void,
                output_as_userpass.password.expose_secret().len(),
            );
        }

        for slice in &mut output_split {
            let mut slice_vec = slice.to_vec();
            slice_vec.zeroize();
        }

        Ok(output_as_userpass)
    }

    /// Changes the username of the account in the vault by updating the username in the encrypted data file.
    ///
    /// # Arguments
    /// * `new_username` - The new username to set for the account.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
    ///
    /// # Errors
    /// * If the vault does not exist, if the account data cannot be decrypted, or if the file cannot be written to.
    pub fn change_account_username(&mut self, new_username: &str) -> Result<(), Error> {
        let mut userpass = self.decrypt_from_file()?;
        userpass.username = new_username.to_string();

        self.encrypt_to_file(userpass)?;

        Ok(())
    }

    /// Changes the password of the account in the vault by updating the password in the encrypted data file.
    ///
    /// # Arguments
    /// * `new_password` - The new password to set for the account, wrapped in a `SecretBox`.
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
    ///
    /// # Errors
    /// * If the vault does not exist, if the account data cannot be decrypted, or if the file cannot be written to.
    pub fn change_account_password(
        &mut self,
        new_password: SecretBox<Vec<u8>>,
    ) -> Result<(), Error> {
        let mut userpass = self.decrypt_from_file()?;
        userpass.password = new_password;

        self.encrypt_to_file(userpass)?;

        Ok(())
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

/// Prints all entries in the vault, including account names, usernames, and decrypted passwords.
///
/// # Arguments
/// * `vault_name` - The name of the vault to print entries from.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault does not exist, if the account names cannot be found, or if decryption fails.
pub fn print_vault_entries(vault_name: &str) -> Result<(), Error> {
    let mut userpass: UserPass;

    let mut store = Store::new(vault_name, NULL)?;
    let account_names = Locations::find_account_names(&store.locations)?;

    if account_names.is_empty() {
        warn!(
            "\nNo accounts found in vault `{}`. You can create an account with `fmp -a`",
            vault_name
        );
        return Ok(());
    };

    let mut table = Table::new();
    table.add_row(row!["Account", "Username", "Password"]);

    for s in account_names.iter() {
        store = Store::new(vault_name, s)?;
        userpass = Store::decrypt_from_file(&mut store)?;

        let mut decrypted_password = SecretBox::new(Box::new(decrypt_variable(
            &mut store.ctx,
            userpass.password.expose_secret().as_slice(),
        )?));

        #[cfg(unix)]
        unsafe {
            libc::mlock(
                decrypted_password.expose_secret().as_ptr() as *const c_void,
                decrypted_password.expose_secret().len(),
            )
        };

        #[cfg(windows)]
        unsafe {
            use windows::Win32::System::Memory::VirtualLock;
            VirtualLock(
                decrypted_password.expose_secret().as_ptr() as *const c_void,
                decrypted_password.expose_secret().len(),
            );
        }

        table.add_row(row![
            s,
            userpass.username,
            String::from_utf8_lossy(decrypted_password.expose_secret())
        ]);

        decrypted_password.zeroize();
    }

    table.printstd();

    Ok(())
}

#[cfg(test)]
#[path = "tests/vault_tests.rs"]
mod vault_tests;
