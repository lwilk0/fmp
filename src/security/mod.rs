//! Security module providing memory-safe password handling and secure operations.
//!
//! This module contains types and functions for handling sensitive data securely,
//! including memory locking, secure zeroization, and timing attack prevention.

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

pub mod secure_clipboard;
pub mod secure_password;

pub use secure_clipboard::SecureClipboardString;
pub use secure_password::SecurePassword;
