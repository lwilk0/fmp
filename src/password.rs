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
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::thread_local;

/// Generates a random password of a specified length using ASCII characters.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the vault name and recipient.
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

    let mut base: HashSet<char> = pool.chars().collect();
    let include: HashSet<char> = app.consider_characters.chars().collect();
    let exclude: HashSet<char> = app.ignore_characters.chars().collect();

    for ch in &exclude {
        base.remove(ch);
    }
    for &ch in &include {
        base.insert(ch);
    }

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

/// Calculates the entropy of a given password based on its character frequency distribution,
/// then applies penalties for common weak patterns and passwords.
///
/// Note: Shannon entropy of the observed string is not a pure strength metric, but we
/// subtract penalties for known weak patterns to avoid overestimating human-chosen passwords.
///
/// # Arguments
/// * `password` - The password for which to calculate entropy.
///
/// # Returns
/// * A tuple containing the calculated entropy as a `f64` and a rating as a string slice.
pub fn calculate_shannon_entropy(password: &str) -> (f64, &'static str) {
    #[allow(clippy::cast_precision_loss)]
    let len = password.chars().count() as f64;
    let mut freq = HashMap::new();

    for c in password.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }

    let mut entropy = 0.0;
    for count in freq.values() {
        let p = f64::from(*count) / len;
        entropy -= p * p.log2();
    }
    entropy *= len;

    let penalty: f64 = pattern_penalty_bits(password);
    let entropy: f64 = (entropy - penalty).max(0.0);

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

fn pattern_penalty_bits(password: &str) -> f64 {
    if password.is_empty() {
        return 0.0;
    }

    let mut penalty = 0.0;

    let lower = password.to_lowercase();
    let norm = normalize_leetspeak(&lower);
    let rev: String = lower.chars().rev().collect();

    if COMMON_PASSWORDS.iter().any(|&p| p == lower) {
        penalty += 60.0;
    }
    if COMMON_PASSWORDS.iter().any(|&p| p == norm) {
        penalty += 40.0;
    }
    if COMMON_PASSWORDS.iter().any(|&p| p == rev) {
        penalty += 30.0;
    }

    if contains_common_substring(&lower, 4) {
        penalty += 12.0;
    }
    if contains_common_substring(&norm, 4) {
        penalty += 10.0;
    }

    if is_repeating_char(&lower) {
        penalty += 40.0;
    }

    if contains_ordered_sequence(&lower, 4) {
        penalty += 15.0;
    }

    if contains_keyboard_sequence(&lower, 4) {
        penalty += 15.0;
    }

    if contains_year(&lower) {
        penalty += 10.0;
    }

    if contains_any_token(&lower, &COMMON_MONTHS_DAYS, 3) {
        penalty += 10.0;
    }

    let (has_l, has_u, has_d, has_s) = classify_chars(password);
    let classes = u8::from(has_l) + u8::from(has_u) + u8::from(has_d) + u8::from(has_s);
    let len = password.chars().count();
    if classes <= 1 {
        penalty += 20.0;
    } else if classes == 2 && len <= 10 {
        penalty += 10.0;
    }

    #[allow(clippy::unnecessary_cast)]
    (penalty as f64).clamp(0.0_f64, 100.0_f64)
}

fn normalize_leetspeak(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        let mapped = match ch {
            '0' => 'o',
            '1' => 'l',
            '3' => 'e',
            '4' | '@' => 'a',
            '5' | '$' => 's',
            '6' | '9' => 'g',
            '7' | '+' => 't',
            '8' => 'b',
            '!' => 'i',
            _ => ch,
        };
        out.push(mapped);
    }
    out
}

fn is_repeating_char(s: &str) -> bool {
    let mut it = s.chars();
    if let Some(first) = it.next() {
        it.all(|c| c == first)
    } else {
        false
    }
}

fn contains_year(s: &str) -> bool {
    let mut run: Vec<char> = Vec::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            run.push(ch);
            if run.len() >= 4 {
                for i in 0..=run.len().saturating_sub(4) {
                    let y: String = run[i..i + 4].iter().collect();
                    if let Ok(val) = y.parse::<u32>() {
                        if (1900..=2099).contains(&val) {
                            return true;
                        }
                    }
                }
            }
        } else {
            run.clear();
        }
    }
    false
}

fn contains_any_token(hay: &str, tokens: &[&str], min_len: usize) -> bool {
    tokens.iter().any(|t| t.len() >= min_len && hay.contains(t))
}

fn contains_common_substring(hay: &str, min_len: usize) -> bool {
    COMMON_PASSWORDS
        .iter()
        .any(|p| p.len() >= min_len && hay.contains(p))
}

fn contains_keyboard_sequence(s: &str, min_len: usize) -> bool {
    for row in &COMMON_KEYBOARD_ROWS {
        let row_chars: Vec<char> = row.chars().collect();
        let row_len = row_chars.len();
        for k in min_len..=row_len {
            for i in 0..=row_len - k {
                let sub: String = row_chars[i..i + k].iter().collect();
                if s.contains(&sub) {
                    return true;
                }
                let rev: String = sub.chars().rev().collect();
                if s.contains(&rev) {
                    return true;
                }
            }
        }
    }
    false
}

fn contains_ordered_sequence(s: &str, min_len: usize) -> bool {
    if min_len <= 1 {
        return false;
    }
    let mut inc_run: usize = 1;
    let mut dec_run: usize = 1;
    let mut prev: Option<char> = None;
    for ch in s.chars() {
        if !ch.is_ascii_alphanumeric() {
            inc_run = 1;
            dec_run = 1;
            prev = None;
            continue;
        }
        if let Some(p) = prev {
            let diff = (ch as i32) - (p as i32);
            if diff == 1 {
                inc_run += 1;
            } else {
                inc_run = 1;
            }
            if diff == -1 {
                dec_run += 1;
            } else {
                dec_run = 1;
            }
            if inc_run >= min_len || dec_run >= min_len {
                return true;
            }
        }
        prev = Some(ch);
    }
    false
}

fn classify_chars(s: &str) -> (bool, bool, bool, bool) {
    let mut l = false;
    let mut u = false;
    let mut d = false;
    let mut sym = false;
    for ch in s.chars() {
        if ch.is_ascii_lowercase() {
            l = true;
        } else if ch.is_ascii_uppercase() {
            u = true;
        } else if ch.is_ascii_digit() {
            d = true;
        } else {
            sym = true;
        }
    }
    (l, u, d, sym)
}

const COMMON_KEYBOARD_ROWS: [&str; 4] = ["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm"];

const COMMON_MONTHS_DAYS: [&str; 42] = [
    // months
    "jan",
    "january",
    "feb",
    "february",
    "mar",
    "march",
    "apr",
    "april",
    "may",
    "jun",
    "june",
    "jul",
    "july",
    "aug",
    "august",
    "sep",
    "sept",
    "september",
    "oct",
    "october",
    "nov",
    "november",
    "dec",
    "december",
    // weekdays
    "mon",
    "monday",
    "tue",
    "tues",
    "tuesday",
    "wed",
    "weds",
    "wednesday",
    "thu",
    "thur",
    "thurs",
    "thursday",
    "fri",
    "friday",
    "sat",
    "saturday",
    "sun",
    "sunday",
];

const COMMON_PASSWORDS: &[&str] = &[
    "123456",
    "password",
    "123456789",
    "12345",
    "12345678",
    "qwerty",
    "1234567",
    "111111",
    "123123",
    "abc123",
    "password1",
    "1234",
    "iloveyou",
    "1q2w3e4r",
    "000000",
    "qwertyuiop",
    "monkey",
    "dragon",
    "letmein",
    "696969",
    "shadow",
    "master",
    "666666",
    "qwerty123",
    "football",
    "welcome",
    "admin",
    "princess",
    "login",
    "solo",
    "passw0rd",
    "starwars",
    "hello",
    "freedom",
    "whatever",
    "qazwsx",
    "trustno1",
    "zaq12wsx",
    "password123",
    "batman",
    "superman",
    "121212",
    "flower",
    "hottie",
    "loveme",
    "photoshop",
    "adobe123",
    "123qwe",
    "qwe123",
    "qwerty1",
    "q1w2e3r4",
    "1qaz2wsx",
    "baseball",
    "1qazxsw2",
    "asdfgh",
    "asdfghjkl",
    "asdf",
    "qazwsxedc",
    "pokemon",
    "football1",
    "charlie",
    "donald",
    "jennifer",
    "michelle",
    "nicole",
    "ashley",
    "sunshine",
    "michael",
    "jordan",
    "harley",
    "ginger",
    "summer",
    "taylor",
    "liverpool",
    "chelsea",
    "arsenal",
    "manchester",
    "q1w2e3",
    "aa123456",
    "abc123456",
    "1234567890",
    "qwerty12345",
    "1q2w3e",
    "temp123",
    "welcome1",
    "admin123",
    "root",
    "toor",
    "letmein1",
    "pass123",
    "pass1234",
    "pass12345",
    "love",
    "lovely",
    "secret",
    "secret1",
    "password!",
    "password$",
    "p@ssw0rd",
    "p@ssword",
    "p@55w0rd",
    "p4ssw0rd",
    "p@ssw0rd1",
    "iloveyou1",
    "qwert",
    "qwertyui",
    "qwertyu",
    "qwerty12",
    "qwerty1234",
    "qwerty!",
    "123321",
    "654321",
    "7777777",
    "87654321",
    "987654321",
    "31415926",
    "password2",
    "112233",
    "q1w2e3r4t5",
    "1q2w3e4r5t",
    "qwertyqwerty",
    "letmein123",
    "adminadmin",
    "qazxsw",
    "zaqxsw",
    "test",
    "testing",
    "test123",
    "test1",
    "password0",
    "welcome123",
    "hello123",
    "dragon123",
    "monkey123",
    "superman1",
    "batman1",
    "baseball1",
    "football2",
    "soccer",
    "soccer1",
    "hockey",
    "hockey1",
    "basketball",
    "basketball1",
    "computer",
    "computer1",
    "internet",
    "internet1",
    "qweasd",
    "asdasd",
    "asd123",
    "asd123456",
    "abc321",
    "abcd1234",
    "abcd123",
    "myspace1",
    "fuckyou",
    "fuckyou1",
    "qweasd123",
    "iloveu",
    "iloveu2",
    "letmein!",
    "passw0rd1",
    "login123",
    "qazwsx123",
    "11111111",
    "222222",
    "333333",
    "444444",
    "555555",
    "888888",
    "999999",
    "q1w2e3r4t5y6",
    "1q2w3e4r",
    "12341234",
    "12121212",
    "love123",
    "sexy",
    "sexy123",
    "blink182",
    "mustang",
    "honda",
    "mercedes",
    "chevrolet",
    "ferrari",
    "porsche",
    "volvo",
    "ford",
    "tesla",
    "nissan",
    "toyota",
    "qweqwe",
    "qwe12345",
    "asdf1234",
    "asdf123",
    "1qaz2wsx3edc",
    "!qaz2wsx",
    "admin1",
    "rootroot",
    "letmein2",
    "passw0rd!",
    "user",
    "user123",
    "welcome!",
    "welcome2",
    "qwert1",
    "qwert12",
    "qwert123",
    "qwert1234",
    "pass!",
    "pass!!",
    "pass$",
    "password@",
    "pass@123",
    "pass@word1",
];

thread_local! {
    static LAST_STRENGTH: RefCell<Option<(u64, f64, &'static str)>> = const { RefCell::new(None) };
}

fn password_cache_hash(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// Draws a password strength meter in the UI (cached per thread by the last seen password hash).
///
/// The calculation is cached across frames and re-used as long as the input password bytes
/// are unchanged. Only a 64-bit hash is stored to avoid retaining the plaintext in memory.
///
/// # Arguments
/// * `ui` - The egui UI context.
/// * `password` - The password string to evaluate.
pub fn password_strength_meter(ui: &mut egui::Ui, password: &str) {
    let hash = password_cache_hash(password);

    let (entropy, rating) = LAST_STRENGTH.with(|cell| {
        if let Some((h, e, r)) = *cell.borrow() {
            if h == hash {
                return (e, r);
            }
        }
        let (e, r) = calculate_shannon_entropy(password);
        *cell.borrow_mut() = Some((hash, e, r));
        (e, r)
    });

    #[allow(clippy::cast_possible_truncation)]
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
