//-- ./src/domain/tokens/token_type.rs

//! # TokenType Enum
//!
//! This module defines the `TokenType` enum, which represents the different types of tokens
//! used in the authentication service. It is used to clearly identify the intent and usage
//! context of a token throughout the system.
//!
//! ## Variants
//! - `Access`: An access token for authenticating API requests.
//! - `Refresh`: A refresh token for obtaining new access tokens.
//! - `EmailVerification`: A token used for verifying a user's email address.
//! - `PasswordReset`: A token used for resetting a user's password.
//!
//! The enum also provides utilities for formatting as a string and for generating a random variant,
//! which is useful for testing.

use rand::seq::IndexedRandom;

/// # TokenType Enum
/// 
/// Represents the type of token used in the authentication service.
///
/// This enum distinguishes between different token purposes. It is used to clearly identify 
/// the intent and usage context of a token throughout the system.
///
/// # Variants
/// - `Access`: An access token for authenticating API requests.
/// - `Refresh`: A refresh token for obtaining new access tokens.
/// - `EmailVerification`: A token used for verifying a user's email address.
/// - `PasswordReset`: A token used for resetting a user's password.
#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum TokenType {
    #[default]
    Access,
    Refresh,
    EmailVerification,
    PasswordReset,
}

/// Implements the `Display` trait for `TokenType` to provide a human-readable, lowercase string
/// representation of each variant. This is useful for logging, debugging, serialization to text-based
/// formats, or whenever you need to convert a `TokenType` to a string for display or storage.
impl std::fmt::Display for TokenType {
    /// Formats the `TokenType` as a lowercase string.
    ///
    /// # Example
    /// ```
    /// use crate::domain::tokens::TokenType;
    /// let t = TokenType::EmailVerification;
    /// assert_eq!(t.to_string(), "email_verification");
    /// println!("{}", t); // prints "email_verification"
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TokenType::Access => "access",
            TokenType::Refresh => "refresh",
            TokenType::EmailVerification => "email_verification",
            TokenType::PasswordReset => "password_reset",
        };
        write!(f, "{}", s)
    }
}

impl TokenType {
    /// Returns a randomly selected `TokenType` variant.
    ///
    /// # Example
    /// ```
    /// use crate::domain::tokens::TokenType;
    /// let random_type = TokenType::random();
    /// println!("Random token type: {}", random_type);
    /// ```
    pub fn random() -> Self {
        let variants = [
            TokenType::Access,
            TokenType::Refresh,
            TokenType::EmailVerification,
            TokenType::PasswordReset,
        ];
        let mut rng = rand::rng();
        variants.choose(&mut rng).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_outputs_expected_strings() {
        assert_eq!(TokenType::Access.to_string(), "access");
        assert_eq!(TokenType::Refresh.to_string(), "refresh");
        assert_eq!(TokenType::EmailVerification.to_string(), "email_verification");
        assert_eq!(TokenType::PasswordReset.to_string(), "password_reset");
    }

    #[test]
    fn random_returns_valid_variant() {
        // Run multiple times to increase confidence
        for _ in 0..20 {
            let t = TokenType::random();
            match t {
                TokenType::Access
                | TokenType::Refresh
                | TokenType::EmailVerification
                | TokenType::PasswordReset => (),
            }
        }
    }

    #[test]
    fn default_is_access() {
        assert_eq!(TokenType::default(), TokenType::Access);
    }
}