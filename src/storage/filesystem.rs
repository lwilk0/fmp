//! File system utility functions for directory operations.

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

use crate::storage::locations::Locations;
use anyhow::Error;
use fs_extra::dir::{CopyOptions, copy};
use std::fs::{create_dir_all, read_dir, remove_dir_all, rename};
use std::path::PathBuf;

/// Renames a directory from `old_path` to `new_path`.
///
/// # Arguments
/// * `old_path` - The current path of the directory to be renamed.
/// * `new_path` - The new path for the directory.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
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

/// Creates a backup of the specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault to back up.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the backup location already exists and cannot be removed, or if there are issues with file operations.
pub fn create_backup(vault_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, "");

    if locations.backup.exists() {
        remove_dir_all(&locations.backup)?;
    }

    create_dir_all(&locations.backup)?;

    let options = CopyOptions::new();
    copy(&locations.vault, &locations.backup, &options)?;

    Ok(())
}

/// Installs a backup of the specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault to restore.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the backup does not exist, or if there are issues with file operations.
pub fn install_backup(vault_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, "");

    if !locations.backup.exists() {
        return Err(anyhow::anyhow!(
            "Backup does not exist at {:?}",
            locations.backup
        ));
    }

    remove_dir_all(&locations.vault)?;

    let options = CopyOptions::new();

    copy(
        locations.backup.join(vault_name),
        locations.fmp.join("vaults"),
        &options,
    )?;

    Ok(())
}

/// Deletes a vault and all its contents.
///
/// # Arguments
/// * `vault_name` - The name of the vault to delete.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault does not exist or cannot be deleted, an error is returned.
pub fn delete_vault(vault_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, "");

    if !locations.vault.exists() {
        return Err(anyhow::anyhow!("Vault `{}` does not exist.", vault_name));
    }

    remove_dir_all(&locations.vault)?;

    // Also remove backup if it exists
    let backup_path = locations.backup.join(vault_name);
    if backup_path.exists() {
        remove_dir_all(&backup_path)?;
    }

    // Remove vault from stats file
    remove_vault_from_stats(vault_name)?;

    Ok(())
}

/// Renames a vault from `old_name` to `new_name`.
///
/// # Arguments
/// * `old_name` - The current name of the vault.
/// * `new_name` - The new name for the vault.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault does not exist, the new name already exists, or the renaming operation fails, an error is returned.
pub fn rename_vault(old_name: &str, new_name: &str) -> Result<(), Error> {
    let old_locations = Locations::new(old_name, "");
    let new_locations = Locations::new(new_name, "");

    if !old_locations.vault.exists() {
        return Err(anyhow::anyhow!("Vault `{}` does not exist.", old_name));
    }

    if new_locations.vault.exists() {
        return Err(anyhow::anyhow!("Vault `{}` already exists.", new_name));
    }

    rename(&old_locations.vault, &new_locations.vault)?;

    // Also rename backup if it exists
    let old_backup = old_locations.backup.join(old_name);
    let new_backup = new_locations.backup.join(new_name);
    if old_backup.exists() {
        if let Some(backup_parent) = new_backup.parent() {
            create_dir_all(backup_parent)?;
        }
        rename(&old_backup, &new_backup)?;
    }

    Ok(())
}

/// Renames an account within a vault from `old_name` to `new_name`.
///
/// # Arguments
/// * `vault_name` - The name of the vault containing the account.
/// * `old_name` - The current name of the account.
/// * `new_name` - The new name for the account.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault or account does not exist, the new account name already exists, or the renaming operation fails, an error is returned.
pub fn rename_account(vault_name: &str, old_name: &str, new_name: &str) -> Result<(), Error> {
    let old_locations = Locations::new(vault_name, old_name);
    let new_locations = Locations::new(vault_name, new_name);

    // Check if vault exists
    old_locations.does_vault_exist()?;

    if !old_locations.account.exists() {
        return Err(anyhow::anyhow!(
            "Account `{}` does not exist in vault `{}`.",
            old_name,
            vault_name
        ));
    }

    if new_locations.account.exists() {
        return Err(anyhow::anyhow!(
            "Account `{}` already exists in vault `{}`.",
            new_name,
            vault_name
        ));
    }

    rename(&old_locations.account, &new_locations.account)?;

    Ok(())
}

/// Deletes an account and all its contents from a vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault containing the account.
/// * `account_name` - The name of the account to delete.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault or account does not exist or cannot be deleted, an error is returned.
pub fn delete_account(vault_name: &str, account_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, account_name);

    // Check if vault exists
    locations.does_vault_exist()?;

    if !locations.account.exists() {
        return Err(anyhow::anyhow!(
            "Account `{}` does not exist in vault `{}`.",
            account_name,
            vault_name
        ));
    }

    remove_dir_all(&locations.account)?;

    Ok(())
}

/// Lists all available backups for a vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault to check for backups.
///
/// # Returns
/// * `Result<Vec<String>, Error>` - Returns a vector of backup names on success, or an error on failure.
///
/// # Errors
/// * If reading the backup directory fails, an error is returned.
pub fn list_backups(vault_name: &str) -> Result<Vec<String>, Error> {
    let locations = Locations::new(vault_name, "");
    let backup_vault_path = locations.backup.join(vault_name);

    if !backup_vault_path.exists() {
        return Ok(Vec::new());
    }

    read_directory(&backup_vault_path)
}

/// Checks if a backup exists for the specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault to check.
///
/// # Returns
/// * `bool` - Returns `true` if a backup exists, `false` otherwise.
pub fn backup_exists(vault_name: &str) -> bool {
    let locations = Locations::new(vault_name, "");
    locations.backup.join(vault_name).exists()
}

/// Deletes the backup for the specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault whose backup should be deleted.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the backup does not exist or cannot be deleted, an error is returned.
pub fn delete_backup(vault_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, "");
    let backup_path = locations.backup.join(vault_name);

    if !backup_path.exists() {
        return Err(anyhow::anyhow!(
            "Backup for vault `{}` does not exist.",
            vault_name
        ));
    }

    remove_dir_all(&backup_path)?;

    Ok(())
}

/// Removes a vault from the vault statistics file
///
/// # Arguments
/// * `vault_name` - The name of the vault to remove from stats
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure
fn remove_vault_from_stats(vault_name: &str) -> Result<(), Error> {
    use std::fs;

    let locations = Locations::new("", "");
    let stats_file = locations.fmp.join("vault_stats.txt");

    if !stats_file.exists() {
        return Ok(()); // No stats file, nothing to remove
    }

    let content = fs::read_to_string(&stats_file)?;
    let updated_content: String = content
        .lines()
        .filter(|line| !line.starts_with(&format!("{}:", vault_name)))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&stats_file, updated_content)?;

    Ok(())
}
