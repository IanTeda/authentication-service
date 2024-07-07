//-- ./src/utilities/jwt.rs

#![allow(unused)] // For beginning only.

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

use std::time::SystemTime;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::time::Duration;
use strum::Display;
use uuid::Uuid;

use crate::prelude::*;

pub static JWT_ISSUER: &str = "Personal Ledger Backend";
pub static JWT_DURATION: u64 = 15 * 60; // 15 minutes as seconds
pub static JWT_REFRESH_DURATION: u64 = 2 * 60 * 60; // 2 hour as seconds

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

// pub struct Header {
// 	typ: String,
// 	alg: String,
// }

#[derive(Display)]
pub enum JwtTypes {
	Access,
	Refresh,
}

impl rand::distributions::Distribution<JwtTypes> for rand::distributions::Standard {
	fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> JwtTypes {
		match rng.gen_range(0..2) {
			0 => JwtTypes::Access,
			_ => JwtTypes::Refresh,
		}
	}
}

// impl std::fmt::Display for Types {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//     	match self {
//            Types::Access => write!(f, "access"),
//            Types::Refresh => write!(f, "refresh"),
//        }
//     }
// }

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Claims {
	iss: String,     // Optional.  Issuer of the JWT.
	pub sub: String, // Optional. Subject (whom token refers to)
	// aud: String,         // Optional. The JWT intended recipient or audience.
	exp: u64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
	nbf: u64, // Optional. Not Before (as UTC timestamp). Identifies the time before which JWT can not be accepted into processing.
	iat: u64, // Optional. Identifies the time at which the JWT was issued. This can be used to establish the age of the JWT or the exact time the token was generated.
	jti: String, // (JWT ID): Unique identifier; this can be used to prevent the JWT from being used more than once.
	jty: String, // Custom. Identify the token as access or refresh
}

///
///
/// # References
///
/// * [IANA JWT](https://www.iana.org/assignments/jwt/jwt.xhtml)
impl Claims {
	pub fn new(issuer: String, subject: String, duration: u64, jwt_type: JwtTypes) -> Self {
		// System Time now
		let now = SystemTime::now();

		// let aud = configuration.application.ip_address;

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

		let token_type = jwt_type.to_string();

		Self {
			iss: issuer,
			sub: subject,
			// aud,
			exp: expiration_timestamp,
			nbf: not_before_timestamp,
			iat: issued_at_timestamp,
			jti: token_id,
			jty: token_type,
		}
	}

	/// Convert claim instance to JWT.
	pub fn to_jwt(&self, jwt_keys: &JwtKeys) -> Result<String, BackendError> {
		let mut header = Header::new(Algorithm::HS512);

		let json_web_token = encode(&Header::default(), self, &jwt_keys.encoding)?;

		Ok(json_web_token)
	}

	/// Verify JWT and, if successful, convert it to claims.
	pub fn from_jwt(token: &str, jwt_keys: &JwtKeys) -> Result<Self, BackendError> {
		let mut validation = Validation::default();
		validation.validate_nbf = true;
		validation.set_issuer(&[JWT_ISSUER]);
		validation.set_required_spec_claims(&["iss", "sub", "exp", "nbf"]);
		let claim =
			decode::<Self>(token, &jwt_keys.decoding, &validation).map(|data| data.claims)?;

		Ok(claim)
	}
}

#[cfg(test)]
mod tests {

	// Bring module into test scope
	use super::*;

	use chrono::{DateTime, Utc};
	use fake::{faker::chrono::en::DateTimeAfter, Fake};
	use rand::distributions::{Alphanumeric, DistString};
	use uuid::Uuid;

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

		// let jwt_type = JwtTypes::Access;
		let jwt_type: JwtTypes = rand::random();

		// Initiate new claim
		let claim = Claims::new(
			JWT_ISSUER.to_owned(),
			uuid_subject,
			random_duration,
			jwt_type,
		);
		// println!("{claim:#?}");

		let jwt_token = claim.to_jwt(&jwt_keys)?;
		// println!("{jwt_token:#?}");

		let claim_from_jwt = Claims::from_jwt(&jwt_token, &jwt_keys)?;
		// println!("{claim_from_jwt:#?}");

		assert_eq!(claim_from_jwt, claim);

		Ok(())
	}
}
