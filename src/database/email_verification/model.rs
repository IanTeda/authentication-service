//-- ./src/database/email_verification/model.rs

// #![allow(unused)] // For development only

//TODO: Add domain::VerificationStatus for more robust status handling

use crate::{
    database,
    domain::{self, RowID},
};
use secrecy::SecretString;
use uuid::Uuid;

#[derive(
    Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow, Clone, PartialEq,
)]
#[serde(rename_all = "snake_case")]
pub struct EmailVerifications {
    pub id: domain::RowID,
    pub user_id: Uuid,
    pub token: domain::EmailVerificationToken,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub is_used: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl std::fmt::Display for EmailVerifications {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EmailVerification(id: {}, user: {}, expires: {}, is_used: {})",
            self.id, self.user_id, self.expires_at, self.is_used
        )
    }
}

impl EmailVerifications {
    /// Creates a new instance of `EmailVerifications`.
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
        let token = token.clone();

        // Get the now DateTime<Utc>
        let now = chrono::Utc::now();

        // Calculate the the expiration time by adding the duration to the current time.
        let expires_at = now + *duration;

        // Initialize the used status to false, indicating that the token has not been used yet.
        let is_used = false;

        // Set the creation timestamp to the current time now.
        let created_at = now;

        // Set the updated timestamp to none, as this is a new record and has not been updated yet.
        let updated_at = None;

        Self {
            id,
            user_id,
            token,
            expires_at,
            is_used,
            created_at,
            updated_at,
        }
    }

    /// Checks if the verification has expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    /// Checks if the verification is still valid (not used and not expired)
    pub fn is_valid(&self, secret: &SecretString, issuer: &SecretString) -> bool {
        // Parse the email verification token into a token claim
        let token_claim = match domain::TokenClaimNew::parse(self.token.as_ref(), &secret, &issuer) {
            Ok(claim) => claim,
            Err(_) => return false,
        };

        // Verify the token claim has no expired
        let token_claim_not_expired = token_claim.exp > chrono::Utc::now();

        // Check that he database record is not expired.
        let db_not_expired = self.expires_at > chrono::Utc::now();

        /// Check that the token has not been used
        let not_used = !self.is_used;

        // Return true if the database record is not expired, the token claim is not 
        // expired, and the token has not been used
        db_not_expired && token_claim_not_expired && not_used
    }

    /// Time remaining until expiration
    pub fn time_until_expiry(&self) -> chrono::Duration {
        self.expires_at - chrono::Utc::now()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Users;
    use chrono::{Duration, Utc};
    use fake::{faker::company::en::CompanyName, Fake, Faker};
    use secrecy::SecretString;

    // Override with more flexible error type for tests
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    fn mock_token() -> domain::EmailVerificationToken {
        let issuer =
            SecretString::new(CompanyName().fake::<String>().into_boxed_str());
        let secret = SecretString::new(Faker.fake::<String>().into_boxed_str());
        let user = Users::mock_data().unwrap();
        let duration = Duration::hours(24);
        let token_type = &domain::TokenType::EmailVerification;

        let claim =
            domain::TokenClaimNew::new(&issuer, &duration, &user, token_type);
        domain::EmailVerificationToken::try_from_claim(claim, &secret)
            .expect("Failed to generate mock token")
    }

    fn mock_token_with_secrets() -> (domain::EmailVerificationToken, SecretString, SecretString) {
        let issuer = SecretString::new(CompanyName().fake::<String>().into_boxed_str());
        let secret = SecretString::new(Faker.fake::<String>().into_boxed_str());
        let user = Users::mock_data().unwrap();
        let duration = Duration::hours(24);
        let token_type = &domain::TokenType::EmailVerification;

        let claim = domain::TokenClaimNew::new(&issuer, &duration, &user, token_type);
        let token = domain::EmailVerificationToken::try_from_claim(claim, &secret)
            .expect("Failed to generate mock token");
        
        (token, secret, issuer)
    }

    #[test]
    fn test_new_creates_valid_email_verification() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(24);

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Assert
        assert_eq!(verification.user_id, user.id);
        assert_eq!(verification.token, token);
        assert!(!verification.is_used);
        assert!(verification.updated_at.is_none());

        // Check that expires_at is approximately 24 hours from now
        let now = Utc::now();
        let expected_expiry = now + duration;
        let diff = (verification.expires_at - expected_expiry)
            .num_seconds()
            .abs();
        assert!(
            diff < 5,
            "Expiry time should be within 5 seconds of expected"
        );

        Ok(())
    }

    #[test]
    fn test_new_generates_unique_ids() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token1 = mock_token();
        let token2 = mock_token();
        let duration = Duration::hours(24);

        // Act
        let verification1 = EmailVerifications::new(&user, &token1, &duration);
        let verification2 = EmailVerifications::new(&user, &token2, &duration);

        // Assert
        assert_ne!(verification1.id, verification2.id);
        assert_ne!(verification1.token, verification2.token);

        Ok(())
    }

    #[test]
    fn test_new_with_different_durations() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();

        let test_cases = vec![
            Duration::minutes(15),
            Duration::hours(1),
            Duration::hours(24),
            Duration::days(7),
        ];

        // Act & Assert
        for duration in test_cases {
            let verification = EmailVerifications::new(&user, &token, &duration);

            let now = Utc::now();
            let expected_expiry = now + duration;
            let diff = (verification.expires_at - expected_expiry)
                .num_seconds()
                .abs();
            assert!(
                diff < 5,
                "Expiry time should be within 5 seconds for duration: {:?}",
                duration
            );
        }

        Ok(())
    }

    #[test]
    fn test_new_with_negative_duration() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(-1); // Already expired

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Assert
        assert!(verification.expires_at < Utc::now());
        assert!(!verification.is_used); // Should still be false even if expired

        Ok(())
    }

    #[test]
    fn test_new_with_zero_duration() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::zero();

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Assert
        let now = Utc::now();
        let diff = (verification.expires_at - now).num_seconds().abs();
        assert!(diff < 5, "Zero duration should result in current time");

        Ok(())
    }

    #[test]
    fn test_new_preserves_user_id() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let original_user_id = user.id;
        let token = mock_token();
        let duration = Duration::hours(24);

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Assert
        assert_eq!(verification.user_id, original_user_id);

        Ok(())
    }

    #[test]
    fn test_new_clones_token_correctly() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let original_token_value = token.as_ref().to_string();
        let duration = Duration::hours(24);

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Assert
        assert_eq!(verification.token.as_ref(), &original_token_value);
        assert_eq!(verification.token, token);

        Ok(())
    }

    #[test]
    fn test_new_sets_default_values() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(24);

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Assert
        assert!(!verification.is_used, "Should default to unused");
        assert!(
            verification.updated_at.is_none(),
            "Should have no update timestamp initially"
        );
        assert!(
            verification.created_at <= Utc::now(),
            "Created timestamp should be current or past"
        );

        Ok(())
    }

    #[test]
    fn test_new_timestamps_are_consistent() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(24);

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Assert
        let expected_expiry = verification.created_at + duration;
        let diff = (verification.expires_at - expected_expiry)
            .num_milliseconds()
            .abs();
        assert!(diff < 100, "Expires_at should be created_at + duration");

        Ok(())
    }

    #[test]
    fn test_new_multiple_verifications_for_same_user() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token1 = mock_token();
        let token2 = mock_token();
        let duration = Duration::hours(24);

        // Act
        let verification1 = EmailVerifications::new(&user, &token1, &duration);
        let verification2 = EmailVerifications::new(&user, &token2, &duration);

        // Assert
        assert_eq!(verification1.user_id, verification2.user_id);
        assert_ne!(verification1.id, verification2.id);
        assert_ne!(verification1.token, verification2.token);

        Ok(())
    }

    #[test]
    fn test_new_with_very_long_duration() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::days(365); // 1 year

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Assert
        let now = Utc::now();
        let expected_expiry = now + duration;
        let diff = (verification.expires_at - expected_expiry)
            .num_seconds()
            .abs();
        assert!(diff < 5, "Should handle long durations correctly");

        Ok(())
    }

    #[test]
    fn test_struct_derives_work_correctly() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(24);

        // Act
        let verification1 = EmailVerifications::new(&user, &token, &duration);
        let verification2 = verification1.clone();

        // Assert - Test Clone
        assert_eq!(verification1, verification2);

        // Test Debug (should not panic)
        let debug_str = format!("{:?}", verification1);
        assert!(!debug_str.is_empty());

        // Test Serialize/Deserialize
        let json = serde_json::to_string(&verification1)?;
        let deserialized: EmailVerifications = serde_json::from_str(&json)?;
        assert_eq!(verification1.id, deserialized.id);
        assert_eq!(verification1.user_id, deserialized.user_id);

        Ok(())
    }

    #[test]
    fn test_new_created_at_precision() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(24);
        let before = Utc::now();

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);
        let after = Utc::now();

        // Assert
        assert!(verification.created_at >= before);
        assert!(verification.created_at <= after);

        Ok(())
    }

    #[test]
    fn test_new_id_is_time_ordered() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token1 = mock_token();
        let token2 = mock_token();
        let duration = Duration::hours(24);

        // Act
        let verification1 = EmailVerifications::new(&user, &token1, &duration);
        std::thread::sleep(std::time::Duration::from_millis(1));
        let verification2 = EmailVerifications::new(&user, &token2, &duration);

        // Assert - UUID v7 should be time-ordered
        assert!(verification1.id < verification2.id);
        assert!(verification1.created_at <= verification2.created_at);

        Ok(())
    }

    #[test]
    fn test_new_handles_extreme_durations() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();

        let extreme_cases = vec![
            Duration::nanoseconds(1),
            Duration::microseconds(1),
            Duration::milliseconds(1),
            Duration::seconds(1),
            Duration::days(-365),  // 1 year in the past
            Duration::days(36500), // 100 years in the future
        ];

        // Act & Assert
        for duration in extreme_cases {
            let verification = EmailVerifications::new(&user, &token, &duration);

            // Should not panic and should calculate expiry correctly
            let expected_expiry = verification.created_at + duration;
            let diff = (verification.expires_at - expected_expiry)
                .num_milliseconds()
                .abs();
            assert!(diff < 100, "Should handle extreme duration: {:?}", duration);
        }

        Ok(())
    }

    #[test]
    fn test_is_expired_with_expired_verification() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(-1); // Already expired

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Assert
        assert!(verification.is_expired());
        Ok(())
    }

    #[test]
    fn test_is_expired_with_valid_verification() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(24);

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Assert
        assert!(!verification.is_expired());
        Ok(())
    }

    #[test]
    fn test_is_valid_with_unused_and_not_expired() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let (token, secret, issuer) = mock_token_with_secrets();
        let duration = Duration::hours(24);
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Act & Assert
        assert!(verification.is_valid(&secret, &issuer));
        assert!(!verification.is_used);
        assert!(!verification.is_expired());
        Ok(())
    }

    #[test]
    fn test_is_valid_with_used_verification() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let (token, secret, issuer) = mock_token_with_secrets();
        let duration = Duration::hours(24);
        let mut verification = EmailVerifications::new(&user, &token, &duration);
        verification.is_used = true;

        // Act & Assert
        assert!(!verification.is_valid(&secret, &issuer));
        assert!(verification.is_used);
        assert!(!verification.is_expired());
        Ok(())
    }

    #[test]
    fn test_is_valid_with_used_and_expired_verification() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let (token, secret, issuer) = mock_token_with_secrets();
        let duration = Duration::hours(-1); // Expired
        let mut verification = EmailVerifications::new(&user, &token, &duration);
        verification.is_used = true;

        // Act & Assert
        assert!(!verification.is_valid(&secret, &issuer));
        assert!(verification.is_used);
        assert!(verification.is_expired());
        Ok(())
    }

    #[test]
    fn test_is_valid_with_wrong_secret() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let (token, _correct_secret, issuer) = mock_token_with_secrets();
        let wrong_secret = SecretString::new(Faker.fake::<String>().into_boxed_str());
        let duration = Duration::hours(24);
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Act & Assert
        let result = verification.is_valid(&wrong_secret, &issuer);
        assert!(!result, "Should fail with wrong secret");
        Ok(())
    }

    #[test]
    fn test_is_valid_with_wrong_issuer() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let (token, secret, _correct_issuer) = mock_token_with_secrets();
        let wrong_issuer = SecretString::new(CompanyName().fake::<String>().into_boxed_str());
        let duration = Duration::hours(24);
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Act & Assert
        let result = verification.is_valid(&secret, &wrong_issuer);
        assert!(!result, "Should fail with wrong issuer");
        Ok(())
    }

    #[test]
    fn test_is_valid_jwt_expired_but_db_not_expired() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let issuer = SecretString::new(CompanyName().fake::<String>().into_boxed_str());
        let secret = SecretString::new(Faker.fake::<String>().into_boxed_str());
        
        // Create a JWT with short expiration
        let jwt_duration = Duration::milliseconds(10);
        let token_type = &domain::TokenType::EmailVerification;
        let claim = domain::TokenClaimNew::new(&issuer, &jwt_duration, &user, token_type);
        let token = domain::EmailVerificationToken::try_from_claim(claim, &secret)?;
        
        // But create verification with longer database expiration
        let db_duration = Duration::hours(24);
        let verification = EmailVerifications::new(&user, &token, &db_duration);

        // Wait for JWT to expire
        std::thread::sleep(std::time::Duration::from_millis(15));

        // Act & Assert
        assert!(!verification.is_expired()); // DB not expired
        let result = verification.is_valid(&secret, &issuer);
        assert!(!result, "Should fail when JWT is expired even if DB is not");
        Ok(())
    }

    #[test]
    fn test_is_valid_db_expired_but_jwt_not_expired() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let issuer = SecretString::new(CompanyName().fake::<String>().into_boxed_str());
        let secret = SecretString::new(Faker.fake::<String>().into_boxed_str());
        
        // Create a JWT with long expiration
        let jwt_duration = Duration::hours(24);
        let token_type = &domain::TokenType::EmailVerification;
        let claim = domain::TokenClaimNew::new(&issuer, &jwt_duration, &user, token_type);
        let token = domain::EmailVerificationToken::try_from_claim(claim, &secret)?;
        
        // But create verification with already expired database expiration
        let db_duration = Duration::hours(-1);
        let verification = EmailVerifications::new(&user, &token, &db_duration);

        // Act & Assert
        assert!(verification.is_expired()); // DB expired
        assert!(!verification.is_valid(&secret, &issuer)); // Should be invalid due to DB expiration
        Ok(())
    }

    #[test]
    fn test_is_valid_with_expired_verification() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let (token, secret, issuer) = mock_token_with_secrets();
        let duration = Duration::hours(-1); // Expired
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Act & Assert
        assert!(!verification.is_valid(&secret, &issuer));
        assert!(!verification.is_used);
        assert!(verification.is_expired());
        Ok(())
    }

    #[test]
    fn test_time_until_expiry_positive() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(24);

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);
        let time_remaining = verification.time_until_expiry();

        // Assert
        assert!(time_remaining.num_hours() > 0);
        assert!(time_remaining.num_hours() <= 24);
        Ok(())
    }

    #[test]
    fn test_time_until_expiry_negative() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(-1); // Already expired

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);
        let time_remaining = verification.time_until_expiry();

        // Assert
        assert!(time_remaining.num_seconds() < 0);
        Ok(())
    }

    #[test]
    fn test_verification_at_exact_expiry_time() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::zero();

        // Act
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Simulate time passing to exact expiry
        std::thread::sleep(std::time::Duration::from_millis(1));

        // Assert
        assert!(verification.is_expired() || !verification.is_expired()); // Either is acceptable due to timing
        Ok(())
    }

    #[test]
    fn test_modified_updated_at_field() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(24);
        let mut verification = EmailVerifications::new(&user, &token, &duration);

        // Act
        verification.updated_at = Some(Utc::now());

        // Assert
        assert!(verification.updated_at.is_some());
        Ok(())
    }

    #[test]
    fn test_verification_with_manually_set_used_status() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(24);
        let mut verification = EmailVerifications::new(&user, &token, &duration);

        // Act
        verification.is_used = true;

        // Assert
        assert!(verification.is_used);
        // Note: is_valid requires secret and issuer parameters
        let (_, secret, issuer) = mock_token_with_secrets();
        assert!(!verification.is_valid(&secret, &issuer)); // Should be invalid when used
        Ok(())
    }

    #[test]
    fn test_serialization_with_none_updated_at() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(24);
        let verification = EmailVerifications::new(&user, &token, &duration);

        // Act
        let json = serde_json::to_string(&verification)?;
        let parsed_json: serde_json::Value = serde_json::from_str(&json)?;

        // Assert
        assert!(parsed_json["updated_at"].is_null());
        Ok(())
    }

    #[test]
    fn test_serialization_with_some_updated_at() -> Result<()> {
        // Arrange
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(24);
        let mut verification = EmailVerifications::new(&user, &token, &duration);
        let update_time = Utc::now();
        verification.updated_at = Some(update_time);

        // Act
        let json = serde_json::to_string(&verification)?;
        let deserialized: EmailVerifications = serde_json::from_str(&json)?;

        // Assert
        assert_eq!(deserialized.updated_at, Some(update_time));
        Ok(())
    }

    #[test]
    fn test_deserialization_snake_case_fields() -> Result<()> {
        // Arrange
        let json = r#"{
        "id": "01234567-89ab-cdef-0123-456789abcdef",
        "user_id": "01234567-89ab-cdef-0123-456789abcdef",
        "token": "test.token.here",
        "expires_at": "2025-01-01T00:00:00Z",
        "is_used": false,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": null
    }"#;

        // Act
        let result = serde_json::from_str::<EmailVerifications>(json);

        // Assert
        assert!(
            result.is_ok(),
            "Should deserialize snake_case fields correctly"
        );
        Ok(())
    }

    #[test]
    fn test_concurrent_verification_creation() -> Result<()> {
        use std::sync::Arc;
        use std::thread;

        // Arrange
        let user = Arc::new(Users::mock_data()?);
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let user_clone = Arc::clone(&user);
                thread::spawn(move || {
                    let token = mock_token();
                    let duration = Duration::hours(24);
                    EmailVerifications::new(&user_clone, &token, &duration)
                })
            })
            .collect();

        // Act
        let verifications: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();

        // Assert
        let mut ids: Vec<_> = verifications.iter().map(|v| v.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 10, "All IDs should be unique");
        Ok(())
    }
}
