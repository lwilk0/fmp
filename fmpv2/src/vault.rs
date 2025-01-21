use dirs;
use cmd_lib::run_cmd;


// Finds where fmp's vault is
//
// USAGE
//
// let var: String = get_fmp_vault_location;
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

    // Encrypts .fmpVault
    run_cmd!(tar -cz $fmp_vault_location | gpg -c -o $fmp_vault_as_encrypted_tar).expect("Failed to execute command");

    println!("\nEncrypted!")
}

// Decrypts the .fmpVault.tar.gz.gpg file to .fmpVault
//
// USAGE
//
// decrypt_fmp_vault();
pub fn decrypt_fmp_vault() {

    let fmp_vault_location = get_fmp_vault_location();
    let fmp_vault_as_encrypted_tar = format!("{}.tar.gz.gpg", fmp_vault_location);

    println!("Decrypting fmp vault...\n");

    run_cmd!(gpg -d $fmp_vault_as_encrypted_tar | tar xz).expect("Failed to execute command");

    println!("\nDecrypted");
}
