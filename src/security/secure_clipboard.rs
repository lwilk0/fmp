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
    inner_content: String,
}

impl SecureClipboardString {
    pub(crate) fn new(clipboard_content: String) -> Self {
        lock_memory(clipboard_content.as_bytes());
        Self {
            inner_content: clipboard_content,
        }
    }
}

impl Drop for SecureClipboardString {
    fn drop(&mut self) {
        // Unlock memory before zeroization
        unlock_memory(self.inner_content.as_bytes());

        // Add extra security measures for secure overwriting
        unsafe {
            let content_bytes = self.inner_content.as_bytes_mut();
            secure_overwrite(content_bytes);
        }
    }
}

impl std::ops::Deref for SecureClipboardString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner_content
    }
}
