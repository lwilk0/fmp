use clap::Parser;
use input_handle::get_string_input;
use std::{path::Path, process::{Command, exit}};

mod account; use account::{get_account_location, read_account};
mod vault; use vault::{exit_vault, get_vault_location, decrypt_vault, encrypt_and_exit, print_vault_entries, delete_vault};
mod checks; use checks::{os_check, vault_exists_check};
mod json; use json::{new_json_account, remove_account, change_password, change_username};
mod password; use password::{calculate_entropy, generate_password};

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

    /// Delete account from vault.
    /// used as: -d, --delete
    #[clap(short = 'd', long = "delete")]
    flag_d: bool,

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

    /// Change password for an account.
    /// used as: -p , --change-password 
    #[clap(short = 'p', long = "change-password")]
    flag_p: bool,

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
    // If flag -c or --create is used
    if opts.flag_c == true {
        println!("FMP SETUP\n");
        println!("Creating .vault in home directory...\n");
        // Get user to name vault
        let vault_name = get_string_input("What should the vault be called? ");
        // Format variables
        let vault_create_location = &get_vault_location(&vault_name);
        let encrypted_vault_location = format!("{}/.tar.gz.gpg", vault_create_location);
        let accounts_loaction = format!("{}/accounts", vault_create_location);
        let mut user_input:String = String::new();
        // If encrypted vault exists
        if Path::new(&encrypted_vault_location).exists() {
            // Ask user for input, handles incorect input
            if user_input != "y" && user_input != "yes" && user_input != "no" && user_input != "n" {
                user_input = input_handle::get_string_input("A vault with that name already exists, remove it? y(es), n(o)").to_lowercase();
            }
            // Remove vault
            if user_input == "y" || user_input == "yes" {
                Command::new("rm")
                    .arg(encrypted_vault_location.as_str()).output().expect("Failed to remove old vault");
            }
            // Exit
            else {
                exit_vault(vault_create_location);
            }
        }
        // Make .vault folder
        Command::new("mkdir")
            .arg(&vault_create_location.as_str()).output().expect("Failed to make .vault folder");
        println!("Done");
        println!("Creating accounts file...\n");
        // Create accounts file
        Command::new("touch")
            .arg(accounts_loaction.as_str()).output().expect("Failed to make account file");
        println!("Done\n");
        // Exit
        encrypt_and_exit(vault_create_location);
         
    }

    // Asks user what vault to access
    let mut vault_to_be_accessed = get_string_input("What vault should be accessed? ");
    // Check if vault with that name exists
    let mut vault_exists = vault_exists_check(get_vault_location(&vault_to_be_accessed));
    // If it does not
    while vault_exists == "no" {
        println!("\nNo vault with that name exists, it can be created with fmp -c\n");
        vault_to_be_accessed = get_string_input("What vault should be accessed? ");
        vault_exists = vault_exists_check(get_vault_location(&vault_to_be_accessed));
    }
    println!("");
    let vault_location = get_vault_location(&vault_to_be_accessed);

    // If flag -a or --add is used
    if opts.flag_a == true {
        let mut user_input: String = "y".to_string();
        // Decrypt vault
        decrypt_vault(&vault_location);
        while user_input == "y" || user_input == "yes" {
            let account = read_account(get_account_location(&vault_location));
            // Get user inputs
            let mut name = input_handle::get_string_input("What should the account be named? ");
            let username = input_handle::get_string_input("\nWhat is the account username?");
            let password = input_handle::get_string_input("\nWhat is the account password");
            println!("");
            // Create new account
            let mut error_handle = new_json_account(&vault_location, name.as_str(), username.as_str(), password.as_str(), account.clone());
            // Handle error
            while error_handle != "ok" {
                name = input_handle::get_string_input("Enter new account name: ");
                error_handle = new_json_account(&vault_location, name.as_str(), username.as_str(), password.as_str(), account.clone());
            }
            // Ask user if they would like to add a new account
            user_input = String::new();
            while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
                user_input = input_handle::get_string_input("\nWould you like to enter a new account? (y)es, (n)o").to_lowercase();
                println!("");
            }
        }
        // Exit
        encrypt_and_exit(&vault_location);
    }

    // If flag -d or --delete is used
    if opts.flag_d == true {
        let mut user_input: String = "y".to_string();
        // Decrypt vault
        decrypt_vault(&vault_location);
        while user_input == "y" || user_input == "yes" {
            let account = read_account(get_account_location(&vault_location));
            // Get account name
            let mut name = input_handle::get_string_input("What account should be removed? ");
            println!("");
            // Removes account 
            let mut error_handle = remove_account(&vault_location, name.as_str(), account.clone());
            // Handle error
            while error_handle != "ok" {
                name = input_handle::get_string_input("Enter correct account name: ");
                error_handle = remove_account(&vault_location, name.as_str(), account.clone());
            }
            // Ask user if they would like to remove another account
            user_input = String::new();
            while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
                user_input = input_handle::get_string_input("\nWould you like to remove another account? (y)es, (n)o").to_lowercase();
                println!("");
            }
        }
        // Exit
        encrypt_and_exit(&vault_location);
    }

    // If flag -p or --change-password is used
    if opts.flag_p == true {
        // Decrypt vault
        decrypt_vault(&vault_location);
        // Get user input
        let name = input_handle::get_string_input("What account password should be changed? ");
        let password = input_handle::get_string_input("\nWhat should the password be changed to?");
        // Changes password
        change_password(&vault_location, password.as_str(), &name);
        // Exit
        encrypt_and_exit(&vault_location);
    }

    // If flag -u or --change-username is used
    if opts.flag_u == true {
        // Decrypt vault
        decrypt_vault(&vault_location);
        // Get user input
        let name = input_handle::get_string_input("What account username should be changed? ");
        let username = input_handle::get_string_input("\nWhat should the username be changed to?");
        // Change username
        change_username(&vault_location, &username.as_str(), &name);
        // Exit
        encrypt_and_exit(&vault_location);
    }

    // If flag -e or --entropy is used
    if opts.flag_e == true {
        // Get password to rate
        let password: String = input_handle::get_string_input("Enter the password for entropy calculation");
        // Calculate entropy
        let entropy_tuple: (f64, &str) = calculate_entropy(&password);
        println!("The password has {:.2} bit entropy, giving it a rating of {}", entropy_tuple.0, entropy_tuple.1);
        exit_vault(&vault_location);
    }

    // If flag -g or --generate-pasword is used
    if opts.flag_g == true {
        // Gets wanted length from user
        let length = input_handle::get_u32_input("How long should the password be? ");
        // Generate password of that length
        let generated_password = generate_password(length);
        let mut user_input: String = String::new();
        // Ask user if they want to save password to account
        while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
            user_input = get_string_input("Would you like to save this password to an account? (y)es, (n)o").to_lowercase();
        }
        // If they do
        if user_input == "y" || user_input == "yes" {
            // Get user inputs
            let account = read_account(get_account_location(&vault_location));
            let name = get_string_input("What should the account be named? ");
            let username = get_string_input("\nWhat is the account username?");
            // Create new account
            new_json_account(&vault_location, name.as_str(), username.as_str(), &generated_password, account);
            // Exit
            encrypt_and_exit(&vault_location);
        }
        // Exit
        exit_vault(&vault_location);
    }

    // If flag -E or --encrypt is used
    if opts.flag_en == true {
        encrypt_and_exit(&vault_location);
    }

    // If flag -b or --backup is used
    if opts.flag_b == true {
        let vault_location_as_encrypted_tar = format!("{}.tar.gz.gpg", vault_location);
        let vault_location_as_backup = format!("{}.bk", vault_location_as_encrypted_tar);
        // Ask user if they want to create or install backup
        let mut user_input: String = String::new();
        // Input validation
        if user_input != "b" && user_input != "backup" && user_input != "i" && user_input != "install" {
            user_input = input_handle::get_string_input("Would you like to create a backup or install a backup? (b)ackup, (i)nstall");
        }
        // If user wants to backup
        if user_input == "b" || user_input == "backup" {
            // Check that encrypted vault exists
            if Path::new(&vault_location_as_encrypted_tar).exists() == false {
                // If it does not exist
                println!("No vault found in home directory. Has it been created?");
                exit_vault(&vault_location);
            }
            // Backup
            Command::new("cp")
                .args([vault_location_as_encrypted_tar.as_str(), vault_location_as_backup.as_str()]).output().expect("Could not create backup");
            println!("\nSuccessfully backed up vault");
        }
        // If user wants to install backup
        else {
            // Check if backup exists
            if Path::new(&vault_location_as_backup).exists() == false {
                // If it does not
                println!("No backup file found in home directory. Has it been created?");
                exit_vault(&vault_location);
            }
            // Install backup
            Command::new("cp")
                .args([vault_location_as_backup.as_str(), vault_location_as_encrypted_tar.as_str()]).output().expect("Could not install backup");
            println!("\nSuccessfully installed backup");
        }
        exit(1);
    }
    // If no flags are supplied
    decrypt_vault(&vault_location);
    print_vault_entries(&vault_location);
    delete_vault(&vault_location);
}