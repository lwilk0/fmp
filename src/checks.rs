use std::{env, process::exit, path::Path};

// Checks the os currently in use
//
// USAGE
//
// os_check();
pub fn os_check() {
    if env::consts::OS != "linux" {
        println!("Sorry, fmp currently only supports Linux.");
        exit(1);
    }
}

// Checks if the vault exists
//
// USAGE
//
// vault_exists_check(get_fmp_vault_location());
pub fn vault_exists_check(fmp_vault_location: String) {
    let directory = format!("{}.tar.gz.gpg", fmp_vault_location);
    if Path::new(&directory).exists() == false {
        println!("The fmp vault does not exist in the current users home directory! It can be created with fmp -c");
        exit(1);
    }
}