use serde_json;aa
use serde::Deserialize;
use std::{fs::{self, File}, io::Write, path::Path, process::Command};
use input_handle::get_string_input;

use crate::{vault::exit_vault, account::{write_account, get_account_location}};

#[derive(Deserialize, Debug)]
pub struct UserPass {
    pub username: String,
    pub password: String,
}

// Reads json file from username and vault file location
//
// USAGE
//
// let var: UserPass = read_json(get_vault_location(), "account");
// while var.username != "ok" {
//    // username is incorrect, handle accordingly
//}
pub fn read_json(vault: String, account: String) -> UserPass{
    // Find where wanted json is located
    let json_file_directory = format!("{}/{}/data.json", vault, account);
    // For error handling
    let error: UserPass = UserPass {username: "err".to_string(), password: "err".to_string(),};
    
    // If directory exists
    if Path::new(&json_file_directory).exists() {
        let json: UserPass = load_json_as_userpass(&json_file_directory);
        return json;
    }
    println!("Invalid Input, username does not exist!");
    return error;
}

// Creates new account
//
// USAGE
//
// let var: UserPass = new_json_account(&vault_location, "name", "username", "password");
// while var != "ok" {
//    // Get new username from user and try again
//}
pub fn new_json_account(vault: &String, name: &str, username: &str, password: &str, mut account: Vec<String>) -> String {
    // What file data.json will end up in
    let new_account_dir = format!("{}/{}", vault, name);
    // data.json location
    let new_account_file = format!("{}/data.json", new_account_dir);
    // For user input
    let mut user_input: String = String::new();
    println!("Creating account...");
    // Handles account already existing
    if Path::new(&new_account_dir).exists() {
        // Gets user input
        while user_input != "e" && user_input != "exit" && user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
            user_input = get_string_input("An Account with that name already exists, would you like to remove it? (y)es, (n)o, (e)xit").to_lowercase();
        }

        // if input is y or yes, the directory will be removed
        if user_input == "y" || user_input == "yes" {
            Command::new("rm")
                .args(["-r", new_account_dir.as_str()]).output().expect("Failed to delete directory");
        }
        // Return with error to handle
        else if user_input == "n" || user_input == "no" {
            return "err".to_string();
        }
        // If input is e or exit, the program is exited
        else {
            println!("Exiting...");
            exit_vault(vault);
        }
        
    }
    // Creates new account directory and data.json file containing "{}"
    Command::new("mkdir")
        .arg(new_account_dir.as_str()).output().expect("Failed to create directory");
    Command::new("touch")
        .arg(new_account_file.as_str()).output().expect("Failed to create data.json file");
    fs::write(&new_account_file, "{}").expect("Could not save to data.json file");

    // Loads data.json file
    let mut json: serde_json::Value = load_json_as_value(&new_account_file);
    // Add data to json
    json = add_fields_to_json(json, username, password);
    save_json_file(new_account_file, json);
    // Add data to accounts
    account.push(String::from(name));
    write_account(get_account_location(&vault), &account);
    println!("\nSucessfully saved new account");
    return "ok".to_string();
}

// Saves json data to json file
//
// USAGE
//
// let var save_json_file(json_file_location, json)
// while var != "ok" {
//    // Get new account from user and try again
//}
pub fn save_json_file(json_file_directory: String, json: serde_json::Value) {
    // Saves json to string
    let json_to_write = serde_json::to_string(&json).unwrap();
    // Opens data.json
    let mut file = File::create(json_file_directory).expect("Could not load file");
    // Add data to data.json
    file.write_all(json_to_write.as_bytes()).expect("Could not write to file");
}

// Remove account from .vault
//
// USAGE
//
// let var remove_account(&vault_location, "name", account)
// while var != "ok" {
//    // Get new account from user and try again
//}
pub fn remove_account(vault: &String, name: &str, mut account: Vec<String>) -> String{
    let location = format!("{}/{}", vault, name);
    println!("Removing account...\n");
    // Find if specified account exists
    if Path::new(&location).exists() {
        // Remove account folder
        Command::new("rm")
            .args(["-r", location.as_str()]).output().expect("Could not remove account folder");
        // Remove account from accounts file
        account.retain(|account| *account != name);
        write_account(get_account_location(vault), &account);
        println!("\nSuccessfully removed account");
        return "ok".to_string();
    }
    else {
        println!("Account does not exist");
        return "err".to_string();
    }
}

// Loads json file and returns as Value
//
// USAGE
//
// var = load_json_as_value(json_file_directory)
pub fn load_json_as_value(json_file_directory: &String) -> serde_json::Value {
    // Read json file to string
    let json_as_string: String = fs::read_to_string(&json_file_directory).unwrap();
    // Convert to json in UserPass structure
    let json: serde_json::Value = serde_json::from_str(&json_as_string).unwrap();
    return json;
}

// Loads json file and returns as UserPass
//
// USAGE
//
// var = load_json_as_userpass(json_file_directory)
pub fn load_json_as_userpass(json_file_directory: &String) -> UserPass {
    // Read json file to string
    let json_as_string: String = fs::read_to_string(&json_file_directory).unwrap();
    // Convert to json in UserPass structure
    let json: UserPass = serde_json::from_str(&json_as_string).unwrap();
    return json;
}

// Changes the password of an account
//
// USAGE
//
// change_password(&vault_location, "password", "account")
pub fn change_password(vault: &String, password: &str, account: &str) {
    // Finds data.json location
    let json_file_directory = format!("{}/{}/data.json", vault, account);
    // Loads json
    let json: UserPass = load_json_as_userpass(&json_file_directory);
    // Saves username from json
    let username = json.username;
    // Creates blank json
    let mut new_json: serde_json::Value = serde_json::from_str("{}").unwrap();
    new_json = add_fields_to_json(new_json, username.as_str(), password);
    save_json_file(json_file_directory, new_json);
}

// Changes the username of an account
//
// USAGE
//
// change_username(&vault_location, "username", "account")
pub fn change_username(vault: &String, username: &str, account: &str) {
    // Finds data.json location
    let json_file_directory = format!("{}/{}/data.json", vault, account);
    // Loads json
    let json: UserPass = load_json_as_userpass(&json_file_directory);
    // Saves username from json
    let password = json.password;
    // Creates blank json
    let mut new_json: serde_json::Value = serde_json::from_str("{}").unwrap();
    new_json = add_fields_to_json(new_json, password.as_str(), username);
    save_json_file(json_file_directory, new_json);
}

// Adds fields to json
//
// USAGE
//
// var = add_fields_to_json(json, "username", "password");
pub fn add_fields_to_json(mut json: serde_json::Value, username: &str, password: &str) -> serde_json::Value {
    // Add username to json
    json["username"] = serde_json::Value::String(username.to_owned());
    // Add password to json
    json["password"] = serde_json::Value::String(password.to_owned());
    return json;
}