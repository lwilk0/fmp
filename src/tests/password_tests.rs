use crate::password::*;

#[test]
fn test_generate_password_default() {
    let config = PasswordConfig::default();
    let password = generate_password(&config).unwrap();

    assert_eq!(password.len(), config.length);
}

#[test]
fn test_generate_password_custom_length() {
    let config = PasswordConfig {
        length: 20,
        ..PasswordConfig::default()
    };

    let password = generate_password(&config).unwrap();
    assert_eq!(password.len(), 20);
}

#[test]
fn test_generate_password_only_lowercase() {
    let config = PasswordConfig {
        length: 20,
        include_uppercase: false,
        include_numbers: false,
        include_symbols: false,
        ..PasswordConfig::default()
    };

    let password = generate_password(&config).unwrap();

    // Should only contain lowercase
    assert!(password.chars().all(|c| c.is_ascii_lowercase()));
}

#[test]
fn test_generate_password_only_uppercase() {
    let config = PasswordConfig {
        length: 20,
        include_lowercase: false,
        include_numbers: false,
        include_symbols: false,
        include_uppercase: true,
        ..PasswordConfig::default()
    };

    let password = generate_password(&config).unwrap();

    // Should only contain uppercase
    assert!(password.chars().all(|c| c.is_ascii_uppercase()));
}

#[test]
fn test_generate_password_only_numbers() {
    let config = PasswordConfig {
        length: 20,
        include_lowercase: false,
        include_uppercase: false,
        include_symbols: false,
        include_numbers: true,
        ..PasswordConfig::default()
    };

    let password = generate_password(&config).unwrap();

    // Should only contain digits
    assert!(password.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_generate_password_only_symbols() {
    let config = PasswordConfig {
        length: 20,
        include_lowercase: false,
        include_uppercase: false,
        include_numbers: false,
        include_symbols: true,
        ..PasswordConfig::default()
    };

    let password = generate_password(&config).unwrap();

    // Should only contain symbols
    assert!(password.chars().all(|c| SYMBOLS.contains(c)));
}

#[test]
fn test_generate_password_mixed() {
    let config = PasswordConfig {
        length: 100, // Larger length to increase probability
        include_lowercase: true,
        include_uppercase: true,
        include_numbers: true,
        include_symbols: true,
        include_spaces: false,
        include_extended: false,
        additional_characters: String::new(),
        excluded_characters: String::new(),
    };

    let password = generate_password(&config).unwrap();

    // Should contain all types (with high probability for length 100)
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| SYMBOLS.contains(c));

    // At least 3 out of 4 should be present
    let present_types = [has_lower, has_upper, has_digit, has_symbol]
        .iter()
        .filter(|&&x| x)
        .count();
    assert!(present_types >= 3);
}

#[test]
fn test_generate_password_exclude_ambiguous() {
    let config = PasswordConfig {
        length: 50,
        include_lowercase: true,
        include_uppercase: true,
        include_numbers: true,
        include_symbols: false, // Keep symbols false for this test
        include_spaces: false,
        include_extended: false,
        additional_characters: String::new(),
        excluded_characters: "0OIl1".to_string(), // Exclude ambiguous characters
    };

    let password = generate_password(&config).unwrap();

    // Should not contain ambiguous characters: 0, O, I, l, 1
    assert!(!password.contains('0'));
    assert!(!password.contains('O'));
    assert!(!password.contains('I'));
    assert!(!password.contains('l'));
    assert!(!password.contains('1'));
}

#[test]
fn test_generate_password_with_lowercase_and_symbols() {
    let config = PasswordConfig {
        length: 20,
        include_lowercase: true,
        include_uppercase: false,
        include_numbers: false,
        include_symbols: true,
        include_spaces: false,
        include_extended: false,
        additional_characters: String::new(),
        excluded_characters: String::new(),
    };

    let password = generate_password(&config).unwrap();

    // Should only contain lowercase and symbols
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| SYMBOLS.contains(c));

    assert!(has_lower);
    assert!(!has_upper);
    assert!(!has_digit);
    assert!(has_symbol);
}

#[test]
fn test_calculate_strength_various_passwords() {
    // Test empty password
    assert_eq!(calculate_password_strength(""), 0);

    // Test weak password (very short with repetition)
    let weak_strength = calculate_password_strength("aa");
    assert!(weak_strength <= 25);

    // Test strong password
    let strong_strength = calculate_password_strength("Abc123!@#XyzLongPassword");
    assert!(strong_strength >= 76);

    // Test medium password
    let medium_strength = calculate_password_strength("Password123");
    assert!(medium_strength > 25 && medium_strength <= 75);
}

#[test]
fn test_handle_empty_password_length() {
    let config = PasswordConfig {
        length: 0,
        ..PasswordConfig::default()
    };

    let result = generate_password(&config);
    assert!(result.is_err());
}

#[test]
fn test_handle_no_character_types() {
    let config = PasswordConfig {
        length: 10,
        include_lowercase: false,
        include_uppercase: false,
        include_numbers: false,
        include_symbols: false,
        include_spaces: false,
        include_extended: false,
        additional_characters: String::new(),
        excluded_characters: String::new(),
    };

    let result = generate_password(&config);
    assert!(result.is_err());
}

#[test]
fn test_generate_password_deterministic_within_constraints() {
    // Multiple generations should produce different passwords
    let config = PasswordConfig {
        length: 10,
        ..PasswordConfig::default()
    };

    let password1 = generate_password(&config).unwrap();
    let password2 = generate_password(&config).unwrap();
    let password3 = generate_password(&config).unwrap();

    // Very unlikely to be identical (possible but highly improbable)
    assert!(password1 != password2 || password2 != password3);
}

#[test]
fn test_calculate_password_strength_edge_cases() {
    // Test single character
    let single_strength = calculate_password_strength("a");
    assert!(single_strength > 0 && single_strength <= 100);

    // Test very long password
    let long_password = "a".repeat(1000);
    let long_strength = calculate_password_strength(&long_password);
    assert!(long_strength > 0 && long_strength <= 100);

    // Test special characters
    let special_strength = calculate_password_strength("@#$%^&*()");
    assert!(special_strength > 0 && special_strength <= 100);
}
