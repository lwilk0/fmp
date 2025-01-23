use dirs;
use cmd_lib::run_cmd;
use std::{path::Path, process::exit};
use super::account;
use super::json;

// Finds where fmp's vault is
//
// USAGE
//
// let var: String = get_fmp_vault_location();
pub fn get_fmp_vault_location() -> String{
    // Gets users home directory
    let home_dir = dirs::home_dir().expect("Could not find home directory!");
    // Appends directory name to end of home directory
    let fmp_vault_location = home_dir.join(".fmpVault");

    return fmp_vault_location.display().to_string();
}

// Encrypts the .fmpVault file to .fmpVault.tar.gz.gpg
//
// USAGE
//
// encrypt_fmp_vault();
pub fn encrypt_fmp_vault() {
    // Gets locations
    let fmp_vault_location = get_fmp_vault_location();
    let fmp_vault_as_encrypted_tar = format!("{}.tar.gz.gpg", fmp_vault_location);

    println!("Encrypting fmp vault...\n");
    if Path::new(&fmp_vault_as_encrypted_tar).exists() {
        run_cmd!(rm $fmp_vault_as_encrypted_tar).expect("Could not remove encrypted file");
    }
    // Encrypts .fmpVault
    run_cmd!(tar -cz $fmp_vault_location -f $fmp_vault_location.tar.gz).expect("Failed to execute command");
    run_cmd!(gpg -c --no-symkey-cache $fmp_vault_location.tar.gz).expect("err");
    run_cmd!(rm -r $fmp_vault_location).expect("err");
    run_cmd!(rm -r $fmp_vault_location.tar.gz).expect("err");

    println!("\nEncrypted!")
}

// Decrypts the .fmpVault.tar.gz.gpg file to .fmpVault
//
// USAGE
//
// decrypt_fmp_vault();
pub fn decrypt_fmp_vault() {

    let home_dir = dirs::home_dir().expect("Could not find home directory!");
    let fmp_vault_location = get_fmp_vault_location();
    let fmp_vault_as_encrypted_tar = format!("{}.tar.gz.gpg", fmp_vault_location);

    println!("Decrypting fmp vault...\n");

    run_cmd!(gpg --no-symkey-cache $fmp_vault_as_encrypted_tar).expect("Failed to execute command");
    run_cmd!(tar -xf $fmp_vault_location.tar.gz -C $home_dir).expect("Failed to execute command");
    run_cmd!(rm $fmp_vault_location.tar.gz);
    println!("\nDecrypted");
}

// Reads all json files and prints to screen
//
// USAGE
//
// read_vault() 
pub fn read_vault() {
    // Gets list of accounts
    let accounts_list: Vec<String> = account::read_account(account::get_account_location());
        // Loop for each entry in accounts_list
        for i in 0..accounts_list.len() {
            // Find corrosponding json file and read
            let service = accounts_list[i].clone();
            let json = json::read_json(get_fmp_vault_location(), service);
            // Output
            println!("{}: Username: {} Password: {}", accounts_list[i], json.username, json.password)
        }
}

// Removes the vault folder
//
// USAGE
//
// delete_vault(get_fmp_vault_location())
pub fn delete_vault(fmp_vault_location: String) {
    if Path::new(&fmp_vault_location).exists() {
        run_cmd!(rm -r $fmp_vault_location);
    }
}

// Exits the vault and program
//
// USAGE
//
// exit(get_fmp_vault_location())
pub fn exit_vault(fmp_vault_location: String) {
    delete_vault(fmp_vault_location);
    exit(1);
}