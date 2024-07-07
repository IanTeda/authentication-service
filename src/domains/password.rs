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

use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use secrecy::{ExposeSecret, Secret };

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, derive_more::From)]
pub struct Password(String);

impl Password {
	/// Parse `String` into a hashed password returning it in a Secret type.
	///
	/// # Parameters
	///
	/// * `password`: The password in a string
	/// ---
	pub fn parse(password: Secret<String>) -> Result<Password, BackendError> {
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
		if is_to_short || is_to_long || no_uppercase || no_lowercase || no_number || no_special {
			Err(BackendError::PasswordFormatInvalid)
		} else {
			let password_hash = compute_password_hash_and_verify(password)?;
			Ok(Self(password_hash))
		}
	}

	// Function to get the `Secret` within the Password struct.
	// pub fn inner(self) -> String {
	// 	// The caller gets the inner string, but they do not have a SubscriberName anymore!
	// 	// That's because `inner` takes `self` by value, consuming it according to move semantics
	// 	self.0
	// }
}

impl AsRef<str> for Password {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl std::fmt::Display for Password {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

/// Compute the password hash, checking it as we go, returning the password hash within
/// a Secret type.
///
/// # Parameters
///
/// * `password`: The password String within a Secret type
/// ---
pub fn compute_password_hash_and_verify(password: Secret<String>) -> Result<String, BackendError> {
	// Generate encryption salt hash
	let salt = SaltString::generate(&mut rand::thread_rng());

	// Initiate new Argon2 instance
	let argon2 = Argon2::new(
		Algorithm::Argon2id,
		Version::V0x13,
		Params::new(15000, 2, 1, None).unwrap(),
	);

	// Hash password to PHC string ($argon2id$v=19$...)
	let password_hash = argon2
		.hash_password(password.expose_secret().as_bytes(), &salt)
		.unwrap()
		.to_string();

	// unimplemented!()
	// Verify password hash
	if verify_password_hash(&password, &password_hash)? {
		Ok(password_hash)
	} else {
		Err(BackendError::PasswordParseError)
	}
}

/// Verify password hash (i.e. verify password)
///
/// # Parameters
///
/// * `password`: Password in a String to check against the hash
/// * `password_hash`: Password hash to check against
/// ---
pub fn verify_password_hash(
	password: &Secret<String>,
	password_hash: &str,
) -> Result<bool, BackendError> {
	// Initiate new Argon2 instance
	let argon2 = Argon2::new(
		Algorithm::Argon2id,
		Version::V0x13,
		Params::new(15000, 2, 1, None).unwrap(),
	);

	// Verify password hash
	let parsed_hash = PasswordHash::new(password_hash).unwrap();

	// Argon2::default().verify_password(password, &parsed_hash)
	let verified = argon2
		.verify_password(password.expose_secret().as_bytes(), &parsed_hash)
		.is_ok();

	// unimplemented!()
	Ok(verified)
}

#[cfg(test)]
mod tests {
	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	use super::Password;
	use argon2::{Argon2, PasswordHash, PasswordVerifier};
	use claims::{assert_err, assert_ok};
	use fake::Fake;
use secrecy::{ExposeSecret, Secret};

	#[test]
	fn less_than_twelve_fails() -> Result<()> {
		let password = "aB1%".to_string();
		let password = Secret::new(password);
		assert_err!(Password::parse(password));

		Ok(())
	}

	#[test]
	fn more_than_twelve_fails() -> Result<()> {
		let random_count = (256..300).fake::<i64>() as usize;
		let password = "aB1%".repeat(random_count);
		let password = Secret::new(password);
		assert_err!(Password::parse(password));

		Ok(())
	}

	#[test]
	fn no_uppercase_characters_fails() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "ab1%".repeat(random_count);
		let password = Secret::new(password);
		assert_err!(Password::parse(password));

		Ok(())
	}

	#[test]
	fn no_number_characters_fails() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "aBc%".repeat(random_count);
		let password = Secret::new(password);
		assert_err!(Password::parse(password));

		Ok(())
	}

	#[test]
	fn no_special_characters_fails() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "aB15".repeat(random_count);
		let password = Secret::new(password);
		assert_err!(Password::parse(password));

		Ok(())
	}

	#[test]
	fn password_passes() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "aB1%".repeat(random_count);
		let password = Secret::new(password);
		assert_ok!(Password::parse(password));

		Ok(())
	}

	#[test]
	fn parse_hash_correctly() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password_original = "aB1%".repeat(random_count);
		let password_original = Secret::new(password_original);

		let password_hash = Password::parse(password_original.clone())?;
		let password_string = password_hash.as_ref();
		let parsed_hash = PasswordHash::new(password_string).unwrap();
		assert!(Argon2::default()
			.verify_password(password_original.expose_secret().as_bytes(), &parsed_hash)
			.is_ok());

		Ok(())
	}
}
