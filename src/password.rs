use rand::Rng;
use import_handle;

use super::json;
use super::vault;
use super::account;

// Generates a password
//
// USAGE
//
// generate_password(12)
pub fn generate_password(length: u8) {
    let password_characters = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','1','2','3','4','5','6','7','8','9','0','!','"','#','$','%','&','\'','(',')','*','+',',','-','.','/',':',';','<','=','>','?','@','[','\\',']','^','_','`','{','|','}','~'];
    let generated_password: String = String::new();
    // Loop for length
    for i in 0..length {
        // Generate random integer between 0 and length of password_characters
        let random_integer: usize = rand::thread_rng().gen_range(1..=password_characters.len()-1);
        // Get character at location random_integer of password_characters
        let random_character: char = password_characters[random_integer];
        // Concatenate generated password thus far with random_character
        let generated_password: String = format!("{}{}", generated_password, random_character);
    }
    // Prints generated password
    println!("{}", generated_password);
    // Ask user if they want to link the password to account
    let mut user_input: String = String::new();
    while user_input != "y" && user_input != "yes" && user_input != "n" && user_input != "no" {
        user_input = import_handle::get_string_input("Would you like to save this password to an account? (y)es, (n)o").to_lowercase();
    }
    // If they do want to link account
    if user_input == "y" || user_input == "yes" {
        // Get user inputs
        let account = account::read_account(account::get_account_location());
        let name = import_handle::get_string_input("What should the account be named? ");
        let username = import_handle::get_string_input("\nWhat is the account username?");
        // Create new account
        json::new_json_account(vault::get_fmp_vault_location(), name.as_str(), username.as_str(), &generated_password, account);
        // Exit
        vault::encrypt_and_exit();
    }
}

// Calculates the entropy of a password
//
// USAGE
//
// calculate_entropy("password")
pub fn calculate_entropy(password: String) {
    let mut character_pool: u8 = 0;

    // Calculates password pool
    if password.chars().any(|c| c.is_ascii_lowercase()) {
        character_pool += 26;
    }
    if password.chars().any(|c| c.is_ascii_uppercase()) {
        character_pool += 26;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        character_pool += 10;
    }
    if password.chars().any(|c| c.is_ascii_punctuation()){
        character_pool += 32;
    }
 
    // Calculates and prints entropy
    // FORMULA
    // L * log2(C), where L is the length of the password and C is the character pool
    println!("The password has {:.2} bit entropy", password.len() as f64 * (character_pool as f64).log2() )
}