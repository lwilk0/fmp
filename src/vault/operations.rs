//! High-level vault operations and convenience functions.

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

use crate::models::Account;
use crate::storage::{Locations, Store};
use anyhow::Error;
use gpgme::Context;
use std::{
    cell::RefCell,
    fs::{File, remove_dir_all},
    io::{BufReader, Read, Write},
    rc::Rc,
};
use zeroize::Zeroize;

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
    store.storage_locations.does_vault_exist()?;
    store.storage_locations.does_account_exist()?;

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
    store.storage_locations.does_vault_exist()?;
    store.storage_locations.create_account_directory()?;
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
    store.storage_locations.does_vault_exist()?;
    store.storage_locations.does_account_exist()?;
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
#[cfg(test)]
pub fn create_vault(
    vault_name: &str,
    recipient: &str,
    ctx: Rc<RefCell<Context>>,
) -> Result<(), Error> {
    create_vault_prepare(vault_name, recipient)?;
    create_vault_finalize(vault_name, recipient, ctx)?;
    Ok(())
}

pub fn create_vault_prepare(vault_name: &str, recipient: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, "");
    locations.initialize_vault()?;

    let mut recipient_file = File::create(&locations.recipient)?;
    recipient_file.write_all(recipient.as_bytes())?;

    Ok(())
}

pub fn create_vault_finalize(
    vault_name: &str,
    recipient: &str,
    ctx: Rc<RefCell<Context>>,
) -> Result<(), Error> {
    let locations = Locations::new(vault_name, "");

    let recipient_key = ctx.borrow_mut().get_key(recipient).map_err(|e| {
        anyhow::anyhow!(
            "Failed to find recipient `{}` for encryption. Error: {}",
            recipient,
            e
        )
    })?;

    let gate_data = b"gate";
    let mut output = Vec::new();
    ctx.borrow_mut()
        .encrypt([&recipient_key], &gate_data[..], &mut output)?;

    let mut gate_file = File::create(&locations.gate)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        gate_file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }

    gate_file.write_all(&output)?;

    Ok(())
}

/// Attempt to decrypt the vault's gate file to warm up GPG (triggers passphrase prompt).
///
/// # Arguments
/// * `vault_name` - The name of the vault to warm up GPG for.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the gate file cannot be read or decrypted.
pub fn warm_up_gpg_blocking(vault_name: &str) -> Result<Vec<u8>, Error> {
    let locations = Locations::new(vault_name, "");
    let mut encrypted = Vec::new();

    let file = File::open(&locations.gate)?;
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut encrypted)?;
    Ok(encrypted)
}

// finalize on main thread: uses `ctx` (Rc<RefCell<Context>>) to decrypt
pub fn warm_up_gpg_finalize(encrypted: Vec<u8>, ctx: Rc<RefCell<Context>>) -> Result<(), Error> {
    let mut out = Vec::new();
    ctx.borrow_mut()
        .decrypt(&encrypted, &mut out)
        .map_err(|e| anyhow::anyhow!("Failed to decrypt warm-up file. Error: {}", e))?;

    out.zeroize();

    Ok(())
}

/// Deletes an account from the specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault containing the account.
/// * `account_name` - The name of the account to delete.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault does not exist, if the account does not exist, or if the account directory cannot be removed.
pub fn delete_account(vault_name: &str, account_name: &str) -> Result<(), Error> {
    let store = Store::new(vault_name, account_name)?;
    store.storage_locations.does_vault_exist()?;
    store.storage_locations.does_account_exist()?;

    remove_dir_all(&store.storage_locations.account)?;

    Ok(())
}
