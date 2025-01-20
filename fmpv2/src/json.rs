use serde_json;
use serde::Deserialize;
use std::{path::Path, fs};

#[derive(Deserialize, Debug)]
pub struct UserPass {
    pub username: String,
    pub password: String,
}

// Reads json file from username and vault file location
//
// USAGE
//
// let var: UserPass = read_json(get_fmp_vault_location(), "account");
// if var.username == "err" {
//    // username is incorect, handle accordingly
//}
pub fn read_json(fmp_vault_location: String, account: &str) -> UserPass{
    // Find where wanted json is located
    let json_directory = format!("{}/{}/data.json", fmp_vault_location, account);
    // For error handling
    let error: UserPass = UserPass {username: "err".to_string(), password: "err".to_string(),};
    
    // If directory exists
    if Path::new(&json_directory).exists() {
        // Read json file to string
        let json_as_string: String = fs::read_to_string(&json_directory).unwrap();
        // Convert to json in UserPass structure
        let json: UserPass = serde_json::from_str(&json_as_string).unwrap();
        return json;
    }
    println!("Invalid Input, username does not exist!");
    return error;
}
