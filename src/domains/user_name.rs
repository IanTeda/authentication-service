//-- ./src/domains/user_name.rs

// #![allow(unused)] // For beginning only.

//! User name domain parsing
//!
//! Parse string into a name, checking for validation as we go.
//! ---

use crate::prelude::*;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, PartialEq)]
pub struct UserName(String);

/// Implementation of the default Thing for creating a new thing.
impl Default for UserName {
	fn default() -> Self {
		Self("Nil".to_string())
	}
}

impl UserName {
	pub fn parse(name: impl Into<String>) -> Result<UserName, BackendError> {
		let name: String = name.into();

		// `.trim()` returns a view over the input `name` without trailing whitespace-like
		// characters. is_empty` checks if the view contains any character.
		let is_empty_or_whitespace = name.trim().is_empty();

		// A grapheme is defined by the Unicode standard as a "user-perceived"
		// character: `å` is a single grapheme, but it is composed of two characters
		// (`a` and `̊`).
		//
		// `graphemes` returns an iterator over the graphemes in the input `s`.
		// `true` specifies that we want to use the extended grapheme definition set,
		// the recommended one.
		let is_too_long = name.graphemes(true).count() > 256;

		// Iterate over all characters in the input `name` to check if any of them matches
		// one of the characters in the forbidden array.
		let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
		let contains_forbidden_characters =
			name.chars().any(|g| forbidden_characters.contains(&g));

		if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
			Err(BackendError::UserNameFormatInvalid(name))
		} else {
			Ok(Self(name))
		}
	}
}

#[cfg(test)]
mod tests {
	// Override with more flexible error
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>;

	use super::{BackendError, UserName};
	use claims::{assert_err, assert_ok};
	use fake::faker::name::en::Name;
	use fake::Fake;

	#[test]
	fn thing_name_default() -> Result<()> {
		let default_thing_name = UserName::default();
		let check = UserName::parse("Nil")?;
		assert_eq!(default_thing_name, check);

		Ok(())
	}

	#[test]
	fn a_256_grapheme_long_name_is_valid() -> Result<()> {
		let name = "a̐".repeat(256);
		assert_ok!(UserName::parse(name));

		Ok(())
	}

	#[test]
	fn a_name_longer_than_256_graphemes_is_rejected() -> Result<()> {
		let name = "a".repeat(257);
		assert_err!(UserName::parse(name.clone()));
		assert!(matches!(
			UserName::parse(name),
			Err(BackendError::UserNameFormatInvalid { .. })
		));

		Ok(())
	}

	#[test]
	fn whitespace_only_names_are_rejected() -> Result<()> {
		let name = " ".to_string();
		assert_err!(UserName::parse(name.clone()));
		assert!(matches!(
			UserName::parse(name),
			Err(BackendError::UserNameFormatInvalid { .. })
		));

		Ok(())
	}

	#[test]
	fn empty_string_is_rejected() -> Result<()> {
		let name = "".to_string();
		assert_err!(UserName::parse(name.clone()));
		assert!(matches!(
			UserName::parse(name),
			Err(BackendError::UserNameFormatInvalid { .. })
		));

		Ok(())
	}

	#[test]
	fn names_containing_an_invalid_character_are_rejected() -> Result<()> {
		for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
			let name = name.to_string();
			assert_err!(UserName::parse(name.clone()));
			assert!(matches!(
				UserName::parse(name),
				Err(BackendError::UserNameFormatInvalid { .. })
			));
		}

		Ok(())
	}

	#[test]
	fn a_valid_name_is_parsed_successfully() -> Result<()> {
		let name: String = Name().fake();
		assert_ok!(UserName::parse(name));

		Ok(())
	}
}
