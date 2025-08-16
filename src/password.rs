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
use rand::prelude::IndexedRandom;
use rand::rng;
use secrecy::SecretBox;
use std::collections::{HashMap, HashSet};

/// Generates a random password of a specified length using ASCII characters.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the vault name and recipient.
pub fn generate_password(app: &mut FmpApp) {
    let mut pool = String::new();
    if app.selections[0] {
        pool.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if app.selections[1] {
        pool.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if app.selections[2] {
        pool.push_str("0123456789");
    }
    if app.selections[3] {
        pool.push_str("!\"#%&'()*+,-./:;<=>?@[\\]^_`{|}-");
    }
    if app.selections[4] {
        pool.push(' ');
    }
    if app.selections[5] {
        pool.push_str("áÁàÀâÂäÄãÃåÅæÆçÇéÉèÈêÊëËíÍìÌîÎïÏñÑóÓòÒôÔöÖõÕøØœŒßúÚùÙûÛüÜ");
    }

    // HashSet is faster for dedupe/exclude/add ops
    let mut base: HashSet<char> = pool.chars().collect();
    let add_set: HashSet<char> = app.consider_characters.chars().collect();
    let retain_set: HashSet<char> = app.ignore_characters.chars().collect();

    for ch in &retain_set {
        base.remove(ch);
    }
    // ...then ensure all "consider" chars are present (overrides ignore).
    for ch in &add_set {
        base.insert(*ch);
    }

    base.extend(add_set.iter());

    let pool_vec: Vec<char> = base.into_iter().collect();

    if !pool_vec.is_empty() {
        app.generated_password = SecretBox::new(Box::new(
            (0..app.password_length)
                .map(|_| *pool_vec.choose(&mut rng()).unwrap())
                .collect::<String>()
                .into_bytes(),
        ));
    }
}

/// Calculates the entropy of a given password based on its length and character pool.
///
/// # Arguments
/// * "password" - The password for which to calculate entropy.
///
/// # Returns
/// * "(f64, &str)" - Returns a tuple containing the calculated entropy as a "f64" and a rating as a string slice.
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
/// * "ui" - The egui UI context.
/// * "password" - The password string to evaluate.
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
