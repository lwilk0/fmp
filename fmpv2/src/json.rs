use serde_json;
use serde::Deserialize;
use std::{fs::{self, File}, io::Write, path::Path, process::exit};
use import_handle::get_string_input;
use cmd_lib::run_cmd;

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
// while var.username == "err" {
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

// Reads json file from username and vault file location
//
// USAGE
//
// let var: UserPass = read_json(get_fmp_vault_location(), "account");
// while var == "err" {
//    // Get new username from user and try again
//}
pub fn new_json_account(fmp_vault_location: String, name: &str, username: &str, password: &str) -> String {
    // What file data.json will end up in
    let new_account_dir = format!("{}/{}", fmp_vault_location, name);
    // data.json location
    let new_account_file = format!("{}/data.json", new_account_dir);
    // For user input
    let mut user_input: String = String::new();
    println!("Creating Account...");
    // Handles account already existing
    if Path::new(&new_account_dir).exists() {
        // Gets user input
        while user_input != "e" && user_input != "exit" && user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
            user_input = get_string_input("An Account with that name already exists, would you like to remove it? (y)es, (n)o, (e)xit").to_lowercase();
        }

        // if input is y or yes, the directory will be removed
        if user_input == "y" || user_input == "yes" {
            run_cmd!(rm -r $new_account_dir).expect("Failed to delete directory");
            println!("\nAccount Removed!")
        }
        // Return with error to handle
        else if user_input == "n" || user_input == "no" {
            return "err".to_string();
        }
        // If input is e or exit, the program is exited
        else {
            println!("Exiting...");
            exit(1)
        }
        
    }
    // Creates new account directory and data.json file containing "{}"
    run_cmd!(mkdir $new_account_dir).expect("Failed to create directory");
    run_cmd!(echo "{}" > $new_account_file).expect("Failed to create directory");
    
    // Loads data.json file
    let mut json: serde_json::Value = load_json(&new_account_file);
    // Add data to json
    json["username"] = serde_json::Value::String(username.to_owned());
    json["password"] = serde_json::Value::String(password.to_owned());
    save_json_file(new_account_file, json);
    println!("\nSucessfully saved new account");
    return "ok".to_string();
}

// Saves json data to json file
//
// USAGE
//
// save_json_file(json_file_loaction, json)
pub fn save_json_file(json_file_directory: String, json: serde_json::Value) {
    // Saves json to string
    let json_to_write = serde_json::to_string(&json).unwrap();
    // Opens data.json
    let mut file = File::create(json_file_directory).expect("Could not load file");
    // Add data to data.json
    file.write_all(json_to_write.as_bytes()).expect("Could not write to file");
}

// Loads json file and returns as Value
//
// USAGE
//
// var = load_json(json_file_directory)
pub fn load_json(json_file_directory: &String) -> serde_json::Value {
    // Read json file to string
    let json_as_string: String = fs::read_to_string(&json_file_directory).unwrap();
    // Convert to json in UserPass structure
    let json: serde_json::Value = serde_json::from_str(&json_as_string).unwrap();
    return json;
}