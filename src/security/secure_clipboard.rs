//! Secure clipboard string implementation for temporary password exposure.

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

use crate::crypto::{lock_memory, secure_overwrite, unlock_memory};

/// A secure string wrapper specifically for clipboard operations
pub struct SecureClipboardString {
    inner: String,
}

impl SecureClipboardString {
    pub(crate) fn new(password: String) -> Self {
        lock_memory(password.as_bytes());
        Self { inner: password }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl Drop for SecureClipboardString {
    fn drop(&mut self) {
        // Unlock memory before zeroization
        unlock_memory(self.inner.as_bytes());

        // The ZeroizeOnDrop will handle the actual zeroization
        // but we add extra security measures
        unsafe {
            let bytes = self.inner.as_bytes_mut();
            secure_overwrite(bytes);
        }
    }
}

impl std::ops::Deref for SecureClipboardString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
