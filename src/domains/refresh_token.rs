//-- ./src/domains/refresh_token.rs

// #![allow(unused)] // For beginning only.

//! JSON Web Token used to authorise a request for a new Access Token
//!
//! Generate a new instance of a Refresh Token and decode an existing Refresh Token
//! into a Token Claim
//! ---

use crate::{domains::token_claim::TokenType, prelude::*};

use jsonwebtoken::{
	decode, encode, errors::Error as JWTError, DecodingKey, EncodingKey, Header,
	Validation,
};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{TokenClaim, TOKEN_ISSUER};

pub static REFRESH_TOKEN_DURATION: u64 = 2 * 60 * 60; // 2 hour as seconds

/// Refresh Token for authorising a new Access Token
#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
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
	/// Parse a new Access Token, returning a Result with an AccessToken or BackEnd error
	///
	/// ## Parameters
	///
	/// * `secret`: Secret<String> containing the token encryption secret
	/// * `user_id`: Uuid of the user that is going to use the Access Token
	/// ---
	pub async fn new(
		secret: &Secret<String>,
		user_id: &Uuid,
	) -> Result<Self, BackendError> {
		// Convert Uuid into a String
		let user_id = user_id.to_string();

		// Build the Access Token Claim
		let token_claim = TokenClaim::new(&secret, &user_id, &TokenType::Refresh);

		// Encode the Token Claim into a URL-safe hash encryption
		let token = encode(
			&Header::default(),
			&token_claim,
			&EncodingKey::from_secret(secret.expose_secret().as_bytes()),
		)?;

		Ok(Self(token))
	}
}

#[cfg(test)]
mod tests {

	// Bring module into test scope
	use super::*;

	use crate::{database::UserModel, error::BackendError::JsonWebToken};
	use claims::assert_err;
	use rand::distributions::{Alphanumeric, DistString};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	#[tokio::test]
	async fn generate_new_refresh_token() -> Result<()> {
		// Generate random secret string
		let secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 60);
		let secret = Secret::new(secret);

		// Get a random user_id for subject
		let random_user = UserModel::generate_random().await?;
		let user_id = random_user.id;

		let refresh_token = RefreshToken::new(&secret, &user_id).await?;

		let token_claim =
			TokenClaim::from_token(refresh_token.as_ref(), &secret).await?;
		// println!("{token_claim:#?}");

		let user_id = user_id.to_string();
		let token_type = TokenType::Refresh.to_string();

		assert_eq!(token_claim.iss, TOKEN_ISSUER);
		assert_eq!(token_claim.sub, user_id);
		assert_eq!(token_claim.jty, token_type);

		Ok(())
	}
}
