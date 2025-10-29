//-- ./src/database/email_verification/insert.rs

// #![allow(unused)] // For development only

use crate::{database::EmailVerifications, AuthenticationError};

impl EmailVerifications {
    #[tracing::instrument(
        name = "Insert email verification",
        skip(database),
        fields(
            verification_id = %self.id,
            user_id = %self.user_id,
            expires_at = %self.expires_at,
            is_used = %self.is_used
        )
    )]
    pub async fn insert(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, AuthenticationError> {
        let query = sqlx::query_as!(
            EmailVerifications,
            r#"
                INSERT INTO email_verifications (id, user_id, token, expires_at, is_used, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, NULL)
                RETURNING *
            "#,
            self.id.into_uuid(),
            self.user_id,
            self.token.as_ref(),
            self.expires_at,
            self.is_used,
            self.created_at
        )
        .fetch_one(database)
        .await;

        match query {
            Ok(email_verification) => Ok(email_verification),
            Err(e) => {
                tracing::error!(
                    "Failed to insert email verification: {} (verification: {:?})",
                    e,
                    self
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(
        name = "Insert email verifications batch in database",
        skip(database, verifications),
        fields(
            total_verifications = verifications.len(),
            operation = "insert_batch"
        )
    )]
    pub async fn insert_batch(
        verifications: &[EmailVerifications],
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<EmailVerifications>, AuthenticationError> {
        // Input validation
        if verifications.is_empty() {
            tracing::warn!(
                verifications_count = 0,
                "Attempted to insert empty batch of email verifications"
            );
            return Err(AuthenticationError::ValidationError("Cannot insert empty batch. Use insert() for single records."
                        .to_string()));
        }

        tracing::info!(
            verifications_count = verifications.len(),
            batch_size = verifications.len(),
            "Starting email verification batch insert transaction"
        );

        const MAX_BATCH_SIZE: usize = 1000;
        if verifications.len() > MAX_BATCH_SIZE {
            return Err(AuthenticationError::ValidationError(format!(
                    "Batch size {} exceeds limit of {}",
                    verifications.len(),
                    MAX_BATCH_SIZE
                )));
        }

        let mut tx = database.begin().await?;
        let mut inserted = Vec::with_capacity(verifications.len());

        for verification in verifications {
            let db_record = sqlx::query_as!(
                EmailVerifications,
                r#"
                    INSERT INTO email_verifications (id, user_id, token, expires_at, is_used, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, NULL)
                    RETURNING *
                "#,
                verification.id.into_uuid(),
                verification.user_id,
                verification.token.as_ref(),
                verification.expires_at,
                verification.is_used,
                verification.created_at
            )
            .fetch_one(&mut *tx)
            .await?;

            inserted.push(db_record);
        }

        tx.commit().await?;
        Ok(inserted)
    }

    #[tracing::instrument(
        name = "Upsert email verification",
        skip(database),
        fields(
            verification_id = %self.id,
            user_id = %self.user_id,
            expires_at = %self.expires_at,
            is_used = %self.is_used
        )
    )]
    pub async fn upsert(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, AuthenticationError> {
        let query = sqlx::query_as!(
        EmailVerifications,
        r#"
            INSERT INTO email_verifications (id, user_id, token, expires_at, is_used, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NULL)
            ON CONFLICT (id) DO UPDATE
            SET user_id = EXCLUDED.user_id,
                token = EXCLUDED.token,
                expires_at = EXCLUDED.expires_at,
                is_used = EXCLUDED.is_used,
                updated_at = NOW()
            RETURNING *
        "#,
        self.id.into_uuid(),
        self.user_id,
        self.token.as_ref(),
        self.expires_at,
        self.is_used,
        self.created_at
        )
        .fetch_one(database)
        .await;

        match query {
            Ok(email_verification) => Ok(email_verification),
            Err(e) => {
                tracing::error!(
                    "Failed to insert email verification: {} (verification: {:?})",
                    e,
                    self
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }
}

//-- Unit Tests
#[cfg(test)]
pub mod unit_tests {
    use super::*;
    use crate::{database::Users, domain};
    use chrono::Duration;
    use sqlx::PgPool;

    // Override with more flexible error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    fn mock_token() -> domain::EmailVerificationToken {
        use fake::{faker::company::en::CompanyName, Fake, Faker};
        use secrecy::SecretString;

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

    #[sqlx::test]
    async fn test_insert_success(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token = mock_token();
        let duration = Duration::hours(24);
        let verification = EmailVerifications::new(&user, &token, &duration);

        //-- Execute Function (Act)
        let database_record = verification.insert(&pool).await?;

        //-- Checks (Assertions)
        assert_eq!(database_record.id, verification.id);
        assert_eq!(database_record.user_id, verification.user_id);
        assert_eq!(database_record.token.as_ref(), verification.token.as_ref());
        assert_eq!(
            database_record.expires_at.timestamp_millis(),
            verification.expires_at.timestamp_millis()
        );
        assert_eq!(database_record.is_used, verification.is_used);
        assert_eq!(
            database_record.created_at.timestamp_millis(),
            verification.created_at.timestamp_millis()
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_duplicate_id_fails(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token1 = mock_token();
        let token2 = mock_token();
        let duration = Duration::hours(24);

        let verification1 = EmailVerifications::new(&user, &token1, &duration);
        let verification2 = EmailVerifications {
            id: verification1.id, // Same ID
            token: token2,
            user_id: verification1.user_id,
            expires_at: verification1.expires_at,
            is_used: false,
            created_at: verification1.created_at,
            updated_at: None,
        };

        //-- Execute Function (Act)
        verification1.insert(&pool).await?;
        let result = verification2.insert(&pool).await;

        //-- Checks (Assertions)
        assert!(result.is_err(), "Inserting duplicate ID should fail");

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_duplicate_token_fails(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token = mock_token();
        let duration = Duration::hours(24);

        let verification1 = EmailVerifications::new(&user, &token, &duration);
        let verification2 = EmailVerifications::new(&user, &token, &duration); // Same token

        //-- Execute Function (Act)
        verification1.insert(&pool).await?;
        let result = verification2.insert(&pool).await;

        //-- Checks (Assertions)
        assert!(result.is_err(), "Inserting duplicate token should fail");

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_nonexistent_user_fails(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        // Don't insert user into database

        let token = mock_token();
        let duration = Duration::hours(24);
        let verification = EmailVerifications::new(&user, &token, &duration);

        //-- Execute Function (Act)
        let result = verification.insert(&pool).await;

        //-- Checks (Assertions)
        assert!(
            result.is_err(),
            "Inserting with nonexistent user should fail"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_with_expired_token(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token = mock_token();
        let duration = Duration::hours(-1); // Already expired
        let verification = EmailVerifications::new(&user, &token, &duration);

        //-- Execute Function (Act)
        let db_record = verification.insert(&pool).await?;

        //-- Checks (Assertions)
        assert!(db_record.expires_at < chrono::Utc::now());
        assert!(!db_record.is_used);

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_with_used_status(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token = mock_token();
        let duration = Duration::hours(24);
        let mut verification = EmailVerifications::new(&user, &token, &duration);
        verification.is_used = true;

        //-- Execute Function (Act)
        let db_record = verification.insert(&pool).await?;

        //-- Checks (Assertions)
        assert!(db_record.is_used);

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_multiple_verifications_same_user(
        pool: PgPool,
    ) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token1 = mock_token();
        let token2 = mock_token();
        let duration = Duration::hours(24);

        let verification1 = EmailVerifications::new(&user, &token1, &duration);
        let verification2 = EmailVerifications::new(&user, &token2, &duration);

        //-- Execute Function (Act)
        let db_record1 = verification1.insert(&pool).await?;
        let db_record2 = verification2.insert(&pool).await?;

        //-- Checks (Assertions)
        assert_eq!(db_record1.user_id, db_record2.user_id);
        assert_ne!(db_record1.id, db_record2.id);
        assert_ne!(db_record1.token.as_ref(), db_record2.token.as_ref());

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_preserves_timestamps(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token = mock_token();
        let duration = Duration::hours(24);
        let verification = EmailVerifications::new(&user, &token, &duration);

        let original_created_at = verification.created_at;
        let original_expires_at = verification.expires_at;

        //-- Execute Function (Act)
        let db_record = verification.insert(&pool).await?;

        //-- Checks (Assertions)
        assert_eq!(
            db_record.created_at.timestamp_millis(),
            original_created_at.timestamp_millis()
        );
        assert_eq!(
            db_record.expires_at.timestamp_millis(),
            original_expires_at.timestamp_millis()
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_batch_success(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let verifications = vec![
            EmailVerifications::new(&user, &mock_token(), &Duration::hours(24)),
            EmailVerifications::new(&user, &mock_token(), &Duration::hours(48)),
            EmailVerifications::new(&user, &mock_token(), &Duration::hours(72)),
        ];

        //-- Execute Function (Act)
        let inserted =
            EmailVerifications::insert_batch(&verifications, &pool).await?;

        //-- Checks (Assertions)
        assert_eq!(inserted.len(), 3);
        for (original, inserted) in verifications.iter().zip(inserted.iter()) {
            assert_eq!(original.id, inserted.id);
            assert_eq!(original.user_id, inserted.user_id);
            assert_eq!(original.token.as_ref(), inserted.token.as_ref());
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_batch_rollback_on_failure(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token = mock_token();
        let duration = Duration::hours(24);
        let verification1 = EmailVerifications::new(&user, &token, &duration);
        let verification2 = EmailVerifications::new(&user, &token, &duration); // Duplicate token

        let verifications = vec![verification1, verification2];

        //-- Execute Function (Act)
        let result = EmailVerifications::insert_batch(&verifications, &pool).await;

        //-- Checks (Assertions)
        assert!(
            result.is_err(),
            "Batch insert should fail on duplicate token"
        );

        // Verify no records were inserted (transaction rolled back)
        let count =
            sqlx::query!("SELECT COUNT(*) as count FROM email_verifications")
                .fetch_one(&pool)
                .await?
                .count
                .unwrap_or(0);
        assert_eq!(count, 0, "No records should be inserted after rollback");

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_batch_empty(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let verifications: Vec<EmailVerifications> = vec![];

        //-- Execute Function (Act)
        let inserted =
            EmailVerifications::insert_batch(&verifications, &pool).await?;

        //-- Checks (Assertions)
        assert_eq!(inserted.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_batch_large_set(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let mut verifications = Vec::new();
        for _ in 0..10 {
            verifications.push(EmailVerifications::new(
                &user,
                &mock_token(),
                &Duration::hours(24),
            ));
        }

        //-- Execute Function (Act)
        let inserted =
            EmailVerifications::insert_batch(&verifications, &pool).await?;

        //-- Checks (Assertions)
        assert_eq!(inserted.len(), 10);

        // Verify all have unique IDs and tokens
        let mut ids: Vec<_> = inserted.iter().map(|v| v.id).collect();
        let mut tokens: Vec<_> = inserted.iter().map(|v| v.token.as_ref()).collect();
        ids.sort();
        tokens.sort();
        ids.dedup();
        tokens.dedup();
        assert_eq!(ids.len(), 10);
        assert_eq!(tokens.len(), 10);

        Ok(())
    }

    #[sqlx::test]
    async fn test_upsert_insert_new_record(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token = mock_token();
        let duration = Duration::hours(24);
        let verification = EmailVerifications::new(&user, &token, &duration);

        //-- Execute Function (Act)
        let db_record = verification.upsert(&pool).await?;

        //-- Checks (Assertions)
        assert_eq!(db_record.id, verification.id);
        assert_eq!(db_record.user_id, verification.user_id);
        assert_eq!(db_record.token.as_ref(), verification.token.as_ref());

        Ok(())
    }

    #[sqlx::test]
    async fn test_upsert_update_existing_record(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token1 = mock_token();
        let token2 = mock_token();
        let duration = Duration::hours(24);
        let verification1 = EmailVerifications::new(&user, &token1, &duration);

        // Insert first record
        verification1.insert(&pool).await?;

        // Create second record with same ID but different token
        let verification2 = EmailVerifications {
            id: verification1.id, // Same ID
            token: token2,        // Different token
            user_id: verification1.user_id,
            expires_at: verification1.expires_at + Duration::hours(24), // Different expiry
            is_used: true, // Different used status
            created_at: verification1.created_at,
            updated_at: None,
        };

        //-- Execute Function (Act)
        let db_record = verification2.upsert(&pool).await?;

        //-- Checks (Assertions)
        assert_eq!(db_record.id, verification1.id);
        assert_eq!(db_record.token.as_ref(), verification2.token.as_ref());
        assert_eq!(
            db_record.expires_at.timestamp_millis(),
            verification2.expires_at.timestamp_millis()
        );
        assert_eq!(db_record.is_used, verification2.is_used);

        Ok(())
    }

    #[sqlx::test]
    async fn test_upsert_nonexistent_user_fails(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        // Don't insert user into database

        let token = mock_token();
        let duration = Duration::hours(24);
        let verification = EmailVerifications::new(&user, &token, &duration);

        //-- Execute Function (Act)
        let result = verification.upsert(&pool).await;

        //-- Checks (Assertions)
        assert!(
            result.is_err(),
            "Upserting with nonexistent user should fail"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_and_upsert_consistency(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token = mock_token();
        let duration = Duration::hours(24);
        let verification = EmailVerifications::new(&user, &token, &duration);

        //-- Execute Function (Act)
        let insert_result = verification.insert(&pool).await?;
        let upsert_result = verification.upsert(&pool).await?;

        //-- Checks (Assertions)
        assert_eq!(insert_result.id, upsert_result.id);
        assert_eq!(insert_result.token.as_ref(), upsert_result.token.as_ref());
        assert_eq!(insert_result.user_id, upsert_result.user_id);

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_concurrent_operations(pool: PgPool) -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let user = Users::mock_data()?;
        user.insert(&pool).await?;

        let token1 = mock_token();
        let token2 = mock_token();
        let duration = Duration::hours(24);
        let verification1 = EmailVerifications::new(&user, &token1, &duration);
        let verification2 = EmailVerifications::new(&user, &token2, &duration);

        //-- Execute Function (Act)
        let (result1, result2) =
            tokio::join!(verification1.insert(&pool), verification2.insert(&pool));

        //-- Checks (Assertions)
        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let db_record1 = result1.unwrap();
        let db_record2 = result2.unwrap();
        assert_ne!(db_record1.id, db_record2.id);
        assert_ne!(db_record1.token.as_ref(), db_record2.token.as_ref());

        Ok(())
    }

    #[test]
    fn test_new_email_verification_with_empty_token() -> Result<()> {
        let user = Users::mock_data()?;
        let empty_token = domain::EmailVerificationToken::from("".to_string());
        let duration = Duration::hours(24);

        // This should be caught at the domain level or during insert
        let verification = EmailVerifications::new(&user, &empty_token, &duration);
        assert_eq!(verification.token.as_ref(), "");
        Ok(())
    }

    #[test]
    fn test_new_email_verification_with_zero_duration() -> Result<()> {
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::zero();

        let verification = EmailVerifications::new(&user, &token, &duration);

        // Should create verification that expires immediately
        assert!(
            verification.expires_at <= chrono::Utc::now() + Duration::seconds(1)
        );
        Ok(())
    }

    #[test]
    fn test_new_email_verification_with_negative_duration() -> Result<()> {
        let user = Users::mock_data()?;
        let token = mock_token();
        let duration = Duration::hours(-1);

        let verification = EmailVerifications::new(&user, &token, &duration);

        // Should create already expired verification
        assert!(verification.expires_at < verification.created_at);
        Ok(())
    }
}
