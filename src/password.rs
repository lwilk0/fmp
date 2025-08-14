//! This module provides password generation and rating.

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
use std::collections::HashMap;

/// Generates a random password of a specified length using ASCII characters.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the vault name and recipient.
pub fn generate_password(app: &mut FmpApp) {
    app.userpass.password = SecretBox::new(Box::new(
        (0..app.password_length)
            .map(|_| (rng().random_range(33..127) as u8))
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
pub fn calculate_shannon_entropy(password: &str) -> (f64, &str) {
    let len = password.chars().count() as f64;
    let mut freq = HashMap::new();

    for c in password.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }

    let mut entropy = 0.0;
    for count in freq.values() {
        let p = *count as f64 / len;
        entropy -= p * p.log2();
    }
    entropy *= len; // Total entropy in bits

    let rating = if entropy <= 28.0 || entropy.is_nan() {
        "Very Weak"
    } else if entropy <= 35.0 {
        "Weak"
    } else if entropy <= 59.0 {
        "Okay"
    } else if entropy <= 127.0 {
        "Strong"
    } else {
        "Very Strong"
    };

    (entropy, rating)
}

/// Draws a password strength meter in the UI.
///
/// # Arguments
/// * `ui` - The egui UI context.
/// * `password` - The password string to evaluate.
pub fn password_strength_meter(ui: &mut egui::Ui, password: &str) {
    let (entropy, rating) = calculate_shannon_entropy(password);

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
        ui.label(format!("{entropy:.2} bits ({rating})"));
    });
}
