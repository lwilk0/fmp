use clap::Parser;a

mod account;
mod json;
mod password;
mod vault; use vault::{vault_to_access, encrypt_and_exit};
mod checks; use checks::os_check;
mod flags; use flags::{add, backup, change_account_password, change_account_username, change_vault_password, create, delete, entropy, gen_password, delete_vault_all_files, rename, no_flags};
#[derive(Debug, Parser)]
struct Options {

    /// Add an account to vault.
    /// used as: -a, --add
    #[clap(short = 'a', long = "add")]
    flag_a: bool,

    /// Backup vault or install backup
    /// user as -b, --backup
    #[clap(short = 'b', long = "backup")]
    flag_b: bool,

    /// Create vault.
    /// used as -c --create-vault
    #[clap(short = 'c', long = "create-vault")]
    flag_c: bool,

    /// Change vault password.
    /// used as -C --change-vault-password
    #[clap(short = 'C', long = "change-vault-password")]
    flag_cvp: bool,

    /// Delete account from vault.
    /// used as: -d, --delete
    #[clap(short = 'd', long = "delete")]
    flag_d: bool,

    /// Delete vault.
    /// used as: -D, --delete
    #[clap(short = 'D', long = "delete-vault")]
    flag_dv: bool,

    /// Calculate password entropy.
    /// used as -e --entropy
    #[clap(short = 'e', long = "entropy")]
    flag_e: bool,

    /// Encrypt vault.
    /// used as -E, --encrypt
    #[clap(short = 'E', long = "encrypt")]
    flag_en: bool,

    /// Generate new password.
    /// used as -g --generate-password
    #[clap(short = 'g', long = "generate-password")]
    flag_g: bool,


    /// Change password for an account.
    /// used as: -p , --change-password 
    #[clap(short = 'p', long = "change-password")]
    flag_p: bool,

    /// Rename vault.
    /// used as: -r , --rename-vault
    #[clap(short = 'r', long = "rename-vault")]
    flag_r: bool,

    /// Change username for an account.
    /// used as: -u , --change-username 
    #[clap(short = 'u', long = "change-username")]
    flag_u: bool,
}

fn main() {
    // Check users current os
    os_check();
    // Stores flag user input bools
    let opts = Options::parse();

    // If flag -c or --create is used NOTE: must be above vault location
    if opts.flag_c == true {
        create();
    }
    // Gets vault location from user
    let vault_location = vault_to_access();

    // If flag -a or --add is used
    if opts.flag_a == true {
        add(&vault_location);
    }

    // If flag -d or --delete is used
    if opts.flag_d == true {
        delete(&vault_location);
    }

    // If flag -p or --change-password is used
    if opts.flag_p == true {
        change_account_password(&vault_location);
    }

    // If flag -u or --change-username is used
    if opts.flag_u == true {
        change_account_username(&vault_location);
    }

    // If flag -e or --entropy is used
    if opts.flag_e == true {
        entropy(vault_location.clone())
    }

    // If flag -g or --generate-password is used
    if opts.flag_g == true {
        gen_password(&vault_location);
    }

    // If flag -E or --encrypt is used
    if opts.flag_en == true {
        encrypt_and_exit(&vault_location);
    }

    // If flag -b or --backup is used
    if opts.flag_b == true {
        backup(&vault_location);
    }

    // If flag -D or --delete-vault is used
    if opts.flag_dv == true {
        delete_vault_all_files(&vault_location);
    }

    // If flag -r or --rename-vault is used
    if opts.flag_r == true {
        rename(&vault_location)
    }

    // If flag -C or --change-vault-password
    if opts.flag_cvp == true {
        change_vault_password(&vault_location);
    }
    // If no flags are supplied
    no_flags(&vault_location);
}