use clap::Parser;
use import_handle;
use cmd_lib::run_cmd;
use std::path::Path;

mod password;
mod vault;
mod json;
mod account;
mod checks;

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

    /// Calculate password entropy.
    /// used as -e --entropy
    #[clap(short = 'e', long = "entropy")]
    flag_e: bool,
}

fn main() {
    checks::os_check();
    checks::vault_exists_check(vault::get_fmp_vault_location());
    // Stores flag user input bools
    let opts = Options::parse();

    // If flag -a or --add is used
    if opts.flag_a == true {
        // Decrypt vault
        vault::decrypt_fmp_vault();
        let account = account::read_account(account::get_account_location());
        // Get user inputs
        let name = import_handle::get_string_input("What should the account be named? ");
        let username = import_handle::get_string_input("\nWhat is the account username?");
        let password = import_handle::get_string_input("\nWhat is the account password");
        // Create new account
        json::new_json_account(vault::get_fmp_vault_location(), name.as_str(), username.as_str(), password.as_str(), account);
        // Exit
        vault::encrypt_and_exit();
    }

    // If flag -d or --delete is used
    if opts.flag_d == true {
        // Decrypt vault
        vault::decrypt_fmp_vault();
        // Get account name
        let name = import_handle::get_string_input("What account should be removed? ");
        // Removes account 
        json::remove_account(vault::get_fmp_vault_location(), name.as_str());
        let mut account = account::read_account(account::get_account_location());
        account.retain(|account| *account != name);
        account::write_account(account::get_account_location(), &account);
        // Exit
        vault::encrypt_and_exit();
    }

    // If flag -p or --change-password is used
    if opts.flag_p == true {
        // Decrypt vault
        vault::decrypt_fmp_vault();
        // Get user input
        let name = import_handle::get_string_input("What account password should be changed? ");
        let password = import_handle::get_string_input("\nWhat should the password be changed to?");
        // Changes password
        json::change_password(vault::get_fmp_vault_location(), password.as_str(), &name);
        // Exit
        vault::encrypt_and_exit();
    }

    // If flag -u or --change-username is used
    if opts.flag_u == true {
        // Decrypt vault
        vault::decrypt_fmp_vault();
        // Get user input
        let name = import_handle::get_string_input("What account username should be changed? ");
        let username = import_handle::get_string_input("\nWhat should the username be changed to?");
        // Change username
        json::change_username(vault::get_fmp_vault_location(), &username.as_str(), &name);
        // Exit
        vault::encrypt_and_exit();
    }

    // If flag -c or --create is used
    if opts.flag_c == true {
        println!("FMP SETUP\n");
        println!("Creating .fmpVault in home directory...\n");
        let encrypted_vault_location = format!("{}/.tar.gz.gpg", vault::get_fmp_vault_location());
        let vault_location = vault::get_fmp_vault_location();
        // If encrypted vault exists
        if Path::new(&encrypted_vault_location).exists() {
            let user_input = "";
            // Ask user for input, handles incorect input
            if user_input != "y" && user_input != "yes" && user_input != "no" && user_input != "n" {
                let user_input = import_handle::get_string_input(".fmpVault.tar.gz.gpg already exists, remove it? y(es), n(o)").to_lowercase();
            }
            // Remove .fmpVault.tar.gpg
            if user_input == "y" || user_input == "yes" {
                run_cmd!(rm $encrypted_vault_location).expect("Failed to remove .fmpVault");
            }
            // Exit
            else {
                vault::exit_vault(vault::get_fmp_vault_location());
            }
        }
        // Make .fmpVault folder
        run_cmd!(mkdir $vault_location).expect("Failed to make .fmpVault folder");
        println!("Done");
        println!("Creating accounts file...\n");
        // Create accounts file
        run_cmd!(touch $vault_location/accounts).expect("Failed to make account file");
        println!("Done\n");
        // Exit
        vault::encrypt_and_exit();
         
    }

    // If flag -e or --entropy is used
    if opts.flag_e == true {
        let password: String = import_handle::get_string_input("Enter the password for entropy calculation");
        password::calculate_entropy(password);
        vault::exit_vault(vault::get_fmp_vault_location());
    }

    // If no flags are supplied
    vault::decrypt_fmp_vault();
    vault::read_vault();
    vault::delete_vault(vault::get_fmp_vault_location());
}