//! Account data model and related functionality.

/*
Copyright (C) 2025  Luke Wilkinson

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::security::SecurePassword;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a comprehensive account with all fields supported by the GUI
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub name: String,
    pub account_type: String,
    pub website: String,
    pub username: String,
    pub password: SecurePassword,
    pub notes: String,
    pub additional_fields: HashMap<String, String>,
    pub created_at: String,
    pub modified_at: String,
}

impl Default for Account {
    fn default() -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            name: String::new(),
            account_type: "Password Account".to_string(),
            website: String::new(),
            username: String::new(),
            password: SecurePassword::empty(),
            notes: String::new(),
            additional_fields: HashMap::new(),
            created_at: now.clone(),
            modified_at: now,
        }
    }
}

impl Account {
    /// Creates a new account with the specified name
    pub fn new(name: String) -> Self {
        let mut account = Self::default();
        account.name = name;
        account
    }

    /// Updates the modified timestamp to the current time
    pub fn update_modified_time(&mut self) {
        self.modified_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }
}
