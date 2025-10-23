use crate::security::secure_password::SecurePassword;

#[test]
fn test_secure_password_new() {
    let password = SecurePassword::new("test123".to_string());
    assert_eq!(password.len(), 7);
}

#[test]
fn test_secure_password_empty() {
    let password = SecurePassword::empty();
    assert_eq!(password.len(), 0);
}

#[test]
fn test_secure_password_masked() {
    let password = SecurePassword::new("secret".to_string());
    let masked = password.masked(0);
    assert_eq!(masked, "••••••");

    let password_empty = SecurePassword::empty();
    let masked_empty = password_empty.masked(8);
    assert_eq!(masked_empty, "••••••••");
}

#[test]
fn test_secure_password_with_exposed() {
    let password = SecurePassword::new("mypassword".to_string());

    let result = password.with_exposed(|pass| {
        assert_eq!(pass, "mypassword");
        pass.len()
    });

    assert_eq!(result, 10);
}

#[test]
fn test_secure_password_expose_for_clipboard() {
    let password = SecurePassword::new("clipboardtest".to_string());
    let clipboard_string = password.expose_for_clipboard();

    clipboard_string.with_exposed(|pass| {
        assert_eq!(pass, "clipboardtest");
    });
}

#[test]
fn test_secure_password_serialization() {
    let password = SecurePassword::new("serialize_test".to_string());

    let serialized = serde_json::to_string(&password);
    assert!(serialized.is_ok());
    assert!(serialized.unwrap().contains("serialize_test"));
}

#[test]
fn test_secure_password_deserialization() {
    let json = "\"deserialize_test\"";
    let deserialized: Result<SecurePassword, _> = serde_json::from_str(json);

    assert!(deserialized.is_ok());
    let password = deserialized.unwrap();
    assert_eq!(password.len(), 16);

    password.with_exposed(|pass| {
        assert_eq!(pass, "deserialize_test");
    });
}

#[test]
fn test_secure_password_roundtrip_serialization() {
    let original = SecurePassword::new("roundtrip".to_string());

    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: SecurePassword = serde_json::from_str(&serialized).unwrap();

    assert_eq!(original.len(), deserialized.len());

    original.with_exposed(|orig_pass| {
        deserialized.with_exposed(|deser_pass| {
            assert_eq!(orig_pass, deser_pass);
        });
    });
}

#[test]
fn test_secure_password_clone() {
    let original = SecurePassword::new("clonetest".to_string());
    let cloned = original.clone();

    assert_eq!(original.len(), cloned.len());

    original.with_exposed(|orig_pass| {
        cloned.with_exposed(|cloned_pass| {
            assert_eq!(orig_pass, cloned_pass);
        });
    });
}

#[test]
fn test_secure_password_debug() {
    let password = SecurePassword::new("debugtest".to_string());
    let debug_str = format!("{password:?}");

    // Debug output should not contain the actual password
    assert!(!debug_str.contains("debugtest"));
    assert!(debug_str.contains("SecurePassword"));
}

#[test]
fn test_secure_password_empty_operations() {
    let empty = SecurePassword::empty();

    assert_eq!(empty.len(), 0);
    assert_eq!(empty.masked(0), "");
    assert_eq!(empty.masked(5), "•••••");

    empty.with_exposed(|pass| {
        assert_eq!(pass, "");
    });
}
