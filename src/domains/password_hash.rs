//-- ./src/domains/passwords.rs

// #![allow(unused)] // For beginning only.

//! Password domain parsing
//!
//! Parse string into a Password, checking for validation and hash as we go.
//!
//! //TODO: This can be written / structured better
//!
//! # References
//!
//! * [NIST password guidelines 2024: 15 rules to follow](https://community.trustcloud.ai/article/nist-password-guidelines-2024-15-rules-to-follow/)
//! * [PassMeRust Password Strength Calculator](https://github.com/dewan-ahmed/PassMeRust/tree/main)
//! * [argon_hash_password](https://github.com/mark-ruddy/argon_hash_password/blob/main/src/lib.rs)
//! ---

use crate::prelude::*;

use crate::domains::RefreshToken;
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version};
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PasswordHash(String);

impl PasswordHash {
	/// Parse `String` into a hashed password returning it in a Secret type.
	///
	/// # Parameters
	///
	/// * `password`: The password in a string
	/// ---
	pub fn parse(password: Secret<String>) -> Result<PasswordHash, BackendError> {
		// Parse String into Secret struct
		// let password = Secret::new(password.into());

		// Password must be at least 12 characters
		let is_to_short = password.expose_secret().len() < 12;

		// Password must be under 256 characters
		let is_to_long = password.expose_secret().len() > 255;

		// Password must contain an upper case letter
		#[allow(clippy::manual_range_contains)]
		let no_uppercase = !password
			.expose_secret()
			.bytes()
			.any(|byte| byte >= b'A' && byte <= b'Z');

		// Password must contain a lower case letter
		#[allow(clippy::manual_range_contains)]
		let no_lowercase = !password
			.expose_secret()
			.bytes()
			.any(|byte| byte >= b'a' && byte <= b'z');

		// Password must contain a number
		#[allow(clippy::manual_range_contains)]
		let no_number = !password
			.expose_secret()
			.bytes()
			.any(|byte| byte >= b'0' && byte <= b'9');

		// Password must contain a special character
		#[allow(clippy::manual_range_contains)]
		let no_special = !password.expose_secret().bytes().any(|byte| {
			byte < b'0'
				|| (byte > b'9' && byte < b'A')
				|| (byte > b'Z' && byte < b'a')
				|| byte > b'z'
		});

		// If any of the validations fail return an error else hash the password
		// and return within a Password Struct.
		if is_to_short
			|| is_to_long
			|| no_uppercase
			|| no_lowercase
			|| no_number || no_special
		{
			Err(BackendError::PasswordFormatInvalid)
		} else {
			// Generate encryption salt hash
			let salt = SaltString::generate(&mut rand::thread_rng());

			// Initiate new Argon2 instance
			let argon2 = Argon2::new(
				Algorithm::Argon2id,
				Version::V0x13,
				Params::new(15000, 2, 1, None).unwrap(),
			);

			// Hash password to PHC string ($argon2id$v=19$...)
			let mut password_hash = argon2
				.hash_password(password.expose_secret().as_bytes(), &salt)
				.unwrap()
				.to_string();

			Ok(Self(password_hash))
		}
	}

	/// Verify password string against password hash (i.e. verify password)
	///
	/// # Parameters
	///
	/// * `password`: Password in a String to check against the hash
	/// * `password_hash`: Password hash to check against
	/// ---
	pub fn verify_password(
		&self,
		password: &Secret<String>,
	) -> Result<bool, BackendError> {
		// Initiate new Argon2 instance
		let argon2 = Argon2::new(
			Algorithm::Argon2id,
			Version::V0x13,
			Params::new(15000, 2, 1, None).unwrap(),
		);

		// Parse into and Argon password hash
		// let argon_password_hash = argon2::PasswordHash::new(self.as_ref()).unwrap();

		let verified = argon2
			.verify_password(
				password.expose_secret().as_bytes(),
				&argon2::PasswordHash::new(self.as_ref()).unwrap(),
			)
			.is_ok();

		// unimplemented!()
		Ok(verified)
	}
}

/// Make a Password instance from String
impl From<String> for PasswordHash {
	fn from(value: String) -> Self {
		Self(value)
	}
}

impl AsRef<str> for PasswordHash {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl std::fmt::Display for PasswordHash {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

#[cfg(test)]
mod tests {
	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	use super::PasswordHash;
	use argon2::{Argon2, PasswordVerifier};
	use claims::{assert_err, assert_ok};
	use fake::Fake;
	use secrecy::{ExposeSecret, Secret};

	#[tokio::test]
	async fn less_than_twelve_fails() -> Result<()> {
		let password = "aB1%".to_string();
		let password = Secret::new(password);
		assert_err!(PasswordHash::parse(password));

		Ok(())
	}

	#[tokio::test]
	async fn more_than_twelve_fails() -> Result<()> {
		let random_count = (256..300).fake::<i64>() as usize;
		let password = "aB1%".repeat(random_count);
		let password = Secret::new(password);
		assert_err!(PasswordHash::parse(password));

		Ok(())
	}

	#[tokio::test]
	async fn no_uppercase_characters_fails() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "ab1%".repeat(random_count);
		let password = Secret::new(password);
		assert_err!(PasswordHash::parse(password));

		Ok(())
	}

	#[tokio::test]
	async fn no_number_characters_fails() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "aBc%".repeat(random_count);
		let password = Secret::new(password);
		assert_err!(PasswordHash::parse(password));

		Ok(())
	}

	#[tokio::test]
	async fn no_special_characters_fails() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "aB15".repeat(random_count);
		let password = Secret::new(password);
		assert_err!(PasswordHash::parse(password));

		Ok(())
	}

	#[tokio::test]
	async fn password_passes() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "aB1%".repeat(random_count);
		let password = Secret::new(password);
		assert_ok!(PasswordHash::parse(password));

		Ok(())
	}

	#[tokio::test]
	async fn parse_hash_correctly() -> Result<()> {
		//-- Setup and Fixtures (Arrange)
		// Generate a password that meets the minimum requirements in parse
		let random_count = (5..30).fake::<i64>() as usize;
		let password_secret = Secret::new("aB1%".repeat(random_count));

		//-- Execute Function (Act)
		// Parse a password hash from the domain
		let password_hash = PasswordHash::parse(password_secret.clone())?;
		println!("{password_hash:#?}");

		//-- Checks (Assertions)
		assert_ok!(password_hash.verify_password(&password_secret));

		Ok(())
	}
}
