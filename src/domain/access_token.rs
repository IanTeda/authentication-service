//-- ./src/domains/access_token.rs

// #![allow(unused)] // For beginning only.

//! JSON Web Token used to authorise RPC access for endpoint requests
//!
//! Generate a new instance of Access Token and decode an existing Access Token
//! into a Token Claim
//! ---

use core::time;
use jsonwebtoken::{encode, EncodingKey, Header};
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

use crate::{database, domain::jwt_token::TokenType, prelude::*};

use super::TokenClaim;

/// Access Token for authorising endpoint requests
/// #[derive(Debug, Clone, Default, PartialEq)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AccessToken(String);

/// Get string reference of the Access Token
impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Roll our own Display trait for Access Token
impl std::fmt::Display for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AccessToken {
    /// # New Access Token
    ///
    /// Create a new Access Token, returning a Result with an AccessToken or
    /// BackEnd error
    ///
    /// ## Parameters
    ///
    /// - `secret<&SecretString>` - containing the token encryption secret
    /// - `issuer<&SecretString>` - Containing the issuer of the JWT
    /// - `duration<&time::Duration>` - How long the token is valid for
    /// - `user_id<&database::Users>` - A database Users instance that is going to use the Access Token
    ///
    #[tracing::instrument(name = "Generate a new Access Token for: ", skip(secret))]
    pub fn new(
        secret: &SecretString ,
        issuer: &SecretString ,
        duration: &time::Duration,
        user: &database::Users,
    ) -> Result<Self, AuthenticationError> {
        // Build the Access Token Claim
        let token_claim =
            TokenClaim::new(issuer, duration, user, &TokenType::Access);

        // Encode the Token Claim into a URL-safe hash encryption
        let token = encode(
            &Header::default(),
            &token_claim,
            &EncodingKey::from_secret(secret.expose_secret().as_bytes()),
        )?;

        Ok(Self(token))
    }

    /// # Parse Access Token
    /// 
    /// Parse the Access Token from the request header, returning a Result with
    /// an AccessToken or BackEnd error
    pub fn parse_header(
        request_header: &tonic::metadata::MetadataMap
    ) -> Result<Self, AuthenticationError> {
        // Get authorization header from request
        let authorization_header = request_header
            .get("authorization")
            .ok_or(AuthenticationError::AuthenticationError(
                "Authorization header not found".to_string(),
            ))?;
        
        // Get the access token string from the authorization header bearer string
        let access_token = authorization_header
            .to_str()
            .map_err(|_| {
                AuthenticationError::AuthenticationError(
                    "Authorization header is not a valid string".to_string(),
                )
            })?
            .replace("Bearer ", "");

        Ok(Self(access_token))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use fake::faker::company::en::CompanyName;
    use fake::faker::number::en::Digit;
    use fake::Fake;
    use rand::distr::{Alphanumeric, SampleString};
    // use rand::distributions::{Alphanumeric, DistString};

    use crate::database;

    // Bring module into test scope
    use super::*;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    #[tokio::test]
    async fn generate_new_access_token() -> Result<()> {
        //-- 1. Setup and Fixtures (Arrange)

        // Generate random secret string
        let random_secret = Alphanumeric.sample_string(&mut rand::rng(), 60);
        let random_secret = SecretString::from(random_secret);

        // Get a random user_id for subject
        let random_user = database::Users::mock_data()?;

        // Generate a random company name as issurer
        let random_issuer = CompanyName().fake::<String>();
        let random_issuer = SecretString::from(random_issuer);

        // Generate a random duration between 1 and 10 hours
        let random_duration =
            std::time::Duration::from_secs((1..36000).fake::<u64>());

        // Create a new random access token
        let access_token = AccessToken::new(
            &random_secret,
            &random_issuer,
            &random_duration,
            &random_user,
        )?;

        //-- 2. Execute Test (Act)
        // Parse a token claim from the access token
        let token_claim = TokenClaim::parse(
            access_token.as_ref(),
            &random_secret,
            &random_issuer,
        )?;

        //-- 3. Test Assertions
        assert_eq!(token_claim.iss, *random_issuer.expose_secret());
        assert_eq!(token_claim.sub, random_user.id.to_string());
        assert_eq!(token_claim.jty, TokenType::Access.to_string());

        Ok(())
    }
}
