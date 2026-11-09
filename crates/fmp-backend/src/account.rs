//! Contains the account model and validation
//!

use crate::secret::SecretString;
use anyhow::{Result, ensure};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

const ACCOUNT_API_VERSION: u32 = 1;

fn current_timestamp() -> String {
    Utc::now().to_rfc3339()
}

fn default_account_type() -> String {
    "Password Account".to_string()
}

#[derive(Serialize, Deserialize)]
pub struct Account {
    #[serde(default = "default_api_version")]
    pub api_version: u32,

    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    pub name: SecretString,

    #[serde(default = "default_account_type")]
    pub account_type: String,

    #[serde(default)]
    pub website: SecretString,

    #[serde(default)]
    pub username: SecretString,

    #[serde(default)]
    pub password: SecretString,

    #[serde(default)]
    pub notes: SecretString,

    #[serde(default)]
    pub additional_fields: BTreeMap<String, SecretString>,

    #[serde(default)]
    pub otpauth: Option<SecretString>,

    #[serde(default = "current_timestamp")]
    pub created_at: String,

    #[serde(default = "current_timestamp")]
    pub modified_at: String,
}

fn default_api_version() -> u32 {
    ACCOUNT_API_VERSION
}

impl Account {
    /// Creates a new emptyz account
    pub fn new(name: String) -> Self {
        let now = current_timestamp();

        Self {
            api_version: ACCOUNT_API_VERSION,
            id: Uuid::new_v4(),
            name: SecretString::new(name),
            account_type: default_account_type(),
            website: SecretString::default(),
            username: SecretString::default(),
            password: SecretString::default(),
            notes: SecretString::default(),
            additional_fields: BTreeMap::new(),
            otpauth: None,
            created_at: now.clone(),
            modified_at: now,
        }
    }

    /// Updates the modification timestamp to the current time
    pub fn update_modified_time(&mut self) {
        self.modified_at = current_timestamp();
    }

    /// Validates account data before it is saved
    pub fn validate(&self) -> Result<()> {
        // `Anyhow` is great, so many good macros!
        ensure!(
            self.api_version == ACCOUNT_API_VERSION,
            "Unsupported account schema version"
        );
        ensure!(
            !self.name.expose_secret().trim().is_empty(),
            "Account name cannot be empty"
        );

        ensure!(self.name.len() <= 256, "Account name is too long");
        ensure!(self.account_type.len() <= 128, "Account type is too long");
        ensure!(self.website.len() <= 2048, "Website field is too long");
        ensure!(self.username.len() <= 4096, "Username is too long");
        ensure!(self.password.len() <= 16384, "Password is too long"); // Hope this is long enough
        ensure!(self.notes.len() <= 1048576, "Notes are too long");
        ensure!(
            self.additional_fields.len() <= 100,
            "Too many additional fields"
        );

        for (key, value) in &self.additional_fields {
            ensure!(
                !key.trim().is_empty(),
                "Additional field name cannot be empty"
            );

            ensure!(key.len() <= 128, "Additional field name is too long");
            ensure!(value.len() <= 65536, "Additional field value is too long");
        }
        if let Some(otpauth) = &self.otpauth {
            ensure!(otpauth.len() <= 4096, "TOTP configuration is too long");
        }

        Ok(())
    }
}
