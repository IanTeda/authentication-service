//-- ./src/domains/refresh_tokens.rs

// #![allow(unused)] // For beginning only.

//! JSON Web Token used to authorise a request for a new Access Token
//!
//! Generate a new instance of a Refresh Token and decode an existing Refresh Token
//! into a Token Claim
//! ---

use jsonwebtoken::{encode, EncodingKey, Header};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use uuid::Uuid;

use crate::{database, domain::token_claim::TokenType, prelude::*};

use super::TokenClaim;

pub static REFRESH_TOKEN_DURATION: u64 = 2 * 60 * 60; // 2 hours as seconds

/// Refresh Token for authorising a new Access Token
// #[derive(serde::Deserialize, Debug, Clone, PartialEq)]
#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize)]
pub struct RefreshToken(String);

/// Get string reference of the Refresh Token
impl AsRef<str> for RefreshToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Roll our own Display trait for Access Token
impl std::fmt::Display for RefreshToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for RefreshToken {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl RefreshToken {
    /// Generate a new Access Token, returning a Result with an AccessToken or BackEnd error
    ///
    /// ## Parameters
    ///
    /// * `secret`: Secret<String> containing the token encryption secret
    /// * `user_id`: Uuid of the user that is going to use the Access Token
    /// ---
    #[tracing::instrument(
        name = "Generate a new Refresh Token for: "
        skip(secret)
    )]
    pub fn new(
        secret: &Secret<String>,
        user: &database::Users,
    ) -> Result<Self, BackendError> {
        // Build the Access Token Claim
        let token_claim = TokenClaim::new(secret, user, &TokenType::Refresh);

        // Encode the Token Claim into a URL-safe hash encryption
        let token = encode(
            &Header::default(),
            &token_claim,
            &EncodingKey::from_secret(secret.expose_secret().as_bytes()),
        )?;

        Ok(Self(token))
    }
}
