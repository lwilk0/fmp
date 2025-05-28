//! This file provides functions to handle user input, including prompting for a value and validating choices against a list of options.

use anyhow::Error;
use log::{info, warn};
use std::str::FromStr;

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

/// Reads a line of input from the user.
///
/// # Returns
/// * `Result<String, Error>` - Returns the input as a string on success, or an error on failure.
///
/// # Errors
/// * If reading from standard input fails, an error is returned.
fn read_input() -> Result<String, Error> {
    let mut input = String::new();

    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow::anyhow!("Failed to read input from user. Error: {}", e))?;

    Ok(input.trim().to_string())
}

/// Prompts the user for input and returns the parsed value.
///
/// # Arguments
/// * `message` - A message to display to the user, prompting for input.
///
/// # Returns
/// * `Result<T, Error>` - Returns the parsed value of type `T` on success, or an error on failure.
///
/// # Errors
/// * If the input cannot be parsed into the specified type `T`, an error is returned.
pub fn input<T: FromStr>(message: &str) -> Result<T, Error> {
    info!("{}", message);

    let mut input = String::new();

    while input.trim().is_empty() {
        input = read_input()?;
    }

    input
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid input: `{}`. Please enter a valid value.", input))
}

/// Prompts the user to make a choice from a list of options.
///
/// # Arguments
/// * `message` - A message to display to the user, prompting for a choice.
/// * `choices` - A slice of string slices representing the valid choices.
///
/// # Returns
/// * `Result<String, Error>` - Returns the user's choice as a string on success, or an error on failure.
/// # Errors
/// * If the user's input does not match any of the provided choices, an error is returned until a valid choice is made.1
pub fn choice(message: &str, choices: &[&str]) -> Result<String, Error> {
    let mut user_input: String = String::new();

    while !choices.contains(&user_input.as_str()) {
        user_input = input::<String>(message)?.to_lowercase().trim().to_string();

        if ["exit", "quit"].contains(&user_input.as_str()) {
            warn!("User chose to exit.");
            return Err(anyhow::anyhow!("User chose to exit."));
        }
    }

    Ok(user_input)
}
