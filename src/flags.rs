//! This module preforms operations based on specified flags.
//! These functions should only be called from the main function.

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
    gui::FmpApp,
    vault::{Locations, Store, rename_directory},
};
use anyhow::Error;
use fs_extra::dir::{CopyOptions, copy};
use gpgme::{Context, Protocol};
use std::fs::{create_dir_all, remove_dir_all, write};

const NULL: &str = "null";

/// Creates a new vault with the specified name and recipient.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the vault name and recipient.
///
/// # Returns
/// * "Result<(), Error>" - Returns "Ok(())" on success, or an error on failure.
///
/// # Errors
/// * If the vault already exists and the user chooses not to overwrite it, or if there are issues with file operations or encryption.
pub fn create_new_vault(app: &mut FmpApp) -> Result<(), Error> {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

    let locations = Locations::new(app.vault_name_create.as_str(), NULL)?;

    if locations.vault_location.exists() {
        Err(anyhow::anyhow!(
            "Vault \"{}\" already exists. Please choose a different name or delete the existing vault.",
            app.vault_name_create
        ))?;
    }

    if ctx.get_key(&app.recipient).is_err() {
        Err(anyhow::anyhow!(
            "Recipient \"{}\" does not exist. Please ensure you have imported the public key into GPG.",
            app.recipient
        ))?;
    }

    Locations::initialize_vault(&locations)?;

    write(locations.recipient_location, &app.recipient).map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}

/// Adds a new account to the specified vault.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the vault and account details.
///
/// # Returns
/// * "Result<(), Error>" - Returns "Ok(())" on success, or an error on failure.
///
/// # Errors
/// * If the account already exists and the user chooses not to overwrite it, or if there are issues with file operations or encryption.
pub fn add_account(app: &mut FmpApp) -> Result<(), Error> {
    let mut store = Store::new(app.vault_name.as_str(), app.account_name_create.as_str())?;

    if store.locations.account_location.exists() {
        Err(anyhow::anyhow!(
            "Account \"{}\" already exists. Please choose a different name or delete the existing account.",
            app.account_name_create
        ))?;
    } else {
        store.locations.create_account_directory()?;
    }

    store.encrypt_to_file(&app.userpass)?;

    Ok(())
}

/// Creates a backup of the specified vault.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the vault name.
///
/// # Returns
/// * "Result<(), Error>" - Returns "Ok(())" on success, or an error on failure.
///
/// # Errors
/// * If the backup location already exists and cannot be removed, or if there are issues with file operations.
pub fn create_backup(app: &mut FmpApp) -> Result<(), Error> {
    let locations = Locations::new(app.vault_name.as_str(), NULL)?;

    if locations.backup_location.exists() {
        remove_dir_all(&locations.backup_location)?;
    }

    create_dir_all(&locations.backup_location)?;

    let options = CopyOptions::new();
    copy(
        &locations.vault_location,
        &locations.backup_location,
        &options,
    )?;

    Ok(())
}

/// Installs a backup of the specified vault.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the vault name.
///
/// # Returns
/// * "Result<(), Error>" - Returns "Ok(())" on success, or an error on failure.
///
/// # Errors
/// * If the backup does not exist, or if there are issues with file operations.
pub fn install_backup(app: &mut FmpApp) -> Result<(), Error> {
    let locations = Locations::new(app.vault_name.as_str(), NULL)?;

    if !locations.backup_location.exists() {
        return Err(anyhow::anyhow!(
            "Backup does not exist at {:?}",
            locations.backup_location
        ));
    }

    remove_dir_all(&locations.vault_location)?;

    let options = CopyOptions::new();

    copy(
        locations.backup_location.join(app.vault_name.as_str()),
        locations.fmp_location.join("vaults"),
        &options,
    )?;

    Ok(())
}

/// Deletes an account from a specified vault.
///
/// # Arguments
/// * "vault_name" - The name of the vault from which the account will be deleted.
///
/// # Returns
/// * "Result<(), Error>" - Returns "Ok(())" on success, or an error on failure.
///
/// # Errors
/// * If the account does not exist in the vault, or if there are issues with file operations.
pub fn delete_account_from_vault(app: &mut FmpApp) -> Result<(), Error> {
    let locations = Locations::new(app.vault_name.as_str(), app.account_name.as_str())?;

    if locations.account_location.exists() {
        remove_dir_all(&locations.account_location)?;
    } else {
        return Err(anyhow::anyhow!(
            "Account \"{}\" does not exist in vault \"{}\". Check for typos or run \"fmp -a\" to add it.",
            app.account_name,
            app.vault_name
        ));
    }

    Ok(())
}

/// Deletes a specified vault.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the vault name.
///
/// # Returns
/// * "Result<(), Error>" - Returns "Ok(())" on success, or an error on failure.
///
/// # Errors
/// * If the vault does not exist, or if there are issues with file operations.
pub fn delete_vault(app: &mut FmpApp) -> Result<(), Error> {
    let locations = Locations::new(app.vault_name.as_str(), NULL)?;

    locations.does_vault_exist()?;

    remove_dir_all(&locations.vault_location)?;

    Ok(())
}

/// Renames a vault.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the vault name and new vault name.
///
/// # Returns
/// * "Result<(), Error>" - Returns "Ok(())" on success, or an error on failure.
///
/// # Errors
/// * If the old vault does not exist, or if there are issues with file operations.
pub fn rename_vault(app: &mut FmpApp) -> Result<(), Error> {
    let old_locations = Locations::new(app.vault_name.as_str(), NULL)?;
    let new_locations = Locations::new(app.vault_name_create.as_str(), NULL)?;

    old_locations.does_vault_exist()?;

    rename_directory(&old_locations.vault_location, &new_locations.vault_location)?;

    Ok(())
}

/// Changes the account data (username and password) for a specified account in a vault.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the vault and account details.
///
/// # Returns
/// * "Result<(), Error>" - Returns "Ok(())" on success, or an error on failure.
///
/// # Errors
/// * If the account does not exist in the vault, or if there are issues with file operations or encryption.
pub fn change_account_data(app: &mut FmpApp) -> Result<(), Error> {
    let mut store = Store::new(app.vault_name.as_str(), app.account_name.as_str())?;

    store.locations.does_account_exist()?;

    store.encrypt_to_file(&app.userpass)?;

    Ok(())
}

/// Changes the account name for a specified account in a vault.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the vault and account details.
///
/// # Returns
/// * "Result<(), Error>" - Returns "Ok(())" on success, or an error on failure.
///
/// # Errors
/// * If the old account does not exist, or if there are issues with file operations.
pub fn change_account_name(app: &mut FmpApp) -> Result<(), Error> {
    let locations_old = Locations::new(app.vault_name.as_str(), app.account_name.as_str())?;
    let locations_new = Locations::new(app.vault_name.as_str(), app.account_name_create.as_str())?;

    locations_old.does_account_exist()?;

    rename_directory(
        &locations_old.account_location,
        &locations_new.account_location,
    )?;

    Ok(())
}
