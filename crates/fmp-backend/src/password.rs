//! Password stuff. Not liking this code

use crate::secret::SecretString;
use anyhow::{Context, Result, anyhow, bail};
use rand_core::{OsRng, TryRngCore};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};
use zeroize::Zeroizing;

pub const MAX_PASSWORD_LENGTH: usize = 4096;
pub const MAX_CHARACTER_POOL: usize = 16_384;

const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMBERS: &str = "0123456789";

pub const SYMBOLS: &str = "!\"#%&'()*+,-./:;<=>?@[\\]^_`{|}$~";

const EXTENDED: &str = "áÁàÀâÂäÄãÃåÅæÆçÇéÉèÈêÊëËíÍìÌîÎïÏñÑóÓòÒôÔöÖõÕøØœŒßúÚùÙûÛüÜ";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PasswordConfig {
    pub length: usize,
    pub include_lowercase: bool,
    pub include_uppercase: bool,
    pub include_numbers: bool,
    pub include_symbols: bool,
    pub include_spaces: bool,
    pub include_extended: bool,
    pub additional_characters: String,
    pub excluded_characters: String,
    pub require_each_enabled_class: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: 16,
            include_lowercase: true,
            include_uppercase: true,
            include_numbers: true,
            include_symbols: false,
            include_spaces: false,
            include_extended: false,
            additional_characters: String::new(),
            excluded_characters: String::new(),
            require_each_enabled_class: false,
        }
    }
}

impl PasswordConfig {
    pub fn validate(&self) -> Result<()> {
        if self.length == 0 {
            bail!("Password length must be greater than zero");
        }

        if self.length > MAX_PASSWORD_LENGTH {
            bail!("Password length cannot exceed {MAX_PASSWORD_LENGTH} characters");
        }

        if self.additional_characters.chars().count() > MAX_CHARACTER_POOL {
            bail!("Too many additional characters");
        }

        if self.excluded_characters.chars().count() > MAX_CHARACTER_POOL {
            bail!("Too many excluded characters");
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PasswordStrengthLevel {
    VeryWeak,
    Weak,
    Fair,
    Good,
    Strong,
}

impl PasswordStrengthLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::VeryWeak => "Very weak",
            Self::Weak => "Weak",
            Self::Fair => "Fair",
            Self::Good => "Good",
            Self::Strong => "Strong",
        }
    }
}

/// Password strength assessor
///
/// `score` is a 0-100 mapping of zxcvbn's five strength levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PasswordStrength {
    pub score: u8,
    pub level: PasswordStrengthLevel,
    pub estimated_guesses_log10: f64,
    pub warning: Option<String>,
    pub suggestions: Vec<String>,
}

impl Default for PasswordStrength {
    fn default() -> Self {
        Self {
            score: 0,
            level: PasswordStrengthLevel::VeryWeak,
            estimated_guesses_log10: 0.0,
            warning: None,
            suggestions: Vec::new(),
        }
    }
}

impl PasswordStrength {
    /// zxcvbn considers scores below 3 too weak.
    pub fn is_acceptable(&self) -> bool {
        self.score >= 75
    }

    pub fn description(&self) -> &'static str {
        self.level.label()
    }
}

/// Generates a password
pub struct GeneratedPassword {
    secret: Zeroizing<String>,
    pub strength: PasswordStrength,
    pub length: usize,
    pub pool_size: usize,
}

impl GeneratedPassword {
    pub fn with_exposed<F, R>(&self, operation: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        operation(self.secret.as_str())
    }

    pub fn into_secret(mut self) -> SecretString {
        let password = std::mem::take(&mut *self.secret);
        SecretString::new(password)
    }
}

impl fmt::Debug for GeneratedPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeneratedPassword")
            .field("password", &"[REDACTED]")
            .field("strength", &self.strength)
            .field("length", &self.length)
            .field("pool_size", &self.pool_size)
            .finish()
    }
}

/// Adds needed characters to pool
fn append_character_set(
    source: &str,
    excluded: &HashSet<char>,
    positions: &mut HashMap<char, usize>,
    pool: &mut Vec<char>,
) -> Vec<usize> {
    let mut class_indices = Vec::new();
    let mut seen_indices = HashSet::new();

    for character in source.chars() {
        if excluded.contains(&character) {
            continue;
        }

        let index = match positions.get(&character).copied() {
            Some(index) => index,
            None => {
                let index = pool.len();
                pool.push(character);
                positions.insert(character, index);
                index
            }
        };

        if seen_indices.insert(index) {
            class_indices.push(index);
        }
    }

    class_indices
}

// Should fix the stupid fn sig, but its funny
fn build_pool(config: &PasswordConfig) -> Result<(Vec<char>, Vec<(&'static str, Vec<usize>)>)> {
    config.validate()?;

    let excluded: HashSet<char> = config.excluded_characters.chars().collect();
    let mut positions = HashMap::new();
    let mut pool = Vec::new();
    let mut class_sets = Vec::new();

    let configured_sets = [
        (config.include_lowercase, "lowercase", LOWERCASE),
        (config.include_uppercase, "uppercase", UPPERCASE),
        (config.include_numbers, "numbers", NUMBERS),
        (config.include_symbols, "symbols", SYMBOLS),
        (config.include_spaces, "spaces", " "),
        (config.include_extended, "extended", EXTENDED),
    ];

    for (enabled, name, characters) in configured_sets {
        if enabled {
            let indices = append_character_set(characters, &excluded, &mut positions, &mut pool);
            class_sets.push((name, indices));
        }
    }

    append_character_set(
        &config.additional_characters,
        &excluded,
        &mut positions,
        &mut pool,
    );

    if pool.is_empty() {
        bail!("No characters are available after applying exclusions");
    }

    if pool.len() > MAX_CHARACTER_POOL {
        bail!("The resulting character pool is too large");
    }

    if config.require_each_enabled_class {
        for (name, indices) in &class_sets {
            if indices.is_empty() {
                bail!("The enabled {name} character class has no available characters");
            }
        }

        if config.length < class_sets.len() {
            bail!(
                "Password length must be at least {} to include every enabled class",
                class_sets.len()
            );
        }
    }

    Ok((pool, class_sets))
}

/// selects an index using the operating system's random source
fn random_index(rng: &mut OsRng, upper: usize) -> Result<usize> {
    if upper == 0 {
        bail!("Cannot select from an empty range");
    }

    if upper == 1 {
        return Ok(0);
    }

    let upper = u64::try_from(upper).context("Character pool is too large for random selection")?;

    let accepted_range = u64::MAX - (u64::MAX % upper);

    loop {
        let value = rng
            .try_next_u64()
            .map_err(|error| anyhow!("Operating-system random source failed: {error}"))?;

        if value < accepted_range {
            return usize::try_from(value % upper)
                .context("Random character index is out of range");
        }
    }
}

/// Generate a password using OS-provided random data.
///
/// Duplicate characters in the configured pool are removed, so adding the same character repeatedly does not weight its probability.
pub fn generate_password(config: &PasswordConfig) -> Result<GeneratedPassword> {
    let (pool, class_sets) = build_pool(config)?;

    let mut rng = OsRng;
    let mut selected_indices = Zeroizing::new(Vec::<usize>::with_capacity(config.length));

    if config.require_each_enabled_class {
        for (name, indices) in &class_sets {
            // build_pool has already checked that these are non-empty.
            let index = random_index(&mut rng, indices.len())
                .with_context(|| format!("Failed to select a {name} character"))?;

            selected_indices.push(indices[index]);
        }
    }

    while selected_indices.len() < config.length {
        selected_indices.push(random_index(&mut rng, pool.len())?);
    }

    // Shuffle the required characters and the remaining random characters so the required classes do not always appear at the beginning.
    for index in (1..selected_indices.len()).rev() {
        let other = random_index(&mut rng, index + 1)?;
        selected_indices.swap(index, other);
    }

    let mut output = Zeroizing::new(String::with_capacity(config.length.saturating_mul(4)));

    for index in selected_indices.iter() {
        output.push(pool[*index]);
    }

    let strength = estimate_password_strength(output.as_str(), &[]);

    Ok(GeneratedPassword {
        secret: output,
        strength,
        length: config.length,
        pool_size: pool.len(),
    })
}

/// Estimate the strength of an existing password
pub fn estimate_password_strength(password: &str, user_inputs: &[&str]) -> PasswordStrength {
    if password.is_empty() {
        return PasswordStrength::default();
    }

    let entropy = zxcvbn::zxcvbn(password, user_inputs);

    let raw_score = u8::from(entropy.score()).min(4);
    let score = raw_score * 25;

    let level = match raw_score {
        0 => PasswordStrengthLevel::VeryWeak,
        1 => PasswordStrengthLevel::Weak,
        2 => PasswordStrengthLevel::Fair,
        3 => PasswordStrengthLevel::Good,
        _ => PasswordStrengthLevel::Strong,
    };

    let (warning, suggestions) = match entropy.feedback() {
        Some(feedback) => (
            feedback.warning().map(|warning| warning.to_string()),
            feedback
                .suggestions()
                .iter()
                .map(ToString::to_string)
                .collect(),
        ),
        None => (None, Vec::new()),
    };

    PasswordStrength {
        score,
        level,
        estimated_guesses_log10: entropy.guesses_log10(),
        warning,
        suggestions,
    }
}

/// Score-only convenience function.
pub fn calculate_password_strength(password: &str) -> u8 {
    estimate_password_strength(password, &[]).score
}

/// short label.
pub fn get_strength_description(strength: u8) -> &'static str {
    match strength {
        0..=25 => "Weak",
        26..=50 => "Fair",
        51..=75 => "Good",
        _ => "Strong",
    }
}

// Most tests LLM generated, I would LOVE to write them manually but im.... busy
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PasswordConfig::default();

        assert_eq!(config.length, 16);
        assert!(config.include_lowercase);
        assert!(config.include_uppercase);
        assert!(config.include_numbers);
        assert!(!config.include_symbols);
        assert!(!config.include_spaces);
        assert!(!config.include_extended);
        assert!(!config.require_each_enabled_class);
    }

    #[test]
    fn test_generated_password_has_requested_length() {
        let config = PasswordConfig {
            length: 48,
            ..PasswordConfig::default()
        };

        let generated = generate_password(&config).unwrap();

        assert_eq!(generated.length, 48);
        generated.with_exposed(|password| {
            assert_eq!(password.chars().count(), 48);
        });
    }

    #[test]
    fn test_excluded_characters_are_never_generated() {
        let config = PasswordConfig {
            length: 100,
            excluded_characters: "abcdefghijklmnopqrstuvwxyz0123456789".to_owned(),
            ..PasswordConfig::default()
        };

        let generated = generate_password(&config).unwrap();

        generated.with_exposed(|password| {
            assert!(password.chars().all(|c| c.is_ascii_uppercase()));
        });
    }

    #[test]
    fn test_required_character_classes_are_present() {
        let config = PasswordConfig {
            length: 3,
            require_each_enabled_class: true,
            ..PasswordConfig::default()
        };

        let generated = generate_password(&config).unwrap();

        generated.with_exposed(|password| {
            assert!(password.chars().any(|c| c.is_ascii_lowercase()));
            assert!(password.chars().any(|c| c.is_ascii_uppercase()));
            assert!(password.chars().any(|c| c.is_ascii_digit()));
        });
    }

    #[test]
    fn test_required_classes_need_enough_length() {
        let config = PasswordConfig {
            length: 2,
            require_each_enabled_class: true,
            ..PasswordConfig::default()
        };

        assert!(generate_password(&config).is_err());
    }

    #[test]
    fn test_empty_pool_is_rejected() {
        let config = PasswordConfig {
            include_lowercase: false,
            include_uppercase: false,
            include_numbers: false,
            include_symbols: false,
            include_spaces: false,
            include_extended: false,
            additional_characters: String::new(),
            ..PasswordConfig::default()
        };

        assert!(generate_password(&config).is_err());
    }

    #[test]
    fn test_common_password_is_scored_weak() {
        let strength = estimate_password_strength("password", &[]);

        assert_eq!(strength.score, 0);
        assert_eq!(strength.level, PasswordStrengthLevel::VeryWeak);
        assert!(!strength.is_acceptable());
    }

    #[test]
    fn test_duplicate_additional_characters_do_not_weight_pool() {
        let config = PasswordConfig {
            length: 32,
            include_lowercase: false,
            include_uppercase: false,
            include_numbers: false,
            include_symbols: false,
            include_spaces: false,
            include_extended: false,
            additional_characters: "λλλλββββ".to_owned(),
            ..PasswordConfig::default()
        };

        let generated = generate_password(&config).unwrap();

        assert_eq!(generated.pool_size, 2);
        generated.with_exposed(|password| {
            assert!(password.chars().all(|c| c == 'λ' || c == 'β'));
        });
    }

    #[test]
    fn test_config_can_be_deserialised_with_defaults() {
        let config: PasswordConfig = serde_json::from_str("{}").unwrap();

        assert_eq!(config.length, 16);
        assert!(config.include_lowercase);
    }

    #[test]
    fn test_generated_password_debug_is_redacted() {
        let generated = generate_password(&PasswordConfig::default()).unwrap();
        let debug = format!("{generated:?}");

        assert!(debug.contains("[REDACTED]"));
        assert!(debug.contains("pool_size"));
    }
}
