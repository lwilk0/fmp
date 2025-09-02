use rand::prelude::IndexedRandom;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;

/// Configuration for password generation
#[derive(Debug, Clone)]
pub struct PasswordConfig {
    pub length: usize,
    pub include_lowercase: bool,
    pub include_uppercase: bool,
    pub include_numbers: bool,
    pub include_symbols: bool,
    pub include_spaces: bool,
    pub include_extended: bool,
    pub additional_characters: String,
    pub excluded_characters: String,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: 16,
            include_lowercase: true,
            include_uppercase: true,
            include_numbers: true,
            include_symbols: false,
            include_spaces: false,
            include_extended: false,
            additional_characters: String::new(),
            excluded_characters: String::new(),
        }
    }
}

/// Generates a random password based on the provided configuration.
///
/// # Arguments
/// * `password_config` - The password generation configuration
///
/// # Returns
/// * `Result<String, String>` - The generated password or an error message
pub fn generate_password(password_config: &PasswordConfig) -> Result<String, String> {
    let mut character_pool = String::new();

    if password_config.include_lowercase {
        character_pool.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if password_config.include_uppercase {
        character_pool.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if password_config.include_numbers {
        character_pool.push_str("0123456789");
    }
    if password_config.include_symbols {
        character_pool.push_str("!\"#%&'()*+,-./:;<=>?@[\\]^_`{|}-");
    }
    if password_config.include_spaces {
        character_pool.push(' ');
    }
    if password_config.include_extended {
        character_pool.push_str("áÁàÀâÂäÄãÃåÅæÆçÇéÉèÈêÊëËíÍìÌîÎïÏñÑóÓòÒôÔöÖõÕøØœŒßúÚùÙûÛüÜ");
    }

    let mut base_character_set: HashSet<char> = character_pool.chars().collect();
    let additional_chars: HashSet<char> = password_config.additional_characters.chars().collect();
    let excluded_chars: HashSet<char> = password_config.excluded_characters.chars().collect();

    // Remove excluded characters
    for character in &excluded_chars {
        base_character_set.remove(character);
    }

    // Add additional characters
    for &character in &additional_chars {
        base_character_set.insert(character);
    }

    let available_characters: Vec<char> = base_character_set.into_iter().collect();

    if available_characters.is_empty() {
        return Err("No characters available for password generation".to_string());
    }

    if password_config.length == 0 {
        return Err("Password length must be greater than 0".to_string());
    }

    let mut random_generator = thread_rng();
    let generated_password: String = (0..password_config.length)
        .map(|_| *available_characters.choose(&mut random_generator).unwrap())
        .collect();

    Ok(generated_password)
}
