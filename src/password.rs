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
/// * `config` - The password generation configuration
///
/// # Returns
/// * `Result<String, String>` - The generated password or an error message
pub fn generate_password(config: &PasswordConfig) -> Result<String, String> {
    let mut pool = String::new();

    if config.include_lowercase {
        pool.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if config.include_uppercase {
        pool.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if config.include_numbers {
        pool.push_str("0123456789");
    }
    if config.include_symbols {
        pool.push_str("!\"#%&'()*+,-./:;<=>?@[\\]^_`{|}-");
    }
    if config.include_spaces {
        pool.push(' ');
    }
    if config.include_extended {
        pool.push_str("áÁàÀâÂäÄãÃåÅæÆçÇéÉèÈêÊëËíÍìÌîÎïÏñÑóÓòÒôÔöÖõÕøØœŒßúÚùÙûÛüÜ");
    }

    let mut base: HashSet<char> = pool.chars().collect();
    let include: HashSet<char> = config.additional_characters.chars().collect();
    let exclude: HashSet<char> = config.excluded_characters.chars().collect();

    // Remove excluded characters
    for ch in &exclude {
        base.remove(ch);
    }

    // Add additional characters
    for &ch in &include {
        base.insert(ch);
    }

    let pool_vec: Vec<char> = base.into_iter().collect();

    if pool_vec.is_empty() {
        return Err("No characters available for password generation".to_string());
    }

    if config.length == 0 {
        return Err("Password length must be greater than 0".to_string());
    }

    let mut rng = thread_rng();
    let password: String = (0..config.length)
        .map(|_| *pool_vec.choose(&mut rng).unwrap())
        .collect();

    Ok(password)
}
