//-- ./src/utilities/jwt.rs

#![allow(unused)] // For beginning only.

//! JSON Web token utility
//!
//! //TODO: Make errors consistent across application
//!
//! # References
//!
//! * [Keats/jsonwebtoken](https://github.com/Keats/jsonwebtoken/tree/master)
//! https://www.shuttle.rs/blog/2024/02/21/using-jwt-auth-rust
//! https://github.com/DefGuard/defguard/blob/main/src/auth/mod.rs

use std::time::SystemTime;

use jsonwebtoken::{
	decode, encode, errors::Error as JWTError, Algorithm, DecodingKey, EncodingKey, Header,
	Validation,
};
use std::time::Duration;

use crate::prelude::BackendError;

pub static JWT_ISSUER: &str = "Personal Ledger Backend";
#[allow(clippy::identity_op)]
pub static JWT_DURATION: u64 = 1 * 60 * 60; // 1 hour as seconds

pub struct JwtKeys {
	encoding: EncodingKey,
	decoding: DecodingKey,
}

impl JwtKeys {
	pub fn new(secret: &[u8]) -> Self {
		Self {
			encoding: EncodingKey::from_secret(secret),
			decoding: DecodingKey::from_secret(secret),
		}
	}
}

// pub enum JwtError {
//     InvalidToken,
//     WrongCredentials,
//     TokenCreation,
//     MissingCredentials,
// }

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Claims {
	iss: String,     // Optional.  Issuer of the JWT.
	pub sub: String, // Optional. Subject (whom token refers to)
	// aud: String,         // Optional. The JWT intended recipient or audience.
	exp: u64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
	nbf: u64, // Optional. Not Before (as UTC timestamp). Identifies the time before which JWT can not be accepted into processing.
	iat: u64, // Optional. Identifies the time at which the JWT was issued. This can be used to establish the age of the JWT or the exact time the token was generated.
	          // jti: String        // (JWT ID): Unique identifier; this can be used to prevent the JWT from being used more than once.
}

impl Claims {
	pub fn new(issuer: String, subject: String, duration: u64) -> Self {
		// System Time now
		let now = SystemTime::now();

		// Token claim will expire at what System Time
		let exp = now
			.checked_add(Duration::from_secs(duration))
			.expect("valid time")
			.duration_since(SystemTime::UNIX_EPOCH)
			.expect("valid timestamp")
			.as_secs();

		// Token claim is not valid before System Time now.
		let nbf = now
			.duration_since(SystemTime::UNIX_EPOCH)
			.expect("valid timestamp")
			.as_secs();

		// Token claim was issued System Time now
		let iat = now
			.duration_since(SystemTime::UNIX_EPOCH)
			.expect("valid timestamp")
			.as_secs();

		Self {
			iss: issuer,
			sub: subject,
			exp,
			nbf,
			iat,
		}
	}

	/// Convert claim instance to JWT.
	pub fn to_jwt(&self, jwt_keys: &JwtKeys) -> Result<String, BackendError> {
		let json_web_token = encode(&Header::default(), self, &jwt_keys.encoding)?;

		Ok(json_web_token)
	}

	/// Verify JWT and, if successful, convert it to claims.
	pub fn from_jwt(token: &str, jwt_keys: &JwtKeys) -> Result<Self, BackendError> {
		let mut validation = Validation::default();
		validation.validate_nbf = true;
		validation.set_issuer(&[JWT_ISSUER]);
		validation.set_required_spec_claims(&["iss", "sub", "exp", "nbf"]);
		let claim = decode::<Self>(
			token,
			// &DecodingKey::from_secret(secret),
			&jwt_keys.decoding,
			&validation,
		)
		.map(|data| data.claims)?;

		Ok(claim)
	}
}

#[cfg(test)]
mod tests {
	use chrono::{DateTime, Utc};
	use fake::{faker::chrono::en::DateTimeAfter, Fake};
	use rand::distributions::{Alphanumeric, DistString};
	use uuid::Uuid;

	use super::{Claims, JwtKeys, JWT_ISSUER};

	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	#[test]
	fn generate_verify_token() -> Result<()> {
		// Generate random secret string
		let random_secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 60);

		// Initiate JWT Keys
		let jwt_keys = JwtKeys::new(random_secret.as_bytes());

		// -- Generate random UUID V7, which requires a timestamp
		// Generate random DateTime after UNIX time epoch (00:00:00 UTC on 1 January 1970)
		let random_datetime: DateTime<Utc> = DateTimeAfter(chrono::DateTime::UNIX_EPOCH).fake();
		// Convert datetime to a UUID timestamp
		let random_uuid_timestamp: uuid::Timestamp = uuid::Timestamp::from_unix(
			uuid::NoContext,
			random_datetime.timestamp() as u64,
			random_datetime.timestamp_nanos_opt().unwrap() as u32,
		);
		// Generate Uuid V7
		let uuid_subject = Uuid::new_v7(random_uuid_timestamp).to_string();

		// Generate random duration
		let random_duration = (10..30000).fake::<u64>();

		// Initiate new claim
		let claim = Claims::new(JWT_ISSUER.to_owned(), uuid_subject, random_duration);
		// println!("{claim:#?}");

		let jwt_token = claim.to_jwt(&jwt_keys)?;
		// println!("{jwt_token:#?}");

		let claim_from_jwt = Claims::from_jwt(&jwt_token, &jwt_keys)?;
		// println!("{claim_from_jwt:#?}");

		assert_eq!(claim_from_jwt, claim);

		Ok(())
	}
}
