use crate::security::secure_clipboard::SecureClipboardString;

#[test]
fn test_secure_clipboard_string_new() {
    let content = "test clipboard content".to_string();
    let secure_clipboard = SecureClipboardString::new(content);

    // Should not panic on creation
    assert_eq!(secure_clipboard.len(), 22);
}

#[test]
fn test_secure_clipboard_string_deref() {
    let content = "deref test".to_string();
    let secure_clipboard = SecureClipboardString::new(content.clone());

    // Should be able to use as str through deref
    assert_eq!(&*secure_clipboard, &content);
    assert_eq!(secure_clipboard.len(), content.len());
    assert!(secure_clipboard.contains("deref"));
}

#[test]
fn test_secure_clipboard_string_with_exposed() {
    let content = "exposed content".to_string();
    let secure_clipboard = SecureClipboardString::new(content.clone());

    let result = secure_clipboard.with_exposed(|exposed| {
        assert_eq!(exposed, &content);
        exposed.len()
    });

    assert_eq!(result, content.len());
}

#[test]
fn test_secure_clipboard_string_empty() {
    let secure_clipboard = SecureClipboardString::new(String::new());

    assert_eq!(secure_clipboard.len(), 0);
    assert!(secure_clipboard.is_empty());

    secure_clipboard.with_exposed(|exposed| {
        assert_eq!(exposed, "");
    });
}

#[test]
fn test_secure_clipboard_string_unicode() {
    let content = "🔒 Secure 密码".to_string();
    let secure_clipboard = SecureClipboardString::new(content.clone());

    secure_clipboard.with_exposed(|exposed| {
        assert_eq!(exposed, &content);
        assert!(exposed.contains("🔒"));
        assert!(exposed.contains("密码"));
    });
}

#[test]
fn test_secure_clipboard_string_special_chars() {
    let content = "!@#$%^&*()_+-=[]{}|;:,.<>?".to_string();
    let secure_clipboard = SecureClipboardString::new(content.clone());

    secure_clipboard.with_exposed(|exposed| {
        assert_eq!(exposed, &content);
        assert!(exposed.contains("!@#$"));
        assert!(exposed.contains("<>?"));
    });
}

#[test]
fn test_secure_clipboard_string_long_content() {
    let content = "x".repeat(10000);
    let secure_clipboard = SecureClipboardString::new(content.clone());

    assert_eq!(secure_clipboard.len(), 10000);

    secure_clipboard.with_exposed(|exposed| {
        assert_eq!(exposed.len(), 10000);
        assert!(exposed.chars().all(|c| c == 'x'));
    });
}

#[test]
fn test_secure_clipboard_string_whitespace() {
    let content = "  \t\n  test content  \t\n  ".to_string();
    let secure_clipboard = SecureClipboardString::new(content.clone());

    secure_clipboard.with_exposed(|exposed| {
        assert_eq!(exposed, &content);
        assert!(exposed.contains("test content"));
        assert!(exposed.starts_with("  \t\n"));
        assert!(exposed.ends_with("\t\n  "));
    });
}

#[test]
fn test_secure_clipboard_string_multiple_operations() {
    let content = "multi operation test".to_string();
    let secure_clipboard = SecureClipboardString::new(content.clone());

    // Multiple calls should work
    let len1 = secure_clipboard.with_exposed(|exposed| exposed.len());
    let len2 = secure_clipboard.with_exposed(|exposed| exposed.len());
    let contains_test = secure_clipboard.with_exposed(|exposed| exposed.contains("test"));

    assert_eq!(len1, len2);
    assert_eq!(len1, content.len());
    assert!(contains_test);
}
