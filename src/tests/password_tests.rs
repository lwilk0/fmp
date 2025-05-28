use crate::password::*;

#[test]
fn test_generate_password_length() {
    let length = 12;
    let password = generate_password(length);

    assert_eq!(password.len(), length);
}

#[test]
fn test_calculate_entropy() {
    let password = String::from("P@ssw0rd123");
    let (entropy, rating) = calculate_entropy(&password);

    assert!(entropy > 0.0);
    assert_eq!(rating, "Strong");
}

#[test]
fn test_calculate_entropy_only_lowercase() {
    let password = String::from("password");
    let (entropy, rating) = calculate_entropy(&password);

    assert!(entropy > 0.0);
    assert_eq!(rating, "Weak");
}

#[test]
fn test_calculate_entropy_only_uppercase() {
    let password = String::from("PASSWORD");
    let (entropy, rating) = calculate_entropy(&password);

    assert!(entropy > 0.0);
    assert_eq!(rating, "Weak");
}

#[test]
fn test_calculate_entropy_only_digits() {
    let password = String::from("12345678");
    let (entropy, rating) = calculate_entropy(&password);

    assert!(entropy > 0.0);
    assert_eq!(rating, "Very Weak");
}

#[test]
fn test_calculate_entropy_only_punctuation() {
    let password = String::from("!@#$%^&*()");
    let (entropy, rating) = calculate_entropy(&password);

    assert!(entropy > 0.0);
    assert_eq!(rating, "Weak");
}
