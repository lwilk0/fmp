use std::fs;
// Gets the location of the accounts file
//
// USAGE
//
// let var: String = get_account_vault_location();
pub fn get_account_location() -> String{
    // Gets users home directory
    let home_dir = dirs::home_dir().expect("Could not find home directory!");
    // Appends directory name to end of home directory
    let account_location = home_dir.join(".fmpVault/accounts");

    return account_location.display().to_string();
}

// Reads account file
//
// USAGE
//
// let account = read_account();
pub fn read_account(account_path: String) -> Vec<String> {
    // Reads acc_path and saves as string to acc
    let  account_string = fs::read_to_string(account_path).expect("Could not read accounts file");
    // Seperates each piece of data through the newline between and saves each word to vector acc
    let mut account: Vec<String> = account_string.split('\n').map(|v| v.to_string()).collect();
    // Removes blank "" from acc
    account.retain(|x| x != "");
    return account;
}

// Writes account vector to account file
//
// USAGE
//
//  write_account(account_path, account);
pub fn write_account(account_path: String, account: &Vec<String>) {
    // Saves vector to accounts file, each piece of data seperated through newline
    fs::write(account_path, account.join("\n")).expect("Could not save accounts file");
}