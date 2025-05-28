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

use crate::password::*;

#[test]
fn test_generate_password_length() {
    let length = 12;
    let password = generate_password(length);

    assert_eq!(password.len(), length);
}

#[test]
fn test_calculate_entropy() {
    let password = String::from("P@ssw0rd123");
    let (entropy, rating) = calculate_entropy(&password);

    assert!(entropy > 0.0);
    assert_eq!(rating, "Strong");
}

#[test]
fn test_calculate_entropy_only_lowercase() {
    let password = String::from("password");
    let (entropy, rating) = calculate_entropy(&password);

    assert!(entropy > 0.0);
    assert_eq!(rating, "Weak");
}

#[test]
fn test_calculate_entropy_only_uppercase() {
    let password = String::from("PASSWORD");
    let (entropy, rating) = calculate_entropy(&password);

    assert!(entropy > 0.0);
    assert_eq!(rating, "Weak");
}

#[test]
fn test_calculate_entropy_only_digits() {
    let password = String::from("12345678");
    let (entropy, rating) = calculate_entropy(&password);

    assert!(entropy > 0.0);
    assert_eq!(rating, "Very Weak");
}

#[test]
fn test_calculate_entropy_only_punctuation() {
    let password = String::from("!@#$%^&*()");
    let (entropy, rating) = calculate_entropy(&password);

    assert!(entropy > 0.0);
    assert_eq!(rating, "Weak");
}
