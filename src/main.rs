use clap::Parser;
use import_handle;
use std::{path::Path, process::{Command, exit}};

mod account; use account::{get_account_location, read_account};
mod vault; use vault::{exit_vault, get_fmp_vault_location, decrypt_fmp_vault, encrypt_and_exit, read_vault, delete_vault};
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
    os_check();
    vault_exists_check(get_fmp_vault_location());
    // Stores flag user input bools
    let opts = Options::parse();

    // If flag -a or --add is used
    if opts.flag_a == true {
        let mut user_input: String = "y".to_string();
        // Decrypt vault
        decrypt_fmp_vault();
        while user_input == "y" || user_input == "yes" {
            let account = read_account(get_account_location());
            // Get user inputs
            let mut name = import_handle::get_string_input("What should the account be named? ");
            let username = import_handle::get_string_input("\nWhat is the account username?");
            let password = import_handle::get_string_input("\nWhat is the account password");
            println!("");
            // Create new account
            let mut error_handle = new_json_account(get_fmp_vault_location(), name.as_str(), username.as_str(), password.as_str(), account.clone());
            // Handle error
            while error_handle != "ok" {
                name = import_handle::get_string_input("Enter new account name: ");
                error_handle = new_json_account(get_fmp_vault_location(), name.as_str(), username.as_str(), password.as_str(), account.clone());
            }
            // Ask user if they would like to add a new account
            user_input = String::new();
            while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
                user_input = import_handle::get_string_input("\nWould you like to enter a new account? (y)es, (n)o").to_lowercase();
                println!("");
            }
        }
        // Exit
        encrypt_and_exit();
    }

    // If flag -d or --delete is used
    if opts.flag_d == true {
        let mut user_input: String = "y".to_string();
        // Decrypt vault
        decrypt_fmp_vault();
        while user_input == "y" || user_input == "yes" {
            let account = read_account(get_account_location());
            // Get account name
            let mut name = import_handle::get_string_input("What account should be removed? ");
            println!("");
            // Removes account 
            let mut error_handle = remove_account(get_fmp_vault_location(), name.as_str(), account.clone());
            // Handle error
            while error_handle != "ok" {
                name = import_handle::get_string_input("Enter correct account name: ");
                error_handle = remove_account(get_fmp_vault_location(), name.as_str(), account.clone());
            }
            // Ask user if they would like to remove another account
            user_input = String::new();
            while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
                user_input = import_handle::get_string_input("\nWould you like to remove another account? (y)es, (n)o").to_lowercase();
                println!("");
            }
        }
        // Exit
        encrypt_and_exit();
    }

    // If flag -p or --change-password is used
    if opts.flag_p == true {
        // Decrypt vault
        decrypt_fmp_vault();
        // Get user input
        let name = import_handle::get_string_input("What account password should be changed? ");
        let password = import_handle::get_string_input("\nWhat should the password be changed to?");
        // Changes password
        change_password(get_fmp_vault_location(), password.as_str(), &name);
        // Exit
        encrypt_and_exit();
    }

    // If flag -u or --change-username is used
    if opts.flag_u == true {
        // Decrypt vault
        decrypt_fmp_vault();
        // Get user input
        let name = import_handle::get_string_input("What account username should be changed? ");
        let username = import_handle::get_string_input("\nWhat should the username be changed to?");
        // Change username
        change_username(get_fmp_vault_location(), &username.as_str(), &name);
        // Exit
        encrypt_and_exit();
    }

    // If flag -c or --create is used
    if opts.flag_c == true {
        println!("FMP SETUP\n");
        println!("Creating .fmpVault in home directory...\n");
        let vault_location = get_fmp_vault_location();
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
                exit_vault(get_fmp_vault_location());
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
        encrypt_and_exit();
         
    }

    // If flag -e or --entropy is used
    if opts.flag_e == true {
        let password: String = import_handle::get_string_input("Enter the password for entropy calculation");
        calculate_entropy(password);
        exit_vault(get_fmp_vault_location());
    }

    // If flag -g or --generate-pasword is used
    if opts.flag_g == true {
        let length = import_handle::get_u32_input("How long should the password be? ");
        generate_password(length);
        exit_vault(get_fmp_vault_location());
    }

    // If flag -E or --encrypt is used
    if opts.flag_en == true {
        encrypt_and_exit();
    }

    // If flag -b or --backup is used
    if opts.flag_b == true {
        let fmp_vault_location_as_encrypted_tar = format!("{}.tar.gz.gpg", get_fmp_vault_location());
        let fmp_vault_location_as_backup = format!("{}.bk", fmp_vault_location_as_encrypted_tar);
        let mut user_input: String = String::new();
        if user_input != "b" && user_input != "backup" && user_input != "i" && user_input != "install" {
            user_input = import_handle::get_string_input("Would you like to create a backup or install a backup? (b)ackup, (i)nstall");
        }

        if user_input == "b" || user_input == "backup" {
            if Path::new(&fmp_vault_location_as_encrypted_tar).exists() == false {
                println!("No vault found in home directory. Has it been created?");
                exit_vault(fmp_vault_location_as_encrypted_tar.clone());
            }
            Command::new("cp")
                .args([fmp_vault_location_as_encrypted_tar.as_str(), fmp_vault_location_as_backup.as_str()]).output().expect("Could not create backup");
            println!("\nSuccessfully backed up vault");
        }
        else {
            if Path::new(&fmp_vault_location_as_backup).exists() == false {
                println!("No backup file found in home directory. Has it been created?");
                exit_vault(fmp_vault_location_as_encrypted_tar.clone());
            }
            Command::new("cp")
                .args([fmp_vault_location_as_backup.as_str(), fmp_vault_location_as_encrypted_tar.as_str()]).output().expect("Could not install backup");
            println!("\nSuccessfully installed backup");
        }
        exit(1);
    }
    // If no flags are supplied
    decrypt_fmp_vault();
    read_vault();
    delete_vault(get_fmp_vault_location());
}