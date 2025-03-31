//-- ./src/utilities/jwt.rs

// #![allow(unused)] // For beginning only.

//! JSON Web token utility
//!
//! //TODO: Make errors consistent across application
//! //TODO: Tidy up token domain into a subfolder
//! //TODO: Implient display for TokenClaim
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
use std::time;
use strum::Display;
use uuid::Uuid;

use crate::database;
use crate::prelude::*;

/// Token Types
//TODO: Impellent own Display trait
#[derive(Debug, Clone, Default, PartialEq, Display)]
pub enum TokenType {
    #[default]
    Access,
    Refresh,
}

/// Pick a random token type
/// 
/// ## Example
/// ```
/// use crate::domain::jwt_token::TokenType;
/// use rand::Rng;
///
/// let random_token_type: TokenType = rand::random();
/// assert!(matches!(random_token_type, TokenType::Access | TokenType::Refresh));
/// ```
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
// #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TokenClaim {
    /// # JWT Issuer
    /// The issuer of the JWT. Usually `https://mydomain.com`
    pub iss: String,
    /// # JWT Subject
    /// Whom the token refers or issued to
    pub sub: String,
    // /// JWT Audience, or the application that will be using the token
    // aud: String,
    /// # JWT Expiration (as UTC timestamp). 
    /// Validate_exp defaults to true in validation
    pub exp: u64,
    /// # JWT Not Before (as UTC timestamp).
    /// Identifies the time before which JWT can not be accepted into processing.
    pub nbf: u64,
    /// # JWT Issued At (as UTC timestamp)
    /// Identifies the time at which the JWT was issued. This can be used to establish 
    /// the age of the JWT or the exact time the token was generated.
    pub iat: u64,
    /// JWT Unique Identifier
    /// This can be used to prevent the JWT from being used more than once.
    pub jti: String,
    /// JWT Type (Custom)
    /// Used to identify if this is an Access or Refresh Token
    pub jty: String,
    /// JWT User Role (Custom)
    /// Used to identify the JWT user role for authorisation
    // TODO: Consider removing this, as the user instance is passed in the RPC response
    pub jur: String,
}

impl TokenClaim {
    /// # New Token Claim
    /// 
    /// This function returns a new token claim using the function parameters
    ///
    /// ## Parameters
    ///
    /// - `issuer<&str?` - The issuer of the JWT.
    /// - `duration<time::Duration>` - How long is the JWT valid for.
    /// - `user<database:Users>` - The user that JWT will be issued to.
    /// - `token_type<token_claim::Kind>` - What type of JWT will the new claim be, access or refresh token.
    /// ---
    pub fn new(
        issuer: &Secret<String>,
        duration: &time::Duration,
        user: &database::Users,
        token_type: &TokenType,
    ) -> Self {
        // Take ownership of the string, since it will be passsed back in the Token Claim
        let issuer = issuer.expose_secret().to_string();

        // Get System Time now
        let now = time::SystemTime::now();

        // Token claim will expire at this System Time
        let expiration_timestamp = now
            .checked_add(duration.to_owned())
            .expect("valid time")
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .expect("valid timestamp")
            .as_secs();

        // Calculate the system time now
        let system_now_timestamp = now
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .expect("valid timestamp")
            .as_secs();
        
        // Convert user id UUID to a string for the clim
        let user_id = user.id.to_string();

        // Token claim id with Uuid V7 with now timestamp
        let token_id = Uuid::now_v7().to_string();

        // Convert the token type to a string
        let token_type = token_type.to_string();

        // Convert the user type to a string
        let user_role = user.role.to_string();

        Self {
            iss: issuer,
            sub: user_id,
            // aud,
            exp: expiration_timestamp,
            nbf: system_now_timestamp,
            iat: system_now_timestamp,
            jti: token_id,
            jty: token_type,
            jur: user_role,
        }
    }

    /// # Parse a Token into a Token Claim
    /// 
    /// This function parses (decodes) a token string into a Token Claim. In doing
    /// so it validates the token.
    ///
    /// ## Parameters
    ///
    /// - `token<&str>` - The Token string to be decoded into a Token Claim.
    /// - `secret<Secret<String>>` - Contains the token encryption secret.
    /// - `issuer<&str>` - Who issused the JWT. Used to verify the token.
    /// ---
    pub fn parse(
        token: &str,
        secret: &Secret<String>,
        issuer: &Secret<String>,
    ) -> Result<Self, BackendError> {
        // Build token validation requirements. By default, the decoding will 
        // automatically validate the expiration (exp) claim
        let mut validation = Validation::default();

        // Issuer (iss) of token to validate against
        validation.set_issuer(&[issuer.expose_secret()]);

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


#[cfg(test)]
mod tests {
    use chrono::Duration;
    use fake::faker::company::en::CompanyName;
    use fake::faker::number::en::Digit;
    use fake::Fake;
    use rand::distributions::{Alphanumeric, DistString};

    use crate::database;

    // Bring module into test scope
    use super::*;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    #[tokio::test]
    async fn generate_new_token_claim() -> Result<()> {
        // Generate a random company name as issurer
        let random_issuer = CompanyName().fake::<String>();
        let random_issuer = Secret::new(random_issuer);

        // Generate a random duration between 1 and 10 hours
        let random_duration =
            std::time::Duration::from_secs((1..36000).fake::<u64>());

        // Get a random user_id for subject
        let random_user = database::Users::mock_data()?;

        // Pick a random token type
        let random_token_type: TokenType = rand::random();

        // Create a new random access token
        let token_claim = TokenClaim::new(
            &random_issuer,
            &random_duration,
            &random_user,
            &random_token_type,
        );

        assert_eq!(token_claim.iss, *random_issuer.expose_secret());
        assert_eq!(token_claim.sub, random_user.id.to_string());
        assert_eq!(token_claim.sub, random_user.id.to_string());
        assert_eq!(token_claim.jur, random_user.role.to_string());

        Ok(())
    }
}
