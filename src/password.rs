//! This file provides password generation and entropy calculation.

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
use crate::gui::FmpApp;
use rand::{Rng, rng};
use secrecy::SecretBox;

/// Generates a random password of a specified length using ASCII characters.
///
/// # Arguments
/// * `length` - The desired length of the password.
///
/// # Returns
/// * `String` - Returns a randomly generated password as a string.
pub fn generate_password(app: &mut FmpApp) {
    app.userpass.password = SecretBox::new(Box::new(
        (0..app.password_length)
            .map(|_| (rng().random_range(33..128) as u8))
            .collect(),
    ));
}

/// Calculates the entropy of a given password based on its length and character pool.
///
/// # Arguments
/// * `password` - The password for which to calculate entropy.
///
/// # Returns
/// * `(f64, &str)` - Returns a tuple containing the calculated entropy as a `f64` and a rating as a string slice.
pub fn calculate_entropy(password: &str) -> (f64, &str) {
    let mut character_pool: u8 = 0;

    if password.chars().any(|c| c.is_ascii_lowercase()) {
        character_pool += 26;
    }

    if password.chars().any(|c| c.is_ascii_uppercase()) {
        character_pool += 26;
    }

    if password.chars().any(|c| c.is_ascii_digit()) {
        character_pool += 10;
    }

    if password.chars().any(|c| c.is_ascii_punctuation()) {
        character_pool += 32;
    }

    // FORMULA
    // L * log2(C), where L is the length of the password and C is the character pool
    let entropy = password.len() as f64 * (character_pool as f64).log2();
    let rating: &str;

    if entropy <= 35.0 {
        rating = "Very Weak"
    } else if entropy <= 59.0 {
        rating = "Weak"
    } else if entropy <= 89.0 {
        rating = "Okay"
    } else if entropy <= 119.0 {
        rating = "Strong"
    } else {
        rating = "Very Strong"
    }

    (entropy, rating)
}

/// Draws a password strength meter in the UI.
///
/// # Arguments
/// * `ui` - The egui UI context.
/// * `password` - The password string to evaluate.
pub fn password_strength_meter(ui: &mut egui::Ui, password: &str) {
    let (entropy, rating) = calculate_entropy(password);

    let length = (entropy / 150.0).clamp(0.0, 1.0) as f32;

    let (color, progress) = match rating {
        "Very Weak" => (egui::Color32::from_rgb(200, 0, 0), length),
        "Weak" => (egui::Color32::from_rgb(255, 140, 0), length),
        "Okay" => (egui::Color32::from_rgb(255, 215, 0), length),
        "Strong" => (egui::Color32::from_rgb(0, 180, 0), length),
        "Very Strong" => (egui::Color32::from_rgb(0, 220, 0), length),
        _ => (egui::Color32::GRAY, 0.0),
    };

    ui.horizontal(|ui| {
        ui.add(
            egui::ProgressBar::new(progress)
                .desired_width(120.0)
                .fill(color),
        );
        ui.label(format!("{:.2} bits ({})", entropy, rating));
    });
}

#[cfg(test)]
#[path = "tests/password_tests.rs"]
mod password_tests;
