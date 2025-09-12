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
    /// Creates a new secure clipboard string
    pub(crate) fn new(clipboard_content: String) -> Self {
        // Lock the clipboard content in memory to prevent swapping
        lock_memory(clipboard_content.as_bytes());
        Self {
            inner_content: clipboard_content,
        }
    }

    #[allow(dead_code)] // Used for testing only as of current
    /// Provides controlled access to the clipboard content
    pub fn with_exposed<F, R>(&self, operation_function: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        // Add some timing obfuscation to prevent timing attacks
        let operation_start_time = std::time::Instant::now();

        let result = operation_function(&self.inner_content);

        // Ensure minimum execution time to prevent timing analysis
        let minimum_duration = std::time::Duration::from_millis(5);
        let elapsed_time = operation_start_time.elapsed();
        if elapsed_time < minimum_duration {
            std::thread::sleep(minimum_duration - elapsed_time);
        }

        result
    }
}

impl Drop for SecureClipboardString {
    fn drop(&mut self) {
        // Unlock memory before zeroization
        unlock_memory(self.inner_content.as_bytes());

        // Add extra security measures for secure overwriting
        unsafe {
            let content_bytes = self.inner_content.as_mut_vec();
            secure_overwrite(content_bytes);
        }

        // Add a small delay to make timing attacks harder
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

impl std::ops::Deref for SecureClipboardString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner_content
    }
}
