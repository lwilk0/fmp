//! Forgot-My-Password(FMP) - A simple password vault application.

use anyhow::Error;
use clap::Parser;
use vault::print_vault_entries;

mod crypto;
mod flags;
mod input;
mod password;
mod vault;
use crate::{
    flags::{
        add_account, calculate_entropy_from_input, change_account_name, change_account_password,
        change_account_username, create_backup, create_new_vault, delete_account_from_vault,
        delete_vault, generate_new_password, install_backup, rename_vault,
    },
    input::input,
    vault::Locations,
};

#[derive(Debug, Parser)]
struct Flags {
    /// Add an account to vault.
    /// used as: -a, --add
    #[clap(short = 'a', long = "add")]
    flag_a: bool,

    /// Create a vault backup.
    /// used as: -b, --backup
    #[clap(short = 'b', long = "backup")]
    flag_b: bool,

    /// Create a new vault.
    /// used as: -c, --create-vault
    #[clap(short = 'c', long = "create-vault")]
    flag_c: bool,

    /// Change an account's name.
    /// used as: -C, --change-account-name
    #[clap(short = 'C', long = "change-account-name")]
    flag_can: bool,

    /// Delete an account from a vault.
    /// used as: -d, --delete-account
    #[clap(short = 'd', long = "delete-account")]
    flag_d: bool,

    /// Delete a vault.
    /// used as: -D, --delete-vault
    #[clap(short = 'D', long = "delete-vault")]
    flag_dv: bool,

    /// Calculate a password's entropy.
    /// used as: -e, --entropy
    #[clap(short = 'e', long = "entropy")]
    flag_e: bool,

    /// Generate a password.
    /// used as: -g, --generate-password
    #[clap(short = 'g', long = "generate-password")]
    flag_g: bool,

    /// Rename a vault.
    /// used as: -r, --rename-vault
    #[clap(short = 'r', long = "rename-vault")]
    flag_r: bool,

    /// Change an account's username.
    /// used as: -u, --change-username
    #[clap(short = 'u', long = "change-username")]
    flag_u: bool,

    /// Install a vault backup.
    /// used as: -i, --install-backup
    #[clap(short = 'i', long = "install-backup")]
    flag_i: bool,

    /// Change an account's password.
    /// used as: -p, --change-password
    #[clap(short = 'p', long = "change-password")]
    flag_p: bool,
}

fn main() -> Result<(), Error> {
    let flags = Flags::parse();

    if flags.flag_c {
        create_new_vault()?;
        return Ok(());
    } else if flags.flag_g {
        generate_new_password()?;
        return Ok(());
    } else if flags.flag_e {
        calculate_entropy_from_input()?;
        return Ok(());
    }

    let mut vault_name: String = input("What vault do you want to access?")?;

    while Locations::does_vault_exist(vault_name.as_str()).is_err() {
        println!("Vault '{}' does not exist.", vault_name);
        vault_name = input("What vault do you want to access?")?;
    }

    if flags.flag_a {
        add_account(vault_name.as_str())?;
        Ok(())
    } else if flags.flag_b {
        create_backup(vault_name.as_str())?;
        Ok(())
    } else if flags.flag_can {
        change_account_name(vault_name.as_str())?;
        Ok(())
    } else if flags.flag_d {
        delete_account_from_vault(vault_name.as_str())?;
        Ok(())
    } else if flags.flag_dv {
        delete_vault(vault_name.as_str())?;
        Ok(())
    } else if flags.flag_r {
        rename_vault(vault_name.as_str())?;
        Ok(())
    } else if flags.flag_u {
        change_account_username(vault_name.as_str())?;
        Ok(())
    } else if flags.flag_i {
        install_backup(vault_name.as_str())?;
        Ok(())
    } else if flags.flag_p {
        change_account_password(vault_name.as_str())?;
        Ok(())
    } else {
        print_vault_entries(vault_name.as_str())?;
        Ok(())
    }
}
