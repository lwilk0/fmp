//! This file preforms operations based on specified flags.
//! These functions should only be called from the main function.

use crate::{
    crypto::securely_retrieve_password,
    input::{choice, input},
    password::{calculate_entropy, generate_password},
    vault::{Locations, Store, UserPass, rename_directory},
};
use anyhow::Error;
use fs_extra::dir::{CopyOptions, copy};
use gpgme::{Context, Protocol};
use log::{info, warn};
use secrecy::SecretBox;
use std::fs::{create_dir_all, read_to_string, remove_dir_all, write};

const NULL: &str = "null";

/// Creates a new vault with a specified name and associates it with a GPG key.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault already exists and the user chooses not to overwrite it, or if there are issues with GPG key retrieval or file operations.
pub fn create_new_vault() -> Result<(), Error> {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

    let vault_name: String = input("What should the vault be called?")?;
    let locations = Locations::new(&vault_name, NULL)?;

    if locations.vault_location.exists() {
        let user_choice_overwrite = choice(
            "\nVault already exists. Do you want to overwrite it? (y/n)",
            &["y", "n"],
        )?;

        if user_choice_overwrite == "y" {
            remove_dir_all(&locations.vault_location)?;
            info!("\nExisting vault `{}` was overwritten.", vault_name);
        } else {
            warn!("\nVault creation cancelled by the user.");
            return Ok(());
        }
    }

    info!(
        "\nWhat email address should the vault be associated with? (This should be a public key you have imported into GPG)"
    );

    let mut recipient: String = input(
        "\nYou can use the command `gpg --list-keys` to see your keys. You can create a new key with `gpg --full-generate-key`.",
    )?;

    // Check if the recipient exists in the GPG keyring
    while ctx.get_key(&recipient).is_err() {
        recipient = input(format!(
            "\nThe recipient `{}` does not exist in your GPG keyring. Please enter a valid email address.",
            recipient
        ).as_str())?;
    }

    Locations::initialize_vault(&locations)?;

    write(locations.recipient_location, recipient).map_err(|e| anyhow::anyhow!("{}", e))?;

    info!(
        "Vault `{}` created successfully at {:?}.",
        vault_name, locations.vault_location
    );

    info!("\nYou can add accounts to the vault with the `fmp -a` command.");
    info!(
        "NOTE: By default, GPG caches your passphrase for 10 minutes. See `https://github.com/lwilk0/Forgot-My-Passwords/GPGCACHE.md`.)"
    );

    Ok(())
}

/// Adds a new account to a specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault to which the account will be added.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the account already exists and the user chooses not to overwrite it, or if there are issues with file operations or password retrieval.
pub fn add_account(vault_name: &str) -> Result<(), Error> {
    let account_name: String = input("\nWhat should the account be called?")?;

    let mut store = Store::new(vault_name, &account_name)?;

    if store.locations.account_location.exists() {
        let user_choice_overwrite = choice(
            "\nAccount already exists. Do you want to overwrite it? (y/n)",
            &["y", "n"],
        )?;

        if user_choice_overwrite == "y" {
            remove_dir_all(&store.locations.account_location)?;
            info!("Existing account `{}` was overwritten.", account_name);
        } else {
            warn!("\nAccount creation cancelled by the user.");
            return Ok(());
        }
    } else {
        store.locations.create_account_directory()?;
    }

    let recipient = read_to_string(store.locations.recipient_location.clone())?;

    let username: String = input("\nEnter the username for the account:")?;
    let encrypted_password = SecretBox::new(Box::new(securely_retrieve_password(
        "\nEnter the password for the account",
        &mut store.ctx,
        recipient.as_str(),
    )?));

    let userpass = UserPass {
        username,
        password: encrypted_password,
    };

    store.encrypt_to_file(userpass)?;

    let repeat = choice("\nDo you want to add another account? (y/n)", &["y", "n"])?;

    if repeat == "y" {
        add_account(vault_name)?;
    } else {
        info!("\nDone!");
    }
    Ok(())
}

/// Creates a backup of a specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault to back up.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault does not exist, or if there are issues with file operations.
pub fn create_backup(vault_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, NULL)?;

    create_dir_all(&locations.backup_location)?;

    let options = CopyOptions::new();
    copy(
        &locations.vault_location,
        &locations.backup_location,
        &options,
    )?;

    info!(
        "Backup created successfully at {:?}",
        locations.backup_location
    );

    Ok(())
}

/// Installs a backup of a specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault to install the backup into.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the backup does not exist, or if there are issues with file operations.
pub fn install_backup(vault_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, NULL)?;

    if !locations.backup_location.exists() {
        return Err(anyhow::anyhow!(
            "Backup does not exist at {:?}",
            locations.backup_location
        ));
    }

    let options = CopyOptions::new();
    copy(
        &locations.backup_location,
        &locations.vault_location,
        &options,
    )?;

    info!(
        "Backup installed successfully from {:?} to {:?}",
        locations.backup_location, locations.vault_location
    );

    Ok(())
}

/// Deletes an account from a specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault from which the account will be deleted.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the account does not exist in the vault, or if there are issues with file operations.
pub fn delete_account_from_vault(vault_name: &str) -> Result<(), Error> {
    let account_name: String = input("\nEnter the name of the account you want to delete: ")?;

    let locations = Locations::new(vault_name, account_name.as_str())?;

    if locations.account_location.exists() {
        remove_dir_all(&locations.account_location)?;
        info!("\nAccount `{}` deleted successfully.", account_name);
    } else {
        return Err(anyhow::anyhow!(
            "\nAccount `{}` does not exist in vault `{}`. Check for typos or run `fmp -a` to add it.",
            account_name,
            vault_name
        ));
    }

    Ok(())
}

/// Deletes a specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault to delete.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the vault does not exist, or if there are issues with file operations.
pub fn delete_vault(vault_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, NULL)?;

    if locations.vault_location.exists() {
        remove_dir_all(&locations.vault_location)?;
        info!("Vault `{}` deleted successfully.", vault_name);
    } else {
        return Err(anyhow::anyhow!(
            "Vault `{}` does not exist. Try checking for typos.",
            vault_name
        ));
    }

    Ok(())
}

/// Renames a specified vault.
///
/// # Arguments
/// * `old_vault_name` - The current name of the vault to rename.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the old vault does not exist, or if there are issues with file operations.
pub fn rename_vault(old_vault_name: &str) -> Result<(), Error> {
    let new_vault_name: String = input("\nEnter the new name for the vault:")?;

    let old_locations = Locations::new(old_vault_name, NULL)?;
    let new_locations = Locations::new(&new_vault_name, NULL)?;

    rename_directory(&old_locations.vault_location, &new_locations.vault_location)?;

    rename_directory(
        &old_locations.backup_location,
        &new_locations.backup_location,
    )?;

    info!(
        "\nVault `{}` renamed to `{}` successfully.",
        old_vault_name, new_vault_name
    );

    Ok(())
}

/// Changes the username of an account in a specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault containing the account.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
pub fn change_account_username(vault_name: &str) -> Result<(), Error> {
    let account_name: String = input("\nEnter the name of the account you want to delete: ")?;

    let mut store = Store::new(vault_name, account_name.as_str())?;

    let new_username: String = input("\nEnter the new username for the account:")?;

    store.change_account_username(new_username.as_str())?;

    info!(
        "\nUsername for account `{}` changed successfully.",
        account_name
    );

    Ok(())
}

/// Changes the password of an account in a specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault containing the account.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
pub fn change_account_password(vault_name: &str) -> Result<(), Error> {
    let account_name: String = input("\nEnter the name of the account you want to delete: ")?;

    let mut store = Store::new(vault_name, account_name.as_str())?;

    let recipient = read_to_string(store.locations.recipient_location.clone())?;

    let new_password = SecretBox::new(Box::new(securely_retrieve_password(
        "\nEnter the new password for the account",
        &mut store.ctx,
        recipient.as_str(),
    )?));

    store.change_account_password(new_password)?;

    info!(
        "\nPassword for account `{}` changed successfully.",
        account_name
    );

    Ok(())
}

/// Changes the name of an account in a specified vault.
///
/// # Arguments
/// * `vault_name` - The name of the vault containing the account.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
pub fn change_account_name(vault_name: &str) -> Result<(), Error> {
    let old_account_name: String = input("\nEnter the name of the account you want to rename:")?;
    let new_account_name: String = input("\nEnter the new name for the account:")?;

    let locations_old = Locations::new(vault_name, old_account_name.as_str())?;
    let locations_new = Locations::new(vault_name, new_account_name.as_str())?;

    rename_directory(
        &locations_old.account_location,
        &locations_new.account_location,
    )?;

    info!(
        "\nAccount `{}` renamed to `{}` successfully.",
        old_account_name, new_account_name
    );

    Ok(())
}

/// Generates a new password of a specified length and prints it to stdout.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the input for password length cannot be parsed, or if there are issues with generating the password.
pub fn generate_new_password() -> Result<(), Error> {
    let length: usize = input("Enter the length of the password:")?;

    let password = generate_password(length);

    info!("\nGenerated password: {}", password);

    Ok(())
}

/// Calculates the entropy of a password input by the user and prints the result to stdout.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the input for the password cannot be read or parsed, or if there are issues with calculating the entropy.
pub fn calculate_entropy_from_input() -> Result<(), Error> {
    let password: String = input("Enter the password to calculate entropy:")?;

    let (entropy, rating) = calculate_entropy(&password);

    info!("\nPassword Entropy: {:.2} bits", entropy);
    info!("Password Rating: {}", rating);

    Ok(())
}
