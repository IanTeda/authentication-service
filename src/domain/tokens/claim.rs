//-- ./src/domain/tokens/claim.rs

//! # Token Claims Module
//!
//! This module defines the `TokenClaim` struct and related functionality for handling
//! JWT (JSON Web Token) claims in the authentication service.
//!
//! ## Features
//! - **Standard JWT Claims**: Supports all standard JWT claims (iss, sub, aud, exp, nbf, iat, jti)
//! - **Custom Claims**: Includes custom `jty` claim for token type identification
//! - **Token Creation**: Provides `new()` method for creating tokens with proper timestamps
//! - **Token Parsing**: Provides `parse()` method for validating and decoding JWT tokens
//! - **Type Safety**: Uses strong types like `Uuid` for IDs and `TokenType` enum for token types
//! - **Security**: Uses `SecretString` for sensitive data like issuer information
//!
//! ## Error Handling
//! Token parsing returns specific `AuthenticationError` variants for different failure modes:
//! - `TokenExpired` for expired tokens
//! - `InvalidToken` for malformed, tampered, or incorrectly signed tokens

use chrono::{self, DateTime, Timelike, Utc};
use jsonwebtoken;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database,
    domain::{
        self,
        tokens::{self, TokenType},
    },
    AuthenticationError,
};

/// # Token Claim Struct
///
/// Represents the standard and custom claims contained within a JWT (JSON Web Token).
///
/// This struct is used for encoding and decoding JWTs in the authentication service,
/// supporting both standard JWT fields and application-specific claims.
///
/// # Fields
/// - `iss`: **Issuer** — Identifies the principal that issued the JWT (e.g., `https://mydomain.com`).
/// - `sub`: **Subject** — Identifies the subject of the JWT (the user or entity the token refers to).
/// - `aud`: **Audience** — Identifies the recipients that the JWT is intended for (the consuming application).
/// - `exp`: **Expiration Time** — UTC timestamp after which the JWT must not be accepted for processing.
/// - `nbf`: **Not Before** — UTC timestamp before which the JWT must not be accepted for processing.
/// - `iat`: **Issued At** — UTC timestamp indicating when the JWT was issued.
/// - `jti`: **JWT ID** — Unique identifier for the JWT, used to prevent replay attacks.
/// - `jty`: **JWT Type** — Custom claim to distinguish between Access, Refresh, EmailVerification and PasswordReset tokens.
///
/// # Notes
/// - Custom claims (`jty`) are included for application-specific logic.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TokenClaim {
    pub jti: Uuid,      // Token unique identifier
    pub jty: TokenType, // Token type (custom)
    #[serde(
        serialize_with = "serialize_secret_string",
        deserialize_with = "deserialize_secret_string"
    )]
    pub iss: SecretString, // Issuer
    pub sub: Uuid,      // Subject
    pub aud: String,    // Audience
    // De/Serialize to timestamp seconds that JWT expects
    #[serde(with = "chrono::serde::ts_seconds")]
    pub iat: DateTime<Utc>, // Issued at
    #[serde(with = "chrono::serde::ts_seconds")]
    pub nbf: DateTime<Utc>, // Not before
    #[serde(with = "chrono::serde::ts_seconds")]
    pub exp: DateTime<Utc>, // Expiration
}

/// Custom serializer for `SecretString` fields in JWT claims.
///
/// This function safely exposes the secret value only during serialization,
/// allowing JWT encoding while maintaining the security benefits of `SecretString`
/// throughout the rest of the application lifecycle.
///
/// # Arguments
/// * `secret` - The `SecretString` to serialize
/// * `serializer` - The serde serializer instance
///
/// # Returns
/// The serialized string value or a serialization error
///
/// # Security Note
/// This function temporarily exposes the secret value for JWT serialization.
/// The `SecretString` type ensures the value remains protected in memory
/// during normal application operations.
fn serialize_secret_string<S>(
    secret: &SecretString,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(secret.expose_secret())
}

/// Custom deserializer for `SecretString` fields in JWT claims.
///
/// This function safely converts a string value back into a `SecretString` during
/// deserialization, allowing JWT decoding while maintaining the security benefits
/// of `SecretString` throughout the rest of the application lifecycle.
///
/// # Arguments
/// * `deserializer` - The serde deserializer instance
///
/// # Returns
/// A `SecretString` containing the deserialized value or a deserialization error
///
/// # Security Note
/// This function creates a new `SecretString` from the deserialized string value.
/// The `SecretString` type ensures the value remains protected in memory
/// during normal application operations after deserialization.
fn deserialize_secret_string<'de, D>(
    deserializer: D,
) -> Result<SecretString, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(SecretString::new(s.into_boxed_str()))
}

impl PartialEq for TokenClaim {
    /// Compares two `TokenClaim` instances for equality.
    ///
    /// This implementation manually compares all fields, including the sensitive
    /// `iss` field by exposing and comparing the underlying secret values.
    /// Two token claims are considered equal if all their fields match exactly.
    ///
    /// # Note
    /// The `iss` field comparison uses `expose_secret()` to access the underlying
    /// secret value for comparison, as `SecretString` does not implement `PartialEq`.
    fn eq(&self, other: &Self) -> bool {
        self.jti == other.jti
            && self.jty == other.jty
            && self.iss.expose_secret() == other.iss.expose_secret()
            && self.sub == other.sub
            && self.aud == other.aud
            && self.iat == other.iat
            && self.nbf == other.nbf
            && self.exp == other.exp
    }
}

impl std::fmt::Display for TokenClaim {
    /// Formats the `TokenClaim` as a readable string with all fields.
    ///
    /// This implementation is useful for logging, debugging, or displaying the contents
    /// of a JWT claim in a human-readable format.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
          f,
          "TokenClaim {{ jti: {}, jty: {}, iss: {}, sub: {}, aud: {}, iat: {}, nbf: {}, exp: {} }}",
          self.jti, self.jty, self.iss.expose_secret(), self.sub, self.aud, self.iat, self.nbf, self.exp
      )
    }
}

impl TokenClaim {
    pub fn new(
        issuer: &SecretString,
        duration: &chrono::Duration,
        user: &database::Users,
        token_type: &tokens::TokenType,
    ) -> Self {
        // Get the user id from the user instance passed in
        let subject = user.id;

        // TODO: consider making this a configuration setting
        let audience = "authentication_service".to_string();

        // Get System Time now, truncated to seconds for JWT compatibility
        let now = chrono::Utc::now()
            .with_nanosecond(0)
            .expect("Failed to truncate nanoseconds from current time");

        // Token expires now plus duration passed in
        let expiration = (now + *duration)
            .with_nanosecond(0)
            .expect("Failed to truncate nanoseconds from expiration time");

        // Set the not before to now
        let not_before = now;

        // Set the issued at to now
        let issued_at = now;

        // Generate a new UUID V7 based on now
        let token_id = uuid::Uuid::now_v7();

        Self {
            jti: token_id,
            jty: token_type.clone(),
            iss: issuer.clone(),
            sub: subject,
            aud: audience,
            iat: issued_at,
            nbf: not_before,
            exp: expiration,
        }
    }

    pub fn parse(
        token: &str,
        secret: &SecretString,
        issuer: &SecretString,
    ) -> Result<Self, AuthenticationError> {
        // Build token validation requirements. By default, the decoding will
        // automatically validate the expiration (exp) claim
        let mut validation = jsonwebtoken::Validation::default();

        // Issuer (iss) of token to validate against
        validation.set_issuer(&[issuer.expose_secret()]);

        // Set the expected audience to match what we set in new()
        validation.set_audience(&["authentication_service"]);

        // Validate Not before (nbf) claim
        validation.validate_nbf = true;

        // What additional values are we going to be validated against
        // Issuer, Expiration Time & Not Before Time
        validation.set_required_spec_claims(&["iss", "iat", "exp", "nbf"]);

        // Decode JWT token into a TokenClaim
        match jsonwebtoken::decode::<TokenClaim>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(
                secret.expose_secret().as_bytes(),
            ),
            &validation,
        ) {
            Ok(token_data) => Ok(token_data.claims),
            Err(jwt_error) => {
                // Convert JWT error to our domain error with context
                match jwt_error.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        Err(AuthenticationError::TokenExpired)
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidToken => {
                        Err(AuthenticationError::InvalidToken(
                            "Token format is invalid".to_string(),
                        ))
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                        Err(AuthenticationError::InvalidToken(
                            "Token issuer is invalid".to_string(),
                        ))
                    }
                    jsonwebtoken::errors::ErrorKind::ImmatureSignature => {
                        Err(AuthenticationError::InvalidToken(
                            "Token is not yet valid (nbf claim)".to_string(),
                        ))
                    }
                    _ => Err(AuthenticationError::InvalidToken(format!(
                        "JWT decode failed: {}",
                        jwt_error
                    ))),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Users;
    use chrono::Duration;
    use fake::faker::company::en::CompanyName;
    use fake::{Fake, Faker};
    use secrecy::SecretString;

    fn mock_user() -> Users {
        Users::mock_data().unwrap()
    }

    fn mock_secret() -> SecretString {
        let fake_secret: String = Faker.fake();
        SecretString::new(fake_secret.into_boxed_str())
    }

    fn mock_issuer() -> SecretString {
        let fake_company: String = CompanyName().fake();
        SecretString::new(fake_company.into_boxed_str())
    }

    #[test]
    fn test_token_claim_new_creates_valid_claim() {
        let user = mock_user();
        let duration = Duration::hours(1);
        let issuer = mock_issuer();
        let token_type = TokenType::Access;

        let claim = TokenClaim::new(&issuer, &duration, &user, &token_type);

        assert_eq!(claim.sub, user.id);
        assert_eq!(claim.jty, token_type);
        assert_eq!(claim.iss.expose_secret(), issuer.expose_secret());
        assert_eq!(claim.aud, "authentication_service");
        assert!(claim.exp > claim.iat);
        assert_eq!(claim.nbf, claim.iat);
    }

    #[test]
    fn test_token_claim_expiration_time() {
        let user = mock_user();
        let duration = Duration::hours(2);
        let issuer = mock_issuer();
        let token_type = TokenType::Refresh;

        let claim = TokenClaim::new(&issuer, &duration, &user, &token_type);
        let expected_exp = claim.iat + duration;

        assert_eq!(claim.exp, expected_exp);
    }

    #[test]
    fn test_token_claim_display() {
        let user = mock_user();
        let duration = Duration::minutes(30);
        let issuer = mock_issuer();
        let token_type = TokenType::EmailVerification;

        let claim = TokenClaim::new(&issuer, &duration, &user, &token_type);
        let display_str = format!("{}", claim);

        assert!(display_str.contains("TokenClaim"));
        assert!(display_str.contains(&user.id.to_string()));
        assert!(display_str.contains("email_verification"));
    }

    #[test]
    fn test_token_claim_equality() {
        let user = mock_user();
        let duration = Duration::hours(1);
        let issuer = mock_issuer();
        let token_type = TokenType::Access;

        let claim1 = TokenClaim::new(&issuer, &duration, &user, &token_type);
        let claim2 = claim1.clone();

        assert_eq!(claim1, claim2);
    }

    #[test]
    fn test_token_claim_parse_valid_token() {
        let user = mock_user();
        let duration = Duration::hours(1);
        let issuer = mock_issuer();
        let secret = mock_secret();
        let token_type = TokenType::Access;

        let original_claim = TokenClaim::new(&issuer, &duration, &user, &token_type);

        // Create a JWT token from the claim
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &original_claim,
            &jsonwebtoken::EncodingKey::from_secret(
                secret.expose_secret().as_bytes(),
            ),
        )
        .unwrap();

        // Parse the token back
        let parsed_claim = TokenClaim::parse(&token, &secret, &issuer).unwrap();

        assert_eq!(parsed_claim, original_claim);
    }

    #[test]
    fn test_token_claim_parse_expired_token() {
        let user = mock_user();
        let duration = Duration::hours(1); // Create a valid token first
        let issuer = mock_issuer();
        let secret = mock_secret();
        let token_type = TokenType::Access;

        // Create a claim that will be expired
        let mut expired_claim =
            TokenClaim::new(&issuer, &duration, &user, &token_type);

        // Manually set the expiration to the past
        expired_claim.exp = chrono::Utc::now() - Duration::hours(1);

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &expired_claim,
            &jsonwebtoken::EncodingKey::from_secret(
                secret.expose_secret().as_bytes(),
            ),
        )
        .unwrap();

        let result = TokenClaim::parse(&token, &secret, &issuer);

        assert!(matches!(result, Err(AuthenticationError::TokenExpired)));
    }

    #[test]
    fn test_token_claim_parse_invalid_token() {
        let secret = mock_secret();
        let issuer = mock_issuer();
        let invalid_token = "invalid.jwt.token";

        let result = TokenClaim::parse(invalid_token, &secret, &issuer);

        assert!(matches!(result, Err(AuthenticationError::InvalidToken(_))));
    }

    #[test]
    fn test_token_claim_parse_wrong_issuer() {
        let user = mock_user();
        let duration = Duration::hours(1);
        let issuer = mock_issuer();
        let wrong_issuer = SecretString::new("wrong_issuer".to_string().into());
        let secret = mock_secret();
        let token_type = TokenType::Access;

        let claim = TokenClaim::new(&issuer, &duration, &user, &token_type);

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claim,
            &jsonwebtoken::EncodingKey::from_secret(
                secret.expose_secret().as_bytes(),
            ),
        )
        .unwrap();

        let result = TokenClaim::parse(&token, &secret, &wrong_issuer);

        assert!(matches!(result, Err(AuthenticationError::InvalidToken(_))));
    }

    #[test]
    fn test_token_claim_parse_wrong_secret() {
        let user = mock_user();
        let duration = Duration::hours(1);
        let issuer = mock_issuer();
        let secret = mock_secret();
        let wrong_secret = SecretString::new("wrong_secret".to_string().into());
        let token_type = TokenType::Access;

        let claim = TokenClaim::new(&issuer, &duration, &user, &token_type);

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claim,
            &jsonwebtoken::EncodingKey::from_secret(
                secret.expose_secret().as_bytes(),
            ),
        )
        .unwrap();

        let result = TokenClaim::parse(&token, &wrong_secret, &issuer);

        assert!(matches!(result, Err(AuthenticationError::InvalidToken(_))));
    }

    #[test]
    fn test_different_token_types() {
        let user = mock_user();
        let duration = Duration::hours(1);
        let issuer = mock_issuer();

        let access_claim =
            TokenClaim::new(&issuer, &duration, &user, &TokenType::Access);
        let refresh_claim =
            TokenClaim::new(&issuer, &duration, &user, &TokenType::Refresh);
        let email_claim = TokenClaim::new(
            &issuer,
            &duration,
            &user,
            &TokenType::EmailVerification,
        );
        let password_claim =
            TokenClaim::new(&issuer, &duration, &user, &TokenType::PasswordReset);

        assert_eq!(access_claim.jty, TokenType::Access);
        assert_eq!(refresh_claim.jty, TokenType::Refresh);
        assert_eq!(email_claim.jty, TokenType::EmailVerification);
        assert_eq!(password_claim.jty, TokenType::PasswordReset);
    }
}
