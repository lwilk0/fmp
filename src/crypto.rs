use anyhow::Error;
use gpgme::Context;
use rpassword::read_password;
use zeroize::Zeroize;

/// Takes a variable and encrypts it for a specific recipient.
/// This is for securely passing around passwords or other sensitive data.
///
/// # Arguments
/// * `ctx` - A mutable reference to the GPGME context.
/// * `data` - A mutable reference to the data to be encrypted.
/// * `recipient` - The recipient's identifier (e.g., email or key ID) for whom the data is encrypted.
///
/// # Returns
/// * `Result<Vec<u8>, Error>` - Returns the encrypted data as a vector of bytes on success, or an error on failure.
///
/// # Errors
/// * If the recipient cannot be found in the context, or if encryption fails, an error is returned.
pub fn encrypt_variable(
    ctx: &mut Context,
    data: &mut Vec<u8>,
    recipient: &str,
) -> Result<Vec<u8>, Error> {
    let recipient_key = ctx
        .get_key(recipient)
        .map_err(|e| anyhow::anyhow!("Failed to find recipient {}. Error: {}", recipient, e))?;

    let mut encrypted_data = Vec::new();

    ctx.encrypt(&[recipient_key], &mut *data, &mut encrypted_data)?;

    data.zeroize();

    Ok(encrypted_data)
}

/// Takes encrypted data and decrypts it, returning the result as a string.
///
/// # Arguments
/// * `ctx` - A mutable reference to the GPGME context.
/// * `encrypted_data` - A slice of bytes containing the encrypted data.
///
/// # Returns
/// * `Result<Vec<u8>, Error>` - Returns the decrypted data as a vector of bytes on success, or an error on failure.
///
/// # Errors
/// * If decryption fails or if the decrypted data cannot be converted to a UTF-8 string, an error is returned.
pub fn decrypt_variable(ctx: &mut Context, encrypted_data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut decrypted_data = Vec::new();

    ctx.decrypt(encrypted_data, &mut decrypted_data)?;

    Ok(decrypted_data)
}

/// Prompts the user for a password and encrypts it for a specific recipient.
///
/// # Arguments
/// * `message` - A message to display to the user when prompting for the password.
/// * `ctx` - A mutable reference to the GPGME context.
/// * `recipient` - The recipient's identifier (e.g., email or key ID) for whom the password is encrypted.
///
/// # Returns
/// * `Result<Vec<u8>, Error>` - Returns the encrypted password as a vector of bytes on success, or an error on failure.
///
/// # Errors
/// * If reading the password fails, or if encryption fails, an error is returned.
pub fn securely_retrieve_password(
    message: &str,
    ctx: &mut Context,
    recipient: &str,
) -> Result<Vec<u8>, Error> {
    println!("{}", message);

    let password =
        read_password().map_err(|e| anyhow::anyhow!("Failed to read password: {}", e))?;

    let encrypted_password = encrypt_variable(ctx, &mut password.into_bytes(), recipient)?;

    Ok(encrypted_password)
}

#[cfg(test)]
#[path = "tests/crypto_tests.rs"]
mod crypto_tests;
