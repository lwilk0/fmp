use dirs;
use std::{path::Path, process::{exit, Command}};
use prettytable::{Table, row};
use input_handle::get_string_input;
use crate::{account::{read_account, get_account_location}, json::read_json, checks::vault_exists_check};

// Gets vault from user, should only be called once at main
//
// USAGE
//
// let vault_location = vault_to_access();
pub fn vault_to_access()  -> String{
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
    return vault_location; 
}

// Finds where vault is
//
// USAGE
//
// let var: String = get_vault_location(&vault);
pub fn get_vault_location(vault: &String) -> String{
    // Gets users home directory
    let home_dir = dirs::home_dir().expect("Could not find home directory!");
    // Appends directory name to end of home directory
    let vault = home_dir.join(format!(".{}", vault));

    return vault.display().to_string();
}

// Encrypts the .vault file to .vault.tar.gz.gpg
//
// USAGE
//
// encrypt_vault(&vault_location);
pub fn encrypt_vault(vault: &String) {
    // Gets locations
    let vault = vault;
    let vault_as_encrypted_tar = format!("{}.tar.gz.gpg", vault);
    let vault_as_tar = format!("{}.tar.gz", vault);
    
    println!("Encrypting vault...\n");
    if Path::new(&vault_as_encrypted_tar).exists() {
        Command::new("rm")
            .arg(vault_as_encrypted_tar.as_str()).output().expect("Could not remove encrypted file");
    }
    // Turns .vault into tarball
    Command::new("tar")
        .args(["-czf", vault_as_tar.as_str(), vault.as_str()]).output().expect("Failed to execute command");
    // Encrypts vault, handles incorect password
    Command::new("gpg")
        .args(["-c", "--no-symkey-cache", vault_as_tar.as_str()]).output().expect("Could not encrypt vault, please run fmp -E to encrypt");
    // If pasword is incorrect, tarball is removed
    while Path::new(&vault_as_encrypted_tar).exists() == false {
        Command::new("rm")
            .arg(vault_as_tar.as_str()).output().expect("Could not remove file");
        exit(1);
    }
    // Cleanup
    Command::new("rm")
        .args(["-r", vault_as_tar.as_str()]).output().expect("Could not remove file");
    Command::new("rm")
        .args(["-r", vault.as_str()]).output().expect("Could not remove file");
    println!("\nEncrypted!");
}

// Decrypts the .vault.tar.gz.gpg file to .vault
//
// USAGE
//
// decrypt_vault(&vault_location);
pub fn decrypt_vault(vault: &String){
    let vault = vault;
    let vault_as_encrypted_tar = format!("{}.tar.gz.gpg", vault);
    let vault_as_tar = format!("{}.tar.gz", vault);
    println!("Decrypting vault...\n");

    // Decrypts vault, handles incorrect password
    Command::new("gpg")
        .args(["-q", "--no-symkey-cache", vault_as_encrypted_tar.as_str()]).output().expect("Could not encrypt vault");
    // If file has not been decrypted
    if Path::new(&vault_as_tar).exists() == false{
        println!("Bad decrypt!");
        exit_vault(vault);
    }
    // Decrypts tarball
    Command::new("tar")
        .args(["-xf", vault_as_tar.as_str(), "-C", "/"]).output().expect("Failed to execute command");
    // Removes tarball
    Command::new("rm")
        .arg(vault_as_tar.as_str()).output().expect("Could not remove tarball vault");
    println!("Decrypted\n");
 
}

// Reads all json files and prints to screen
//
// USAGE
//
// print_vault_entries(&vault_location) 
pub fn print_vault_entries(vault: &String) {
    // Gets list of accounts
    let accounts_list: Vec<String> = read_account(get_account_location(&vault));
        // Loop for each entry in accounts_list
        if accounts_list.len() == 0 {
            println!("No accounts have been created! Use fmp -a to create an account.");
            return;
        }
        // Create the table
        let mut table = Table::new();
        table.add_row(row!["Account", "Username", "Password"]);
        for i in 0..accounts_list.len() {
            // Find corrosponding json file and read
            let account = accounts_list[i].clone();
            let json = read_json(vault.clone(), account);
            // Output
            table.add_row(row![accounts_list[i], json.username, json.password]);
        }
        table.printstd();
}

// Removes the vault folder
//
// USAGE
//
// delete_vault(&vault_location)
pub fn delete_vault(vault: &String) {
    if Path::new(&vault).exists() {
        Command::new("rm")
            .args(["-r", vault.as_str()]).output().expect("Could not remove .vault");
    }
}

// Exits the vault and program
//
// USAGE
//
// exit(&vault_location)
pub fn exit_vault(vault: &String) {
    delete_vault(vault);
    exit(1);
}

// Encrypts the vault any tidy's files up
//
// USAGE
//
// encrtpt_and_exit(&vault_location);
pub fn encrypt_and_exit(vault: &String) {
    encrypt_vault(vault);
    delete_vault(vault);
    exit_vault(vault);
}

// Removes all files related to a vault
//
// USAGE
//
// delete_vault_full(&vault_location, &vault_location_encrypted)
pub fn delete_vault_full(vault: &String, vault_encrypted: &String) {
    // Decrypts vault
    decrypt_vault(&vault);
    // Remove all vault files
    Command::new("rm")
        .arg(vault_encrypted.as_str()).output().expect("Failed to remove old vault");
    Command::new("rm")
        .args(["-r", vault.as_str()]).output().expect("Failed to remove old vault");
}