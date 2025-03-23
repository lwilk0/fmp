use std::{env, process::exit, path::Path};

// Checks the os currently in use
//
// USAGE
//
// os_check();
pub fn os_check() {
    // If current OS is not linux
    if env::consts::OS != "linux" {
        println!("Sorry, fmp currently only supports Linux.");
        exit(1);
    }
}

// Checks if the vault exists
//
// USAGE
//
//let mut var = vault_exists_check(get_vault_location(&vault_name));
//    while var == "no" {
//        // Vault with name vault_name does not exist
//    }
pub fn vault_exists_check(vault_name: String) -> String{
    // Encrypted vault location
    let directory = format!("{}.tar.gz.gpg", vault_name);
    // If it does not exist
    if Path::new(&directory).exists() == false {
        return "no".to_string();
    }
    
    return "yes".to_string();
}