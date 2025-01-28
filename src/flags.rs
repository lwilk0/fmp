use input_handle::{get_string_input, get_u32_input};
use std::{path::Path, process::{Command, exit}};
use crate::{account::{get_account_location, read_account}, json::{change_password, change_username, new_json_account, read_json, remove_account, UserPass}, password::{calculate_entropy, generate_password}, vault::{decrypt_vault, delete_vault, delete_vault_full, encrypt_and_exit, encrypt_vault, exit_vault, get_vault_location, print_vault_entries}};

pub fn create() {
    println!("FMP SETUP\n");
    println!("Creating .vault in home directory...\n");
    // Get user to name vault
    let vault_name = get_string_input("What should the vault be called? ");
    // Format variables
    let vault_create_location = &get_vault_location(&vault_name);
    let encrypted_vault_location = format!("{}.tar.gz.gpg", vault_create_location);
    let accounts_location = format!("{}/accounts", vault_create_location);
    let mut user_input:String = String::new();
    // If encrypted vault exists
    if Path::new(&encrypted_vault_location).exists() {
        // Ask user for input, handles incorrect input
        if user_input != "y" && user_input != "yes" && user_input != "no" && user_input != "n" {
            user_input = input_handle::get_string_input("\nA vault with that name already exists, remove it? y(es), n(o)").to_lowercase();
        };
        // Remove vault
        if user_input == "y" || user_input == "yes" {
            println!("\nDecrypt the vault to remove it...\n");
            delete_vault_full(&vault_create_location, &encrypted_vault_location);
        }
        // Exit
        else {
            exit_vault(vault_create_location);
        }
    }
    // Make .vault folder
    Command::new("mkdir")
        .arg(&vault_create_location.as_str()).output().expect("Failed to make .vault folder");
    println!("\nDone");
    println!("\nCreating accounts file...");
    // Create accounts file
    Command::new("touch")
        .arg(accounts_location.as_str()).output().expect("Failed to make account file");
    println!("\nDone\n");
    // Exit
    encrypt_and_exit(vault_create_location);
}


pub fn add(vault: &String) {
    let mut user_input: String = "y".to_string();
        // Decrypt vault
        decrypt_vault(vault);
        while user_input == "y" || user_input == "yes" {
            let account = read_account(get_account_location(vault));
            // Get user inputs
            let mut name = input_handle::get_string_input("What should the account be named? ");
            let username = input_handle::get_string_input("\nWhat is the account username?");
            let password = input_handle::get_string_input("\nWhat is the account password");
            println!("");
            // Create new account
            let mut error_handle = new_json_account(vault, name.as_str(), username.as_str(), password.as_str(), account.clone());
            // Handle error
            while error_handle != "ok" {
                name = input_handle::get_string_input("Enter new account name: ");
                error_handle = new_json_account(vault, name.as_str(), username.as_str(), password.as_str(), account.clone());
            }
            // Ask user if they would like to add a new account
            user_input = String::new();
            while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
                user_input = input_handle::get_string_input("\nWould you like to enter a new account? (y)es, (n)o").to_lowercase();
                println!("");
            }
        }
        // Exit
        encrypt_and_exit(vault);
}

pub fn delete(vault: &String) {
    let mut user_input: String = "y".to_string();
        // Decrypt vault
        decrypt_vault(vault);
        while user_input == "y" || user_input == "yes" {
            let account = read_account(get_account_location(vault));
            // Get account name
            let mut name = input_handle::get_string_input("What account should be removed? ");
            println!("");
            // Removes account 
            let mut error_handle = remove_account(vault, name.as_str(), account.clone());
            // Handle error
            while error_handle != "ok" {
                name = input_handle::get_string_input("Enter correct account name: ");
                error_handle = remove_account(vault, name.as_str(), account.clone());
            }
            // Ask user if they would like to remove another account
            user_input = String::new();
            while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
                user_input = input_handle::get_string_input("\nWould you like to remove another account? (y)es, (n)o").to_lowercase();
                println!("");
            }
        }
        // Exit
        encrypt_and_exit(vault);
}

pub fn change_account_password(vault: &String) {
    // Decrypt vault
    decrypt_vault(vault);
    // Get user input
    let name = input_handle::get_string_input("What account password should be changed? ");
    let password = input_handle::get_string_input("\nWhat should the password be changed to?");
    // Changes password
    change_password(vault, password.as_str(), &name);
    // Exit
    encrypt_and_exit(vault);
}

pub fn change_account_username(vault: &String) {
    // Decrypt vault
    decrypt_vault(vault);
    // Get user input
    let name = input_handle::get_string_input("What account username should be changed? ");
    let username = input_handle::get_string_input("\nWhat should the username be changed to?");
    // Change username
    change_username(vault, &username.as_str(), &name);
    // Exit
    encrypt_and_exit(vault);
}

pub fn entropy(vault: String) {
    let password: String;
    // Ask user if they want to enter a password or use an already existing one
    let mut user_input = String::new();
    while user_input != "e" && user_input != "enter" && user_input != "a" && user_input != "account" {
        user_input = get_string_input("Would you like to enter a password or use one linked to an account? (e)nter, (a)ccount");
        println!("");

    }
    if user_input == "e" || user_input == "enter" {
        // Get password to rate
        password = get_string_input("Enter the password for entropy calculation");
    }
    else {
        decrypt_vault(&vault);
        let mut account = get_string_input("What is the account for the password you want to rate?");
        println!("");
        let mut json: UserPass = read_json(vault.clone(), account);
        while json.username == "err" {
            println!("");
            account = get_string_input("What is the account for the password you want to rate?");
            json = read_json(vault.clone(), account);
            println!("");
        }
        password = json.password;
    }
    // Calculate entropy
    let entropy_tuple: (f64, &str) = calculate_entropy(&password);
    println!("The password has {:.2} bits of entropy, giving it a rating of {}\n", entropy_tuple.0, entropy_tuple.1.to_lowercase());
    exit_vault(&vault);
}

pub fn gen_password(vault: &String) {
    // Gets wanted length from user
    let length = get_u32_input("How long should the password be? ");
    // Generate password of that length
    let generated_password = generate_password(length);
    // Prints generated password
    println!("\n{}", generated_password);
    // Calculate entropy
    let entropy_tuple: (f64, &str) = calculate_entropy(&generated_password);
    println!("The password has {:.2} bits of entropy, giving it a rating of {}.\n", entropy_tuple.0, entropy_tuple.1.to_lowercase());
    // Ask user if they want to save password to account
    let mut user_input: String = String::new();
    while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
        user_input = get_string_input("Would you like to save this password to an account? (y)es, (n)o").to_lowercase();
    }
    // If they do
    if user_input == "y" || user_input == "yes" {
        decrypt_vault(vault);
        // Get user inputs
        let account = read_account(get_account_location(vault));
        let name = get_string_input("What should the account be named? ");
        let username = get_string_input("\nWhat is the account username?");
        // Create new account
        new_json_account(vault, name.as_str(), username.as_str(), &generated_password, account);
        // Exit
        encrypt_and_exit(vault);
    }
    // Exit
    exit_vault(vault);
}

pub fn backup(vault: &String) {
    let vault_location_as_encrypted_tar = format!("{}.tar.gz.gpg", vault);
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
                exit_vault(vault);
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
                exit_vault(vault);
            }
            // Install backup
            Command::new("cp")
                .args([vault_location_as_backup.as_str(), vault_location_as_encrypted_tar.as_str()]).output().expect("Could not install backup");
            println!("\nSuccessfully installed backup");
        }
        exit(1);
}

pub fn decrypt_vault_all_files(vault: &String) {
    let vault_encrypted = format!("{}.tar.gz.gpg", vault);
    // Delete all files related to the vault
    delete_vault_full(vault, &vault_encrypted);
}

pub fn rename(vault: &String) {
    let mut new_name = get_string_input("What would you like to rename the vault to? ");
    // Format variables
    let mut vault_new_directory = get_vault_location(&new_name);
    let mut vault_new_directory_encrypted = format!("{}.tar.gz.gpg", vault_new_directory);
    let vault_old_encrypted = format!("{}.tar.gz.gpg", vault);
    let vault_old_encrypted_backup = format!("{}.bk", vault_old_encrypted);
    // If vault with that name already exists
    while Path::new(&vault_new_directory_encrypted).exists() {
        // Ask user what to do with it
        let mut user_input = get_string_input("Vault already exists, would you like to remove it? y(es), n(o), e(xit)");
        // Input validation
        while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" && user_input != "e" && user_input != "exit"{
            println!("\nInvalid input, please try again");
            user_input = get_string_input("Vault already exists, would you like to remove it? y(es), n(o), e(xit)");
        }
        // If user wants to remove vault
        if user_input == "y" || user_input == "yes" {
            delete_vault_full(&vault_new_directory, &vault_new_directory_encrypted);
        }
        // If user does not want to remove vault
        else if user_input == "n" || user_input == "no" {
            println!("Enter new name:");
            new_name = get_string_input("What would you like to rename the vault to? ");
            vault_new_directory = get_vault_location(&new_name);
            vault_new_directory_encrypted = format!("{}.tar.gz.gpg", vault_new_directory);
        }
        // Exit
        else {
            exit(1);
        }
    }
    // Decrypt old vault
    decrypt_vault(vault);
    // Rename folder
    Command::new("mv") 
        .args([vault.to_string(), vault_new_directory.to_string()]).output().expect("Could not rename vault");
    // Remove old encrypted file
    Command::new("rm") 
        .arg(vault_old_encrypted.as_str()).output().expect("Could not remove old encrypted vault");
    // If old encrypted file backup exists
    println!("{}", vault_old_encrypted_backup);
    if Path::new(&vault_old_encrypted_backup).exists() {
        // Remove old encrypted file backup
        Command::new("rm") 
            .arg(vault_old_encrypted_backup.as_str()).output().expect("Could not remove old encrypted vault");
    }
    // Exit
    encrypt_and_exit(&vault_new_directory);
}

pub fn change_vault_password(vault: &String) {
    decrypt_vault(vault);
    println!("\nEnter new password:\n");
    encrypt_vault(vault);
    exit_vault(vault);
}

pub fn no_flags(vault: &String) {
    decrypt_vault(vault);
    print_vault_entries(vault);
    delete_vault(vault);
}