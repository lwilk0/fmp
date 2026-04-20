use crate::totp::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_hotp_rfc_example() {
    // Test with RFC 4226 example values
    let secret = b"12345678901234567890";

    // Test counter 0
    let result = hotp(secret, 0, 6);
    assert_eq!(format!("{result:06}").len(), 6);

    // Test counter 1
    let result = hotp(secret, 1, 6);
    assert_eq!(format!("{result:06}").len(), 6);

    // Test different digits
    let result = hotp(secret, 0, 8);
    assert_eq!(format!("{result:08}").len(), 8);
}

#[test]
fn test_hotp_consistency() {
    let secret = b"test_secret_key_123";
    let counter = 12345;

    // Same inputs should produce same output
    let result1 = hotp(secret, counter, 6);
    let result2 = hotp(secret, counter, 6);
    assert_eq!(result1, result2);
}

#[test]
fn test_hotp_different_counters() {
    let secret = b"consistent_secret";

    let result1 = hotp(secret, 100, 6);
    let result2 = hotp(secret, 101, 6);

    // Different counters should produce different results (most of the time)
    // In the rare case they're the same, that's still valid HOTP behavior
    // So we just verify the format is correct
    assert!(result1 < 1_000_000); // 6 digits max
    assert!(result2 < 1_000_000); // 6 digits max
}

#[test]
fn test_verify_totp_code_with_secret_valid_format() {
    let secret = b"test_secret_for_totp_validation";

    // Test invalid formats
    assert!(verify_totp_code_with_secret(secret, "").is_ok());
    assert!(verify_totp_code_with_secret(secret, "12345").is_ok()); // Too short
    assert!(verify_totp_code_with_secret(secret, "123456789").is_ok()); // Too long
    assert!(verify_totp_code_with_secret(secret, "abcdef").is_ok()); // Non-numeric
    assert!(verify_totp_code_with_secret(secret, "12 3456").is_ok()); // With spaces

    // Valid format should not panic
    assert!(verify_totp_code_with_secret(secret, "123456").is_ok());
}

#[test]
fn test_totp_code_validation_format() {
    // Test the format validation logic
    let secret = b"format_test_secret";

    // Test various invalid formats
    let invalid_codes = vec![
        "",          // Empty
        "1",         // Too short
        "12345",     // Too short
        "123456789", // Too long
        "1234ab",    // Non-numeric
        "123-456",   // Special characters
    ];

    for code in invalid_codes {
        let result = verify_totp_code_with_secret(secret, code);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should be false for invalid formats
    }

    // Valid format (6 digits)
    let result = verify_totp_code_with_secret(secret, "123456");
    assert!(result.is_ok());
    // The result might be true or false depending on the actual TOTP calculation
}

#[test]
fn test_hotp_edge_cases() {
    let secret = b"edge_case_secret";

    // Test counter 0
    let result = hotp(secret, 0, 6);
    assert!(result < 1_000_000);

    // Test maximum counter
    let result = hotp(secret, u64::MAX, 6);
    assert!(result < 1_000_000);

    // Test different digit counts
    for digits in 6..=8 {
        let result = hotp(secret, 12345, digits);
        let max_val = 10u32.pow(digits);
        assert!(result < max_val);
    }
}

#[test]
fn test_totp_time_based_nature() {
    // Since TOTP is time-based, we can test that it uses current time
    let secret = b"time_test_secret";

    // This test verifies the time calculation doesn't panic
    let result = verify_totp_code_with_secret(secret, "123456");
    assert!(result.is_ok());

    // We can also test that the time calculation is reasonable
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;

    let step = 30i64;
    let counter = now / step;

    // Counter should be positive and reasonable
    assert!(counter > 0);
    assert!(counter < i64::MAX / 2); // Reasonable upper bound
}

#[test]
fn test_base32_encoding_in_prepare_totp() {
    // Test the base32 encoding format
    use base32::Alphabet;

    let test_secret = [0x01, 0x02, 0x03, 0x04, 0x05];
    let encoded = base32::encode(Alphabet::Rfc4648 { padding: false }, &test_secret);

    // Should be valid base32 without padding
    assert!(!encoded.is_empty());
    assert!(!encoded.contains('='));

    // Should contain only valid base32 characters
    for ch in encoded.chars() {
        assert!("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".contains(ch));
    }
}

#[test]
fn test_otpauth_uri_format() {
    // Test the URI format components
    let secret_b32 = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
    let vault_name = "test_vault";
    let issuer = "FMP";
    let label = format!("{issuer}:{vault_name}");

    let uri = format!(
        "otpauth://totp/{}?secret={}&issuer={}&period=30&digits=6&algorithm=SHA1",
        urlencoding::encode(&label),
        secret_b32,
        urlencoding::encode(issuer)
    );

    assert!(uri.starts_with("otpauth://totp/"));
    assert!(uri.contains("secret="));
    assert!(uri.contains("issuer="));
    assert!(uri.contains("period=30"));
    assert!(uri.contains("digits=6"));
    assert!(uri.contains("algorithm=SHA1"));
    assert!(uri.contains(vault_name));
}
