use clap::Parser;
use import_handle;
use cmd_lib::run_cmd;
use std::path::Path;
mod vault;
mod json;
mod account;

#[derive(Debug, Parser)]
struct Options {

    /// Add an account to vault.
    /// used as: -a, --add
    #[clap(short = 'a', long = "add")]
    flag_a: bool,

    /// Delete account from vault.
    /// used as: -d, --delete
    #[clap(short = 'd', long = "delete")]
    flag_d: bool,

    /// Change password in account.
    /// used as: -p , --change-password 
    #[clap(short = 'p', long = "change-password")]
    flag_p: bool,

    /// Change username in account.
    /// used as: -u , --change-username 
    #[clap(short = 'u', long = "change-username")]
    flag_u: bool,

    /// Create vault.
    /// used as -c --create-vault
    #[clap(short = 'c', long = "create-vault")]
    flag_c: bool,
}

fn main() {
    // Stores flag user input bools
    let opts = Options::parse();

    // If flag -a or --add is used
    if opts.flag_a == true {
        vault::decrypt_fmp_vault();
        let account = account::read_account(account::get_account_location());
        let name = import_handle::get_string_input("What should the account be named? ");
        let username = import_handle::get_string_input("\nWhat is the account username?");
        let password = import_handle::get_string_input("\nWhat is the account password");
        json::new_json_account(vault::get_fmp_vault_location(), name.as_str(), username.as_str(), password.as_str(), account);
        vault::encrypt_fmp_vault();
        vault::delete_vault(vault::get_fmp_vault_location());
        vault::exit_vault(vault::get_fmp_vault_location());
    }

    // If flag -d or --delete is used
    if opts.flag_d == true {
        vault::decrypt_fmp_vault();
        let name = import_handle::get_string_input("What account should be removed? ");
        json::remove_account(vault::get_fmp_vault_location(), name.as_str());
        let mut account = account::read_account(account::get_account_location());
        account.retain(|account| *account != name);
        account::write_account(account::get_account_location(), &account);
        vault::encrypt_fmp_vault();
        vault::delete_vault(vault::get_fmp_vault_location());
        vault::exit_vault(vault::get_fmp_vault_location());
    }

    // If flag -p or --change-password is used
    if opts.flag_p == true {
        vault::decrypt_fmp_vault();
        let name = import_handle::get_string_input("What account password should be changed? ");
        let password = import_handle::get_string_input("\nWhat should the password be changed to?");
        json::change_password(vault::get_fmp_vault_location(), password.as_str(), &name);
        vault::encrypt_fmp_vault();
        vault::delete_vault(vault::get_fmp_vault_location());
        vault::exit_vault(vault::get_fmp_vault_location());
    }

    // If flag -u or --change-username is used
    if opts.flag_u == true {
        vault::decrypt_fmp_vault();
        let name = import_handle::get_string_input("What account username should be changed? ");
        let username = import_handle::get_string_input("\nWhat should the username be changed to?");
        json::change_username(vault::get_fmp_vault_location(), &username.as_str(), &name);
        vault::encrypt_fmp_vault();
        vault::delete_vault(vault::get_fmp_vault_location());
        vault::exit_vault(vault::get_fmp_vault_location());
    }

    // If flag -c or --create is used
    if opts.flag_c == true {
        println!("FMP SETUP\n");
        println!("Creating .fmpVault in home directory...\n");
        let encrypted_vault_location = format!("{}/.tar.gz.gpg", vault::get_fmp_vault_location());
        let vault_location = vault::get_fmp_vault_location();

        if Path::new(&encrypted_vault_location).exists() {
            let user_input = "";
            if user_input != "y" && user_input != "yes" && user_input != "no" && user_input != "n" {
                let user_input = import_handle::get_string_input(".fmpVault.tar.gz.gpg already exists, remove it? y(es), n(o)").to_lowercase();
            }
            if user_input == "y" || user_input == "yes" {
                run_cmd!(rm $encrypted_vault_location).expect("Failed to remove .fmpVault");
            }
            else {
                vault::exit_vault(vault::get_fmp_vault_location());
            }
        }
        run_cmd!(mkdir $vault_location).expect("Failed to make .fmpVault folder");

        println!("Done");
        println!("Creating accounts file...\n");

        run_cmd!(touch $vault_location/accounts).expect("Failed to make account file");
        println!("Done\n");

        vault::encrypt_fmp_vault();
        vault::delete_vault(vault::get_fmp_vault_location());
        vault::exit_vault(vault::get_fmp_vault_location());
         
    }
    vault::decrypt_fmp_vault();
    vault::read_vault();
    vault::delete_vault(vault::get_fmp_vault_location());
}