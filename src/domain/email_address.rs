//-- ./src/domains/email_address.rs

// #![allow(unused)] // For beginning only.

//! Email address domain parsing
//!
//! Parse string into an email address, checking for validation as we go.
//! ---

use crate::prelude::*;

use serde::{Deserialize, Serialize};
use sqlx::Decode;
use validator::ValidateEmail;

#[derive(
    Debug, Clone, Serialize, Deserialize, Decode, PartialEq, derive_more::From,
)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Returns a Result of EmailAddress if the input satisfies all our validation
    /// constraints
    pub fn parse(email: impl Into<String>) -> Result<EmailAddress, BackendError> {
        let email = email.into();
        if email.trim().is_empty() {
            return Err(BackendError::EmailIsEmpty);
        }

        if email.validate_email() {
            Ok(Self(email))
        } else {
            Err(BackendError::EmailFormatInvalid(email))
        }
    }

    #[cfg(test)]
    pub fn mock_data() -> Result<Self, BackendError> {
        use fake::faker::internet::en::SafeEmail;
        use fake::Fake;

        let random_email: String = SafeEmail().fake();

        EmailAddress::parse(random_email)
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::EmailAddress;
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(EmailAddress::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ianteda.com".to_string();
        assert_err!(EmailAddress::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(EmailAddress::parse(email));
    }

    #[test]
    fn valid_email_parsed() {
        let email: String = SafeEmail().fake();
        assert!(EmailAddress::parse(email).is_ok());
    }
}
