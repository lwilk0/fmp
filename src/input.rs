use anyhow::Error;
use std::str::FromStr;

/// Prompts the user for input and returns the parsed value.
///
/// # Arguments
/// * `message` - A message to display to the user, prompting for input.
///
/// # Returns
/// * `Result<T, Error>` - Returns the parsed value of type `T` on success, or an error if parsing fails.
///
/// # Errors
/// * If the input cannot be parsed into the specified type `T`, an error is returned.
pub fn input<T: FromStr>(message: &str) -> Result<T, Error> {
    println!("{}", message);

    let mut input = String::new();

    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Failed to parse input: {}.", input))
}

/// Prompts the user to make a choice from a list of options.
///
/// # Arguments
/// * `message` - A message to display to the user, prompting for a choice.
/// * `choices` - A slice of string slices representing the valid choices.
///
/// # Returns
/// * `Result<String, Error>` - Returns the user's choice as a string on success, or an error if the choice is invalid.
///
/// # Errors
/// * If the user's input does not match any of the provided choices, an error is returned until a valid choice is made.1
pub fn choice(message: &str, choices: &[&str]) -> Result<String, Error> {
    let mut user_input = String::new();

    while !choices.contains(&user_input.as_str()) {
        user_input = input(message)?;
    }

    Ok(user_input)
}
