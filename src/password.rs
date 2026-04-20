use rand::{Rng, rng};
use std::collections::HashSet;

/// Configuration for password generation
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
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

// Pre-defined character sets for better performance
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMBERS: &str = "0123456789";
pub const SYMBOLS: &str = "!\"#%&'()*+,-./:;<=>?@[\\]^_`{|}$~";
const EXTENDED: &str = "áÁàÀâÂäÄãÃåÅæÆçÇéÉèÈêÊëËíÍìÌîÎïÏñÑóÓòÒôÔöÖõÕøØœŒßúÚùÙûÛüÜ";

// TODO: Proper errors
/// Generates a random password based on the provided configuration.
///
/// # Arguments
/// * `password_config` - The password generation configuration
///
/// # Returns
/// * `Result<String, String>` - The generated password or an error message
pub fn generate_password(password_config: &PasswordConfig) -> Result<String, String> {
    if password_config.length == 0 {
        return Err("Password length must be greater than 0".to_string());
    }

    // Build character pool more efficiently
    let mut character_pool = String::with_capacity(256);

    if password_config.include_lowercase {
        character_pool.push_str(LOWERCASE);
    }
    if password_config.include_uppercase {
        character_pool.push_str(UPPERCASE);
    }
    if password_config.include_numbers {
        character_pool.push_str(NUMBERS);
    }
    if password_config.include_symbols {
        character_pool.push_str(SYMBOLS);
    }
    if password_config.include_spaces {
        character_pool.push(' ');
    }
    if password_config.include_extended {
        character_pool.push_str(EXTENDED);
    }

    // Add additional characters
    character_pool.push_str(&password_config.additional_characters);

    // Convert to HashSet only if we need to exclude characters
    let available_characters: Vec<char> = if password_config.excluded_characters.is_empty() {
        character_pool.chars().collect()
    } else {
        let excluded_chars: HashSet<char> = password_config.excluded_characters.chars().collect();
        character_pool
            .chars()
            .filter(|c| !excluded_chars.contains(c))
            .collect()
    };

    if available_characters.is_empty() {
        return Err("No characters available for password generation".to_string());
    }

    // Generate password more efficiently
    let mut rng = rng();
    let mut password = String::with_capacity(password_config.length);

    for _ in 0..password_config.length {
        let idx = rng.random_range(0..available_characters.len());
        password.push(available_characters[idx]);
    }

    Ok(password)
}

/// Calculates password strength score (0-100) based on estimated bit entropy.
///
/// Entropy = log2(pool_size) * length, capped at 128 bits = 100.
pub fn calculate_password_strength(password: &str) -> u8 {
    if password.is_empty() {
        return 0;
    }

    let length = password.len();

    // Detect which character classes are present to estimate pool size
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| !c.is_alphanumeric() && c.is_ascii());
    let has_extended = password.chars().any(|c| !c.is_ascii());

    let pool_size: f32 = [
        (has_lower, 26.0f32),
        (has_upper, 26.0),
        (has_digit, 10.0),
        (has_special, 32.0),
        (has_extended, 64.0),
    ]
    .iter()
    .filter(|(present, _)| *present)
    .map(|(_, size)| size)
    .sum();

    if pool_size == 0.0 {
        return 0;
    }

    // bits of entropy = log2(pool_size) * length
    // cap at 128 bits → 100 score
    let bits = pool_size.log2() * length as f32;
    let score = ((bits / 128.0) * 100.0).min(100.0) as u8;
    score
}

/// Returns a color class based on password strength
pub fn get_strength_color_class(strength: u8) -> &'static str {
    match strength {
        0..=25 => "strength-weak",
        26..=50 => "strength-fair",
        51..=75 => "strength-good",
        _ => "strength-strong",
    }
}

/// Returns a human-readable strength description
pub fn get_strength_description(strength: u8) -> &'static str {
    match strength {
        0..=25 => "Weak",
        26..=50 => "Fair",
        51..=75 => "Good",
        _ => "Strong",
    }
}
