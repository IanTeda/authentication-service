//-- ./src/database/email_verification/model.rs

#![allow(unused)] // For development only

//! # Email Verification Model
//!
//! This file defines the `EmailVerification` struct representing an email verification
//! record in the database, along with serialization, deserialization, and database mapping
//! traits.
//!
//! Provides constructors for creating new email verification instances, as well as helper methods for
//! generating mock email verification data for testing purposes.
//!
//! # Contents
//! - `EmailVerification` struct definition
//! - Constructor for new email verification instances
//! - Mock data generation for tests

use crate::{
    database,
    domain::{self, RowID},
};
use uuid::Uuid;

#[derive(
    Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow, Clone, PartialEq,
)]
pub struct EmailVerification {
    pub id: domain::RowID,
    pub user_id: Uuid,
    pub token: domain::EmailVerificationToken,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub used: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl EmailVerification {
    /// Creates a new instance of the EmailVerification struct.
    ///
    /// ## Parameters
    /// - `user: &database::Users` - The user that will be used as a foreign key
    /// - `token: &str` - The verification token
    /// - `expires_at: DateTime<Utc>` - The expiration timestamp for the token
    pub fn new(
        user: &database::Users,
        token: &domain::EmailVerificationToken,
        duration: &chrono::Duration,
    ) -> Self {
        // Generate a new unique identifier for the email verification record.
        let id = RowID::new();

        // Get the user ID from the provided user instance.
        let user_id = user.id;

        // Clone the token to ensure ownership and prevent borrowing issues.
        // This is necessary because the token is a reference in the function signature.
        // Cloning ensures that we have a valid owned instance of the token.
        // This is important for the struct's lifetime and ownership semantics.
        let token = token.clone();

        // Get the now DateTime<Utc>
        let now = chrono::Utc::now();

        // Calculate the the expiration time by adding the duration to the current time.
        let expires_at = now + *duration;

        // Initialize the used status to false, indicating that the token has not been used yet.
        let used = false;

        // Set the creation timestamp to the current time now.
        let created_at = now;

        Self {
            id,
            user_id,
            token,
            expires_at,
            used,
            created_at,
        }
    }
}

#[cfg(test)]
mod unit_validation_tests {
    use super::*;
    use chrono::Duration;
    use fake::{faker::company::en::CompanyName, Fake, Faker};
    use secrecy::SecretString;

    fn mock_user() -> database::Users {
        database::Users::mock_data().unwrap()
    }

    fn mock_secret() -> SecretString {
        let fake_secret: String = Faker.fake();
        SecretString::new(fake_secret.into_boxed_str())
    }

    fn mock_issuer() -> SecretString {
        let fake_company: String = CompanyName().fake();
        SecretString::new(fake_company.into_boxed_str())
    }

    fn mock_token() -> domain::EmailVerificationToken {
        use fake::Fake;
    
        const MIN_HOURS: i64 = 1;
        const MAX_HOURS: i64 = 168; // 7 days
    
        let issuer = mock_issuer();
        let random_hours: i64 = (MIN_HOURS..=MAX_HOURS).fake();
        let duration = chrono::Duration::hours(random_hours);
        let user = mock_user();
        let token_type = &domain::TokenType::EmailVerification;
    
        let claim = domain::TokenClaimNew::new(&issuer, &duration, &user, token_type);
        let secret = mock_secret();
    
        domain::EmailVerificationToken::try_from_claim(claim, &secret)
            .expect("Failed to generate mock email verification token")
    }

    #[test]
    fn test_email_verification_new() {
        let user = mock_user();
        let token = mock_token();
        let duration = Duration::hours(24);

        let verification = EmailVerification::new(&user, &token, &duration);

        assert_eq!(verification.user_id, user.id);
        assert_eq!(verification.token, token);
        assert!(!verification.used);
        assert!(verification.expires_at > verification.created_at);
        assert!(verification.created_at <= chrono::Utc::now());
    }

    #[test]
    fn test_email_verification_new_calculates_correct_expiration() {
        let user = mock_user();
        let token = mock_token();
        let duration = Duration::hours(48);

        let verification = EmailVerification::new(&user, &token, &duration);

        let expected_duration = verification.expires_at - verification.created_at;
        assert_eq!(expected_duration, duration);
    }

    #[test]
    fn test_email_verification_new_with_different_durations() {
        let user = mock_user();
        let token = mock_token();

        let test_durations = vec![
            Duration::minutes(30),
            Duration::hours(1),
            Duration::hours(24),
            Duration::days(1),
            Duration::days(7),
        ];

        for duration in test_durations {
            let verification = EmailVerification::new(&user, &token, &duration);
            
            let actual_duration = verification.expires_at - verification.created_at;
            assert_eq!(actual_duration, duration);
            assert!(!verification.used);
        }
    }

    #[test]
    fn test_email_verification_new_generates_unique_ids() {
        let user = mock_user();
        let token = mock_token();
        let duration = Duration::hours(1);

        let verification1 = EmailVerification::new(&user, &token, &duration);
        let verification2 = EmailVerification::new(&user, &token, &duration);

        assert_ne!(verification1.id, verification2.id);
    }

    #[test]
    fn test_email_verification_new_with_different_users() {
        let user1 = mock_user();
        let user2 = mock_user();
        let token = mock_token();
        let duration = Duration::hours(1);

        let verification1 = EmailVerification::new(&user1, &token, &duration);
        let verification2 = EmailVerification::new(&user2, &token, &duration);

        assert_eq!(verification1.user_id, user1.id);
        assert_eq!(verification2.user_id, user2.id);
        assert_ne!(verification1.user_id, verification2.user_id);
    }

    #[test]
    fn test_email_verification_new_defaults() {
        let user = mock_user();
        let token = mock_token();
        let duration = Duration::hours(1);

        let verification = EmailVerification::new(&user, &token, &duration);

        assert_eq!(verification.used, false);
        assert!(verification.created_at <= chrono::Utc::now());
        assert!(verification.expires_at > verification.created_at);
    }

    #[test]
    fn test_email_verification_new_with_zero_duration() {
        let user = mock_user();
        let token = mock_token();
        let duration = Duration::zero();

        let verification = EmailVerification::new(&user, &token, &duration);

        // With zero duration, expires_at should be approximately equal to created_at
        let time_diff = verification.expires_at - verification.created_at;
        assert!(time_diff <= Duration::seconds(1)); // Allow for small timing differences
    }

    #[test]
    fn test_email_verification_new_with_negative_duration() {
        let user = mock_user();
        let token = mock_token();
        let duration = Duration::hours(-1); // Negative duration

        let verification = EmailVerification::new(&user, &token, &duration);

        // Should create an already-expired token
        assert!(verification.expires_at < verification.created_at);
    }

    #[test]
    fn test_email_verification_new_time_precision() {
        let user = mock_user();
        let token = mock_token();
        let duration = Duration::hours(1);

        let before = chrono::Utc::now();
        let verification = EmailVerification::new(&user, &token, &duration);
        let after = chrono::Utc::now();

        assert!(verification.created_at >= before);
        assert!(verification.created_at <= after);
        assert!(verification.expires_at >= before + duration);
        assert!(verification.expires_at <= after + duration);
    }
}
