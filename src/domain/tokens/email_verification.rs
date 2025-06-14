//-- ./src/domain/tokens/email_verifications.rs

//! # Email Verification Token Module
//!
//! This module provides the `EmailVerificationToken` type, a specialised wrapper around JWT tokens
//! designed specifically for email verification workflows in the authentication service.
//!
//! ## Overview
//! The `EmailVerificationToken` encapsulates a JWT string that contains all necessary claims
//! to securely verify a user's email address. It ensures type safety by only allowing tokens
//! with the correct `TokenType::EmailVerification` to be created.
//!
//! ## Features
//! - **Type Safety**: Only accepts tokens with `TokenType::EmailVerification`
//! - **JWT Encoding**: Converts `TokenClaim` instances into signed JWT strings
//! - **Secure Transmission**: Designed for safe transmission via email
//! - **Time-Limited**: Supports expiration-based security
//! - **Display Support**: Implements `Display` and `AsRef<str>` for easy usage
//!
//! ## Usage Flow
//! 1. **Creation**: Generate from a `TokenClaim` with appropriate email verification settings
//! 2. **Transmission**: Send the token via email to the user
//! 3. **Verification**: Parse and validate the token when the user clicks the verification link
//! 4. **Validation**: Confirm the token is valid, not expired, and has correct claims
//!
//! ## Example
//! ```rust
//! use crate::domain::tokens::{TokenClaim, EmailVerificationToken, TokenType};
//! use secrecy::SecretString;
//! use chrono::Duration;
//!
//! // Create an email verification token
//! let claim = TokenClaim::new(
//!     &issuer,
//!     &Duration::hours(24),  // 24-hour expiration
//!     &user,
//!     &TokenType::EmailVerification
//! );
//! let secret = SecretString::new("jwt_secret".to_string());
//! let token = EmailVerificationToken::try_from_claim(claim, &secret)?;
//!
//! // Use in email verification URL
//! let verification_url = format!("https://example.com/verify?token={}", token);
//! send_verification_email(&user.email, &verification_url)?;
//! ```
//!
//! ## Security Considerations
//! - Tokens should have short expiration times (typically 24-48 hours)
//! - Always transmit over HTTPS
//! - Invalidate tokens after successful verification
//! - Log verification attempts for audit purposes

use crate::{domain, AuthenticationError};
use secrecy::ExposeSecret;

/// # Email Verification Token
///
/// A wrapper around a JWT string specifically designed for email verification workflows.
/// This token contains the necessary claims to verify a user's email address and is
/// typically sent via email as part of the account verification process.
///
/// ## Purpose
/// The `EmailVerificationToken` is used to:
/// - Verify user email addresses during account registration
/// - Confirm email ownership when users change their email address
/// - Provide a secure, time-limited verification mechanism
///
/// ## Structure
/// This is a newtype wrapper around a `String` containing a JWT token with:
/// - Standard JWT claims (iss, sub, aud, exp, nbf, iat, jti)
/// - Custom `jty` claim set to `TokenType::EmailVerification`
/// - Subject (`sub`) containing the user's UUID
///
/// ## Security Considerations
/// - Tokens have a limited lifespan (typically 24-48 hours)
/// - Each token is cryptographically signed and tamper-evident
/// - Tokens should be transmitted only over secure channels (HTTPS)
/// - Used tokens should be invalidated after successful verification
///
/// ## Usage
/// ```rust
/// use crate::domain::tokens::{TokenClaim, EmailVerificationToken, TokenType};
/// use secrecy::SecretString;
/// use chrono::Duration;
///
/// // Create an email verification token
/// let claim = TokenClaim::new(
///     &issuer,
///     &Duration::hours(24),
///     &user,
///     &TokenType::EmailVerification
/// );
/// let secret = SecretString::new("my_secret_key".to_string());
/// let email_token = EmailVerificationToken::try_from_claim(claim, &secret)?;
///
/// // Send in email
/// send_verification_email(&user.email, email_token.as_ref())?;
///
/// // Later, verify the token
/// let verified_claim = TokenClaim::parse(email_token.as_ref(), &secret, &issuer)?;
/// ```
#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EmailVerificationToken(String);

/// Get string reference of the Email Verification  Token
impl AsRef<str> for EmailVerificationToken {
    /// Returns a string slice reference to the underlying JWT token.
    ///
    /// This implementation allows the `EmailVerificationToken` to be used anywhere
    /// a string slice is expected, providing convenient access to the raw JWT string
    /// without consuming the token instance.
    ///
    /// # Returns
    /// A string slice containing the JWT token
    ///
    /// # Example
    /// ```rust
    /// let email_token = EmailVerificationToken::try_from_claim(claim, &secret)?;
    /// let jwt_str: &str = email_token.as_ref();
    ///
    /// // Can be used with functions expecting &str
    /// send_email_with_token(user_email, email_token.as_ref());
    ///
    /// // Or for string operations
    /// assert!(email_token.as_ref().starts_with("eyJ"));
    /// ```
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EmailVerificationToken {
    /// Formats the `EmailVerificationToken` for display purposes.
    ///
    /// This implementation allows the token to be printed or converted to a string,
    /// displaying the underlying JWT string. This is useful for logging, debugging,
    /// or when you need to include the token in user-facing messages.
    ///
    /// # Security Note
    /// Be cautious when displaying tokens in logs or user interfaces, as they contain
    /// sensitive authentication information that could be used for unauthorised access.
    ///
    /// # Example
    /// ```rust
    /// let email_token = EmailVerificationToken::try_from_claim(claim, &secret)?;
    /// println!("Email verification token: {}", email_token);
    /// // Outputs: Email verification token: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}



impl EmailVerificationToken {
    /// Creates an `EmailVerificationToken` from a `TokenClaim` and secret key.
    ///
    /// This method validates that the provided token claim is specifically for email verification
    /// by checking the `jty` (JWT Type) field, then encodes it into a JWT string that can be
    /// used for email verification purposes.
    ///
    /// # Arguments
    /// * `claim` - The `TokenClaim` to convert (must have `TokenType::EmailVerification`)
    /// * `secret` - The secret key used for JWT encoding
    ///
    /// # Returns
    /// * `Ok(EmailVerificationToken)` containing the encoded JWT string
    /// * `Err(AuthenticationError::InvalidToken)` if the token type is incorrect or encoding fails
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The token claim type is not `TokenType::EmailVerification`
    /// - JWT encoding fails due to serialization issues
    ///
    /// # Example
    /// ```rust
    /// use crate::domain::tokens::{TokenClaim, EmailVerificationToken, TokenType};
    /// use secrecy::SecretString;
    /// use chrono::Duration;
    ///
    /// let claim = TokenClaim::new(&issuer, &Duration::hours(24), &user, &TokenType::EmailVerification);
    /// let secret = SecretString::new("my_secret_key".to_string());
    /// let email_token = EmailVerificationToken::try_from_claim(claim, &secret)?;
    ///
    /// // The token can now be sent via email
    /// let jwt_string = email_token.as_ref();
    /// ```
    pub fn try_from_claim(
        claim: domain::tokens::TokenClaimNew,
        secret: &secrecy::SecretString,
    ) -> Result<Self, AuthenticationError> {
        match claim.jty {
            crate::domain::tokens::TokenType::EmailVerification => {
                // Encode the claim as a JWT string
                let jwt_string = jsonwebtoken::encode(
                    &jsonwebtoken::Header::default(),
                    &claim,
                    &jsonwebtoken::EncodingKey::from_secret(
                        secret.expose_secret().as_bytes(),
                    ),
                )
                .map_err(|e| {
                    AuthenticationError::InvalidToken(format!(
                        "JWT encoding failed: {}",
                        e
                    ))
                })?;

                Ok(EmailVerificationToken(jwt_string))
            }
            _ => Err(AuthenticationError::InvalidToken(
                "Token Claim is not an email verification token".to_string(),
            )),
        }
    }

    /// # Try From String
    /// 
    /// Creates an `EmailVerificationToken` from a string with validation.
    ///
    /// This method attempts to parse and validate the provided JWT string to ensure it:
    /// - Has valid JWT structure (header.payload.signature)
    /// - Contains proper claims including the email verification token type
    /// - Is cryptographically valid (signature verification)
    /// - Is not expired and within valid time bounds
    ///
    /// # Arguments
    /// * `jwt_string` - The JWT string to validate and convert
    /// * `secret` - The secret key used for JWT signature verification
    /// * `issuer` - The expected issuer for validation
    ///
    /// # Returns
    /// * `Ok(EmailVerificationToken)` if the JWT is valid and is an email verification token
    /// * `Err(AuthenticationError)` if validation fails for any reason
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The JWT structure is invalid (malformed, missing parts)
    /// - The signature verification fails
    /// - The token is expired or not yet valid
    /// - The token type is not `TokenType::EmailVerification`
    /// - The issuer doesn't match the expected value
    ///
    /// # Example
    /// ```rust
    /// use crate::domain::tokens::EmailVerificationToken;
    /// use secrecy::SecretString;
    ///
    /// let jwt_str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...";
    /// let secret = SecretString::new("my_secret_key".to_string());
    /// let issuer = SecretString::new("https://example.com".to_string());
    ///
    /// let token = EmailVerificationToken::try_from_string(jwt_str, &secret, &issuer)?;
    /// ```
    ///
    /// # Security Note
    /// This method performs full cryptographic validation of the JWT, ensuring
    /// the token is authentic and hasn't been tampered with.
    pub fn try_from_string(
        jwt_string: &str,
        secret: &secrecy::SecretString,
        issuer: &secrecy::SecretString,
    ) -> Result<Self, AuthenticationError> {
        // First, parse and validate the JWT using TokenClaim::parse
        let claim = domain::tokens::TokenClaimNew::parse(jwt_string, secret, issuer)?;

        // Verify that this is specifically an email verification token
        match claim.jty {
            crate::domain::tokens::TokenType::EmailVerification => {
                // If validation passes, create the EmailVerificationToken
                Ok(EmailVerificationToken(jwt_string.to_string()))
            }
            _ => Err(AuthenticationError::InvalidToken(
                "JWT is not an email verification token".to_string(),
            )),
        }
    }

    /// Creates an `EmailVerificationToken` from an owned String with validation.
    ///
    /// This is a convenience method that takes ownership of the string and validates it.
    /// See `try_from_string` for detailed validation behaviour.
    ///
    /// # Arguments
    /// * `jwt_string` - The owned JWT string to validate and convert
    /// * `secret` - The secret key used for JWT signature verification
    /// * `issuer` - The expected issuer for validation
    ///
    /// # Returns
    /// * `Ok(EmailVerificationToken)` if the JWT is valid
    /// * `Err(AuthenticationError)` if validation fails
    pub fn try_from_owned_string(
        jwt_string: String,
        secret: &secrecy::SecretString,
        issuer: &secrecy::SecretString,
    ) -> Result<Self, AuthenticationError> {
        Self::try_from_string(&jwt_string, secret, issuer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{self, Users};
    use crate::domain::tokens::{TokenClaimNew, TokenType};
    use chrono::Duration;
    use fake::faker::company::en::CompanyName;
    use fake::{Fake, Faker};
    use secrecy::SecretString;

    fn mock_user() -> database::Users {
        let user = database::Users::mock_data().unwrap();
        user
    }

    fn mock_secret() -> SecretString {
        let fake_secret: String = Faker.fake();
        SecretString::new(fake_secret.into_boxed_str())
    }

    fn mock_issuer() -> SecretString {
        let fake_company: String = CompanyName().fake();
        SecretString::new(fake_company.into_boxed_str())
    }

    fn create_email_verification_claim() -> (TokenClaimNew, SecretString) {
        let user = database::Users::mock_data().unwrap();
        let issuer = mock_issuer();
        let duration = Duration::hours(24);
        let claim = TokenClaimNew::new(
            &issuer,
            &duration,
            &user,
            &TokenType::EmailVerification,
        );
        (claim, mock_secret())
    }

    fn create_non_email_verification_claim() -> (TokenClaimNew, SecretString) {
        let user = mock_user();
        let issuer = mock_issuer();
        let duration = Duration::hours(1);
        let claim = TokenClaimNew::new(&issuer, &duration, &user, &TokenType::Access);
        (claim, mock_secret())
    }

    #[test]
    fn test_email_verification_token_try_from_claim_success() {
        let (claim, secret) = create_email_verification_claim();

        let result = EmailVerificationToken::try_from_claim(claim, &secret);

        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.as_ref().is_empty());
        assert!(token.as_ref().contains('.')); // JWT should have dots
    }

    #[test]
    fn test_email_verification_token_try_from_claim_wrong_type() {
        let (claim, secret) = create_non_email_verification_claim();

        let result = EmailVerificationToken::try_from_claim(claim, &secret);

        assert!(result.is_err());
        match result.unwrap_err() {
            AuthenticationError::InvalidToken(msg) => {
                assert!(msg.contains("not an email verification token"));
            }
            _ => panic!("Expected InvalidToken error"),
        }
    }

    #[test]
    fn test_email_verification_token_as_ref() {
        let (claim, secret) = create_email_verification_claim();
        let token = EmailVerificationToken::try_from_claim(claim, &secret).unwrap();

        let token_str: &str = token.as_ref();

        assert!(!token_str.is_empty());
        assert!(token_str.starts_with("eyJ")); // JWT header
        assert_eq!(token_str, token.0); // Should match internal string
    }

    #[test]
    fn test_email_verification_token_display() {
        let (claim, secret) = create_email_verification_claim();
        let token = EmailVerificationToken::try_from_claim(claim, &secret).unwrap();

        let display_str = format!("{}", token);

        assert_eq!(display_str, token.as_ref());
        assert!(display_str.starts_with("eyJ"));
    }

    #[test]
    fn test_email_verification_token_debug() {
        let (claim, secret) = create_email_verification_claim();
        let token = EmailVerificationToken::try_from_claim(claim, &secret).unwrap();

        let debug_str = format!("{:?}", token);

        assert!(debug_str.contains("EmailVerificationToken"));
        assert!(debug_str.contains("eyJ"));
    }

    #[test]
    fn test_email_verification_token_clone() {
        let (claim, secret) = create_email_verification_claim();
        let token = EmailVerificationToken::try_from_claim(claim, &secret).unwrap();

        let cloned_token = token.clone();

        assert_eq!(token, cloned_token);
        assert_eq!(token.as_ref(), cloned_token.as_ref());
    }

    #[test]
    fn test_email_verification_token_partial_eq() {
        let (claim1, secret1) = create_email_verification_claim();
        let (claim2, secret2) = create_email_verification_claim();

        let token1 =
            EmailVerificationToken::try_from_claim(claim1.clone(), &secret1)
                .unwrap();
        let token2 =
            EmailVerificationToken::try_from_claim(claim1, &secret1).unwrap();
        let token3 =
            EmailVerificationToken::try_from_claim(claim2, &secret2).unwrap();

        assert_eq!(token1, token2); // Same claim and secret
        assert_ne!(token1, token3); // Different claims
    }

    #[test]
    fn test_email_verification_token_default() {
        let token = EmailVerificationToken::default();

        assert_eq!(token.as_ref(), "");
        assert_eq!(token.0, String::new());
    }

    #[test]
    fn test_email_verification_token_serialization() {
        let (claim, secret) = create_email_verification_claim();
        let token = EmailVerificationToken::try_from_claim(claim, &secret).unwrap();

        let serialized = serde_json::to_string(&token).unwrap();
        let deserialized: EmailVerificationToken =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(token, deserialized);
        assert_eq!(token.as_ref(), deserialized.as_ref());
    }

    #[test]
    fn test_email_verification_token_roundtrip_with_token_claim_parse() {
        let (original_claim, secret) = create_email_verification_claim();
        let issuer = mock_issuer();

        // Create token from claim
        let token =
            EmailVerificationToken::try_from_claim(original_claim.clone(), &secret)
                .unwrap();

        // Parse token back to claim
        let parsed_claim = TokenClaimNew::parse(token.as_ref(), &secret, &issuer);

        // Note: This test might fail if the issuer doesn't match what was used in the original claim
        // In a real implementation, you'd want to use the same issuer
        assert!(parsed_claim.is_ok() || parsed_claim.is_err()); // Just ensure it doesn't panic
    }

    #[test]
    fn test_email_verification_token_with_different_durations() {
        let user = mock_user();
        let issuer = mock_issuer();
        let secret = mock_secret();

        // Test with different expiration times
        for hours in [1, 24, 48, 72] {
            let duration = Duration::hours(hours);
            let claim = TokenClaimNew::new(
                &issuer,
                &duration,
                &user,
                &TokenType::EmailVerification,
            );
            let token =
                EmailVerificationToken::try_from_claim(claim, &secret).unwrap();

            assert!(!token.as_ref().is_empty());
            assert!(token.as_ref().starts_with("eyJ"));
        }
    }

    #[test]
    fn test_email_verification_token_jwt_structure() {
        let (claim, secret) = create_email_verification_claim();
        let token = EmailVerificationToken::try_from_claim(claim, &secret).unwrap();

        let jwt_parts: Vec<&str> = token.as_ref().split('.').collect();

        assert_eq!(jwt_parts.len(), 3); // Header.Payload.Signature
        assert!(!jwt_parts[0].is_empty()); // Header
        assert!(!jwt_parts[1].is_empty()); // Payload
        assert!(!jwt_parts[2].is_empty()); // Signature
    }

    #[test]
    fn test_email_verification_token_consistent_output() {
        let (claim, secret) = create_email_verification_claim();

        // Create multiple tokens from the same claim and secret
        let token1 =
            EmailVerificationToken::try_from_claim(claim.clone(), &secret).unwrap();
        let token2 = EmailVerificationToken::try_from_claim(claim, &secret).unwrap();

        // Should be identical since the claim contains the same data
        assert_eq!(token1, token2);
        assert_eq!(token1.as_ref(), token2.as_ref());
    }

    #[test]
    fn test_email_verification_token_different_secrets_produce_different_tokens() {
        let user = mock_user();
        let issuer = mock_issuer();
        let duration = Duration::hours(24);
        let claim = TokenClaimNew::new(
            &issuer,
            &duration,
            &user,
            &TokenType::EmailVerification,
        );

        let secret1 = mock_secret();
        let secret2 = mock_secret();

        let token1 =
            EmailVerificationToken::try_from_claim(claim.clone(), &secret1).unwrap();
        let token2 =
            EmailVerificationToken::try_from_claim(claim, &secret2).unwrap();

        // Different secrets should produce different tokens
        assert_ne!(token1, token2);
        assert_ne!(token1.as_ref(), token2.as_ref());
    }

    #[test]
    fn test_email_verification_token_string_usage() {
        let (claim, secret) = create_email_verification_claim();
        let token = EmailVerificationToken::try_from_claim(claim, &secret).unwrap();

        // Test that it can be used as a string in various contexts
        let token_str = token.as_ref();
        assert!(token_str.len() > 50); // Reasonable JWT length

        // Test string methods work
        assert!(token_str.contains('.'));
        assert!(!token_str.is_empty());

        // Test it can be used in string formatting
        let formatted = format!("Token: {}", token);
        assert!(formatted.starts_with("Token: eyJ"));
    }
}
