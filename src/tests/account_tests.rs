use crate::models::account::Account;
use crate::security::secure_password::SecurePassword;

#[test]
fn test_account_default() {
    let account = Account::default();

    assert_eq!(account.name, String::new());
    assert_eq!(account.account_type, "Password Account");
    assert_eq!(account.website, String::new());
    assert_eq!(account.username, String::new());
    assert_eq!(account.password.len(), 0);
    assert_eq!(account.notes, String::new());
    assert_eq!(account.additional_fields.len(), 0);
    assert!(!account.modified_at.is_empty());
}

#[test]
fn test_account_add_additional_field() {
    let mut account = Account::default();

    account
        .additional_fields
        .insert("API Key".to_string(), "secret123".to_string());

    assert_eq!(account.additional_fields.len(), 1);
    assert_eq!(
        account.additional_fields.get("API Key"),
        Some(&"secret123".to_string())
    );
}

#[test]
fn test_account_add_multiple_additional_fields() {
    let mut account = Account::default();

    account
        .additional_fields
        .insert("API Key".to_string(), "secret123".to_string());
    account
        .additional_fields
        .insert("Token".to_string(), "token456".to_string());
    account
        .additional_fields
        .insert("Security Question".to_string(), "Pet name".to_string());

    assert_eq!(account.additional_fields.len(), 3);
    assert_eq!(
        account.additional_fields.get("API Key"),
        Some(&"secret123".to_string())
    );
    assert_eq!(
        account.additional_fields.get("Token"),
        Some(&"token456".to_string())
    );
    assert_eq!(
        account.additional_fields.get("Security Question"),
        Some(&"Pet name".to_string())
    );
}

#[test]
fn test_account_overwrite_additional_field() {
    let mut account = Account::default();

    account
        .additional_fields
        .insert("API Key".to_string(), "old_secret".to_string());
    account
        .additional_fields
        .insert("API Key".to_string(), "new_secret".to_string());

    assert_eq!(account.additional_fields.len(), 1);
    assert_eq!(
        account.additional_fields.get("API Key"),
        Some(&"new_secret".to_string())
    );
}

#[test]
fn test_update_modified_time() {
    let mut account = Account::default();
    let original_time = account.modified_at.clone();

    // Sleep longer to ensure time difference
    std::thread::sleep(std::time::Duration::from_secs(1));

    account.update_modified_time();

    // Modified time should have changed
    assert_ne!(account.modified_at, original_time);

    // Should be a valid timestamp format
    assert!(account.modified_at.len() >= 19); // "YYYY-MM-DD HH:MM:SS" format
    assert!(account.modified_at.contains(' '));
    assert!(account.modified_at.contains(':'));
    assert!(account.modified_at.contains('-'));
}

#[test]
fn test_account_serialization() {
    let account = Account::default();

    // Test that the account can be serialized (should not panic)
    let serialized = serde_json::to_string(&account);
    assert!(serialized.is_ok());

    // Test deserialization
    let serialized_str = serialized.unwrap();
    let deserialized: Result<Account, _> = serde_json::from_str(&serialized_str);
    assert!(deserialized.is_ok());

    let deserialized_account = deserialized.unwrap();
    assert_eq!(account.name, deserialized_account.name);
    assert_eq!(account.username, deserialized_account.username);
    account.password.with_exposed(|orig_pass| {
        deserialized_account.password.with_exposed(|deser_pass| {
            assert_eq!(orig_pass, deser_pass);
        });
    });
}

#[test]
fn test_account_with_all_fields() {
    let mut account = Account {
        name: "Test Account".to_string(),
        account_type: "Email Account".to_string(),
        website: "https://example.com".to_string(),
        username: "testuser".to_string(),
        password: SecurePassword::new("testpass123".to_string()),
        notes: "Test notes".to_string(),
        ..Account::default()
    };

    account.additional_fields.insert(
        "Recovery Email".to_string(),
        "recovery@test.com".to_string(),
    );

    // Test serialization with all fields populated
    let serialized = serde_json::to_string(&account);
    assert!(serialized.is_ok());

    let serialized_str = serialized.unwrap();
    let deserialized: Account = serde_json::from_str(&serialized_str).unwrap();

    assert_eq!(account.name, deserialized.name);
    assert_eq!(account.account_type, deserialized.account_type);
    assert_eq!(account.website, deserialized.website);
    assert_eq!(account.username, deserialized.username);
    account.password.with_exposed(|orig_pass| {
        deserialized.password.with_exposed(|deser_pass| {
            assert_eq!(orig_pass, deser_pass);
        });
    });
    assert_eq!(account.notes, deserialized.notes);
    assert_eq!(account.additional_fields, deserialized.additional_fields);
}

#[test]
fn test_account_empty_additional_field_key() {
    let mut account = Account::default();

    // Should handle empty key
    account
        .additional_fields
        .insert("".to_string(), "value".to_string());

    assert_eq!(account.additional_fields.len(), 1);
    assert_eq!(
        account.additional_fields.get(""),
        Some(&"value".to_string())
    );
}

#[test]
fn test_account_empty_additional_field_value() {
    let mut account = Account::default();

    // Should handle empty value
    account
        .additional_fields
        .insert("key".to_string(), "".to_string());

    assert_eq!(account.additional_fields.len(), 1);
    assert_eq!(account.additional_fields.get("key"), Some(&"".to_string()));
}

#[test]
fn test_account_unicode_fields() {
    let mut account = Account {
        name: "测试账户".to_string(),
        account_type: "Password Account".to_string(),
        website: "https://测试.com".to_string(),
        username: "用户名".to_string(),
        password: SecurePassword::new("密码123".to_string()),
        notes: "测试笔记 with 🔒".to_string(),
        ..Account::default()
    };

    account
        .additional_fields
        .insert("密钥".to_string(), "秘密值".to_string());

    // Test serialization with Unicode content
    let serialized = serde_json::to_string(&account);
    assert!(serialized.is_ok());

    let serialized_str = serialized.unwrap();
    let deserialized: Account = serde_json::from_str(&serialized_str).unwrap();

    assert_eq!(account.name, deserialized.name);
    assert_eq!(account.account_type, deserialized.account_type);
    assert_eq!(account.website, deserialized.website);
    assert_eq!(account.username, deserialized.username);
    account.password.with_exposed(|orig_pass| {
        deserialized.password.with_exposed(|deser_pass| {
            assert_eq!(orig_pass, deser_pass);
        });
    });
    assert_eq!(account.notes, deserialized.notes);
    assert_eq!(account.additional_fields, deserialized.additional_fields);
}
