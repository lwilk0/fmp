use clap::Parser;
use import_handle;
use std::{path::Path, process::Command};

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

    /// Generate new password.
    /// used as -g --generate-pasword
    #[clap(short = 'g', long = "generate-password")]
    flag_g: bool,

    /// Encrypt vault.
    /// used as -E, --encrypt
    #[clap(short = 'E', long = "encrypt")]
    flag_en: bool,
}

fn main() {
    checks::os_check();
    checks::vault_exists_check(vault::get_fmp_vault_location());
    // Stores flag user input bools
    let opts = Options::parse();

    // If flag -a or --add is used
    if opts.flag_a == true {
        let mut user_input: String = "y".to_string();
        // Decrypt vault
        vault::decrypt_fmp_vault();
        while user_input == "y" || user_input == "yes" {
            let account = account::read_account(account::get_account_location());
            // Get user inputs
            let mut name = import_handle::get_string_input("What should the account be named? ");
            let username = import_handle::get_string_input("\nWhat is the account username?");
            let password = import_handle::get_string_input("\nWhat is the account password");
            println!("");
            // Create new account
            let mut error_handle = json::new_json_account(vault::get_fmp_vault_location(), name.as_str(), username.as_str(), password.as_str(), account.clone());
            // Handle error
            while error_handle != "ok" {
                name = import_handle::get_string_input("Enter new account name: ");
                error_handle = json::new_json_account(vault::get_fmp_vault_location(), name.as_str(), username.as_str(), password.as_str(), account.clone());
            }
            // Ask user if they would like to add a new account
            user_input = String::new();
            while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
                user_input = import_handle::get_string_input("\nWould you like to enter a new account? (y)es, (n)o").to_lowercase();
                println!("");
            }
        }
        // Exit
        vault::encrypt_and_exit();
    }

    // If flag -d or --delete is used
    if opts.flag_d == true {
        let mut user_input: String = "y".to_string();
        // Decrypt vault
        vault::decrypt_fmp_vault();
        while user_input == "y" || user_input == "yes" {
            let account = account::read_account(account::get_account_location());
            // Get account name
            let mut name = import_handle::get_string_input("What account should be removed? ");
            println!("");
            // Removes account 
            let mut error_handle = json::remove_account(vault::get_fmp_vault_location(), name.as_str(), account.clone());
            // Handle error
            while error_handle != "ok" {
                name = import_handle::get_string_input("Enter correct account name: ");
                error_handle = json::remove_account(vault::get_fmp_vault_location(), name.as_str(), account.clone());
            }
            // Ask user if they would like to remove another account
            user_input = String::new();
            while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
                user_input = import_handle::get_string_input("\nWould you like to remove another account? (y)es, (n)o").to_lowercase();
                println!("");
            }
        }
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
        let vault_location = vault::get_fmp_vault_location();
        let encrypted_vault_location = format!("{}/.tar.gz.gpg", vault_location);
        let accounts_loaction = format!("{}/accounts", vault_location);
        let mut user_input:String = String::new();
        // If encrypted vault exists
        if Path::new(&encrypted_vault_location).exists() {
            // Ask user for input, handles incorect input
            if user_input != "y" && user_input != "yes" && user_input != "no" && user_input != "n" {
                user_input = import_handle::get_string_input(".fmpVault.tar.gz.gpg already exists, remove it? y(es), n(o)").to_lowercase();
            }
            // Remove .fmpVault.tar.gpg
            if user_input == "y" || user_input == "yes" {
                Command::new("rm")
                    .arg(encrypted_vault_location.as_str()).output().expect("Failed to remove old vault");
            }
            // Exit
            else {
                vault::exit_vault(vault::get_fmp_vault_location());
            }
        }
        // Make .fmpVault folder
        Command::new("mkdir")
            .arg(vault_location.as_str()).output().expect("Failed to make .fmpVault folder");
        println!("Done");
        println!("Creating accounts file...\n");
        // Create accounts file
        Command::new("touch")
            .arg(accounts_loaction.as_str()).output().expect("Failed to make account file");
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

    // If flag -g or --generate-pasword is used
    if opts.flag_g == true {
        let length = import_handle::get_u32_input("How long should the password be? ");
        password::generate_password(length);
        vault::exit_vault(vault::get_fmp_vault_location());
    }

    // If flag -E or --encrypt is used
    if opts.flag_en == true {
        vault::encrypt_and_exit();
    }
    // If no flags are supplied
    vault::decrypt_fmp_vault();
    vault::read_vault();
    vault::delete_vault(vault::get_fmp_vault_location());
}