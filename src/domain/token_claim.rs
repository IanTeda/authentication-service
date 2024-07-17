//-- ./src/utilities/jwt.rs

// #![allow(unused)] // For beginning only.

//! JSON Web token utility
//!
//! //TODO: Make errors consistent across application
//!
//! # References
//!
//! * [Keats/jsonwebtoken](https://github.com/Keats/jsonwebtoken/tree/master)
//! * [Implementing JWT Authentication in Rust](https://www.shuttle.rs/blog/2024/02/21/using-jwt-auth-rust)
//! * [DefGuard/defguard](https://github.com/DefGuard/defguard/blob/main/src/auth/mod.rs
//! * [JSON Web Token (JWT)(https://www.rfc-editor.org/rfc/rfc7519#section-4.1.3)

use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use secrecy::{ExposeSecret, Secret};
use std::time::Duration;
use std::time::SystemTime;
use strum::Display;
use uuid::Uuid;

use crate::prelude::*;

pub static TOKEN_ISSUER: &str = "Personal Ledger Backend";

#[derive(Display, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

impl rand::distributions::Distribution<TokenType> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> TokenType {
        match rng.gen_range(0..2) {
            0 => TokenType::Access,
            _ => TokenType::Refresh,
        }
    }
}

/// Token Claim used in generating JSON Web Tokens (JWT)
///
/// Tokens have a limited time span and are used to authenticate requests to the server
///
/// # References
///
/// * [IANA JWT](https://www.iana.org/assignments/jwt/jwt.xhtml)
/// ---
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TokenClaim {
    pub iss: String, // Optional.  Issuer of the JWT.
    pub sub: String, // Optional. Subject (whom token refers to)
    // aud: String,         // Optional. The JWT intended recipient or audience.
    pub exp: u64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub nbf: u64, // Optional. Not Before (as UTC timestamp). Identifies the time before which JWT can not be accepted into processing.
    pub iat: u64, // Optional. Identifies the time at which the JWT was issued. This can be used to establish the age of the JWT or the exact time the token was generated.
    pub jti: String, // (JWT ID): Unique identifier; this can be used to prevent the JWT from being used more than once.
    pub jty: String, // Custom. Identify the token as access or refresh
}

impl TokenClaim {
    /// Create a new Token Claim
    ///
    /// # Example
    ///
    /// ```
    ///
    /// ```
    ///
    /// # Parameters
    ///
    /// * `secret`: The secret string wrapped in a Secret for encoding token
    /// * `subject`: The recipient of the token, or who will use the token during a request. This is the User Uuid.
    /// * `token_type`: token_claim::Kind, will the new claim be an access or refresh token
    /// ---
    pub fn new(
        secret: &Secret<String>,
        user_id: &str,
        token_type: &TokenType,
    ) -> Self {
        // Secret used to encode and decode tokens
        let secret = secret.to_owned();

        // Set JWT issuer
        let issuer = TOKEN_ISSUER.to_owned();

        let user_id = user_id.to_string();

        // System Time now
        let now = SystemTime::now();

        // Match duration against token kind
        let duration = match token_type {
            TokenType::Access => super::access_token::ACCESS_TOKEN_DURATION,
            TokenType::Refresh => super::refresh_token::REFRESH_TOKEN_DURATION,
        };

        // Token claim will expire at what System Time
        let expiration_timestamp = now
            .checked_add(Duration::from_secs(duration))
            .expect("valid time")
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("valid timestamp")
            .as_secs();

        // Token claim is not valid before System Time now.
        let not_before_timestamp = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("valid timestamp")
            .as_secs();

        // Token claim was issued System Time now
        let issued_at_timestamp = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("valid timestamp")
            .as_secs();

        // Token claim id with Uuid V7 with now timestamp
        let token_id = Uuid::now_v7().to_string();

        let token_type = token_type.to_string();

        Self {
            iss: issuer,
            sub: user_id,
            // aud,
            exp: expiration_timestamp,
            nbf: not_before_timestamp,
            iat: issued_at_timestamp,
            jti: token_id,
            jty: token_type,
        }
    }

    /// Decode a Token into to Token Claim
    ///
    /// ## Parameters
    ///
    /// * `token` [String]: The Token string to be decoded into a Token Claim.
    /// * `secret`: Secret<String> containing the token encryption secret
    /// ---
    pub fn from_token(
        token: &str,
        secret: &Secret<String>,
    ) -> Result<Self, BackendError> {
        // By default, automatically validate the expiration (exp) claim
        let mut validation = Validation::default();

        // Issuer (iss) of token to validate against
        validation.set_issuer(&[TOKEN_ISSUER]);

        // Validate Not before (nbf) claim
        validation.validate_nbf = true;

        // What is going to be validated against
        validation.set_required_spec_claims(&["iss", "exp", "nbf"]);

        // Decode Access Token into a Token Claim
        let token_claim = decode::<TokenClaim>(
            &token,
            &DecodingKey::from_secret(secret.expose_secret().as_bytes()),
            &validation,
        )
        .map(|data| data.claims)?;

        Ok(token_claim)
    }
}
