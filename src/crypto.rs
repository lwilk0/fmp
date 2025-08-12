//! This module provides functions for encrypting and decrypting variables using GPGME.

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

use crate::{content::show_password_button, gui::FmpApp, password::password_strength_meter};
use libc::c_void;
use secrecy::{ExposeSecret, SecretBox};
use zeroize::Zeroize;

/// Securely retrieves a password from the user interface.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing user credentials.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the UI.
/// * `text` - A string slice containing the label text to display alongside the password input field.
pub fn securely_retrieve_password(app: &mut FmpApp, ui: &mut egui::Ui, text: &str) {
    let mut password =
        String::from_utf8(app.userpass.password.expose_secret().clone()).unwrap_or_default();

    ui.horizontal(|ui| {
        ui.label(text);

        let response = if app.show_password {
            ui.add(egui::TextEdit::singleline(&mut password).desired_width(200.0))
        } else {
            ui.add(
                egui::TextEdit::singleline(&mut password)
                    .password(true)
                    .desired_width(200.0),
            )
        };

        if app.show_password {
            app.show_password = show_password_button(app.show_password, ui, "Hide");
        } else {
            app.show_password = show_password_button(app.show_password, ui, "Show");
        }

        if response.changed() {
            let mut pw_bytes = password.as_bytes().to_vec();
            app.userpass.password = SecretBox::new(Box::new(pw_bytes.clone()));
            pw_bytes.zeroize();
        }
    });

    if !password.is_empty() && app.show_password {
        password_strength_meter(ui, &password);
    }

    password.zeroize();
}

/// Locks the memory of the provided data to prevent it from being swapped to disk.
///
/// # Arguments
/// * `data` - A slice of bytes representing the data to be locked in memory.
pub fn lock_memory(data: &[u8]) {
    #[cfg(unix)]
    unsafe {
        libc::mlock(data.as_ptr() as *const c_void, data.len());
    }

    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Memory::VirtualLock;
        VirtualLock(data.as_ptr() as *const c_void, data.len());
    }
}
