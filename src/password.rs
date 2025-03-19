use rand::Rng;aa

// Generates a password
//
// USAGE
//
// generate_password(12)
pub fn generate_password(length: u32) -> String {
    let password_characters = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','1','2','3','4','5','6','7','8','9','0','!','"','#','$','%','&','\'','(',')','*','+',',','-','.','/',':',';','<','=','>','?','@','[','\\',']','^','_','`','{','|','}','~'];
    let mut generated_password: String = String::new();
    
    // Loop for length
    for _i in 0..length {
        // Generate random integer between 0 and length of password_characters
        let random_integer: usize = rand::rng().random_range(1..=password_characters.len()-1);
        // Get character at location random_integer of password_characters
        let random_character: char = password_characters[random_integer];
        // Concatenate generated password thus far with random_character
        generated_password = format!("{}{}", generated_password, random_character);
    }
    return generated_password;
}

// Calculates the entropy of a password
//
// USAGE
//
// let var: (f64, &str) = calculate_entropy(&"password")
pub fn calculate_entropy(password: &String) -> (f64, &str) {
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
    let entropy = password.len() as f64 * (character_pool as f64).log2();
    let rating: &str;
    
    // Gets password rating
    if entropy <= 35.0 {
        rating = "Very Weak"
    }
    
    else if entropy <= 59.0 {
        rating = "Weak"
    }
    
    else if entropy <= 119.0 {
        rating = "Strong"
    }
    
    else {
        rating = "Very Strong"
    }
    // Return entropy and rating as tuple
    return (entropy, rating);
}