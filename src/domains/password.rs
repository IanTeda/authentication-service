//-- ./src/domains/passwords.rs

#![allow(unused)] // For beginning only.

//! Email address domain parsing
//!
//! Parse string into an email address, checking for validation as we go.
//!
//! # References
//!
//! * [NIST password guidelines 2024: 15 rules to follow](https://community.trustcloud.ai/article/nist-password-guidelines-2024-15-rules-to-follow/)
//! * [PassMeRust Password Strength Calculator](https://github.com/dewan-ahmed/PassMeRust/tree/main)
//! ---

use crate::prelude::*;

use secrecy::Secret;

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl Password {
	pub fn parse(password: impl Into<String>) -> Result<Password, BackendError> {
		let password: String = password.into();
		// println!("{password:#?}");

		// Password must be at least 12 characters
		let is_to_short = password.len() < 12;

		// Password must be under 256 characters
		let is_to_long = password.len() > 255;

		// Password must contain an upper case letter
		#[allow(clippy::manual_range_contains)]
		let no_uppercase = !password.bytes().any(|byte| byte >= b'A' && byte <= b'Z');

		// Password must contain a lower case letter
		#[allow(clippy::manual_range_contains)]
		let no_lowercase = !password.bytes().any(|byte| byte >= b'a' && byte <= b'z');

		// Password must contain a number
		#[allow(clippy::manual_range_contains)]
		let no_number = !password.bytes().any(|byte| byte >= b'0' && byte <= b'9');

		// Password must contain a special character
		#[allow(clippy::manual_range_contains)]
		let no_special = !password.bytes().any(|byte| {
			byte < b'0'
				|| (byte > b'9' && byte < b'A')
				|| (byte > b'Z' && byte < b'a')
				|| byte > b'z'
		});

		if is_to_short || is_to_long || no_uppercase || no_lowercase || no_number || no_special {
			Err(BackendError::PasswordFormatInvalid)
		} else {
			Ok(Self(Secret::new(password)))
		}
	}
}

#[cfg(test)]
mod tests {
	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	use super::Password;
	use claims::{assert_err, assert_ok};
	use fake::Fake;

	#[test]
	fn less_than_twelve_fails() -> Result<()> {
		let password = "aB1%";
		assert_err!(Password::parse(password));

		Ok(())
	}

	#[test]
	fn more_than_twelve_fails() -> Result<()> {
		let random_count = (256..300).fake::<i64>() as usize;
		let password = "aB1%".repeat(random_count);
		assert_err!(Password::parse(password));

		Ok(())
	}

	#[test]
	fn no_uppercase_characters_fails() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "ab1%".repeat(random_count);
		assert_err!(Password::parse(password));

		Ok(())
	}

	#[test]
	fn no_number_characters_fails() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "aBc%".repeat(random_count);
		assert_err!(Password::parse(password));

		Ok(())
	}

	#[test]
	fn no_special_characters_fails() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "aB15".repeat(random_count);
		assert_err!(Password::parse(password));

		Ok(())
	}

	#[test]
	fn password_passes() -> Result<()> {
		let random_count = (5..30).fake::<i64>() as usize;
		let password = "aB1%".repeat(random_count);
		assert_ok!(Password::parse(password));

		Ok(())
	}
}
