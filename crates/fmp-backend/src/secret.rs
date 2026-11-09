//! Contains the logic for containing and dropping secrets

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::fmt;
use zeroize::Zeroizing;

/// A string containing sensitive information.
/// Zeroized when dropped.
/// No implicit Clone, Deref or Debug disclosure IS or SHOULD BE provided.
pub struct SecretString {
    inner: Zeroizing<String>,
}

impl SecretString {
    pub fn new(value: String) -> Self {
        Self {
            inner: Zeroizing::new(value),
        }
    }

    /// Exposes the secret.
    pub fn expose_secret(&self) -> &str {
        self.inner.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the number of Unicode scalar values.
    pub fn char_count(&self) -> usize {
        self.inner.chars().count()
    }

    /// Replacing the Zeroizing<String>, drops and wipes the previous allocation.
    pub fn replace(&mut self, value: String) {
        self.inner = Zeroizing::new(value);
    }
}

impl Default for SecretString {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString([X])")
    }
}

impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expose_secret())
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Ok(Self::new(value))
    }
}

// Most tests LLM generated, I would LOVE to write them manually but im.... busy
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_does_not_expose_secret() {
        let secret = SecretString::new("super-secret-password".into());

        let debug = format!("{secret:?}");

        assert!(!debug.contains("super-secret-password"));
        assert!(debug.contains("[X]"));
    }

    #[test]
    fn secret_can_be_exposed_explicitly() {
        let secret = SecretString::new("test-password".into());

        assert_eq!(secret.expose_secret(), "test-password");
    }

    #[test]
    fn replacement_updates_secret() {
        let mut secret = SecretString::new("old-password".into());

        secret.replace("new-password".into());

        assert_eq!(secret.expose_secret(), "new-password");
        assert_eq!(secret.len(), 12);
    }

    #[test]
    fn empty_secret_is_handled() {
        let secret = SecretString::default();

        assert!(secret.is_empty());
        assert_eq!(secret.len(), 0);
    }

    #[test]
    fn serialization_is_explicitly_plaintext() {
        let secret = SecretString::new("test-password".into());

        let serialized = serde_json::to_string(&secret).unwrap();

        assert_eq!(serialized, "\"test-password\"");
    }
}
