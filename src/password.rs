use rand::random;

pub fn generate_password(length: usize) -> String {
    (0..length)
        .map(|_| (0x20u8 + (random::<f32>() * 96.0) as u8) as char)
        .collect()
}

pub fn calculate_entropy(password: &String) -> (f64, &str) {
    let mut character_pool: u8 = 0;

    if password.chars().any(|c| c.is_ascii_lowercase()) {
        character_pool += 26;
    }

    if password.chars().any(|c| c.is_ascii_uppercase()) {
        character_pool += 26;
    }

    if password.chars().any(|c| c.is_ascii_digit()) {
        character_pool += 10;
    }

    if password.chars().any(|c| c.is_ascii_punctuation()) {
        character_pool += 32;
    }

    // FORMULA
    // L * log2(C), where L is the length of the password and C is the character pool
    let entropy = password.len() as f64 * (character_pool as f64).log2();
    let rating: &str;

    if entropy <= 35.0 {
        rating = "Very Weak"
    } else if entropy <= 59.0 {
        rating = "Weak"
    } else if entropy <= 119.0 {
        rating = "Strong"
    } else {
        rating = "Very Strong"
    }
    // Return entropy and rating as tuple
    return (entropy, rating);
}

#[cfg(test)]
#[path = "tests/password_tests.rs"]
mod password_tests;
