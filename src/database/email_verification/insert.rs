//-- ./src/database/email_verification/insert.rs

#![allow(unused)] // For development only

use crate::{database::EmailVerifications, AuthenticationError};

impl EmailVerifications {
    /// Inserts this email verification record into the database.
    ///
    /// This function creates a new row in the `email_verifications` table using the values
    /// from this `EmailVerifications` instance. It returns the inserted record as stored in
    /// the database, including any fields that may be set or modified by the database itself.
    ///
    /// # Arguments
    /// * `database` - A reference to the PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(Self)` - The inserted email verification record as returned by the database.
    /// * `Err(AuthenticationError)` - If the insertion fails due to a database error,
    ///   constraint violation, or connection issue.
    ///
    /// # Behaviour
    /// - Inserts all fields from the struct into the corresponding database columns.
    /// - Fails if a record with the same `id` or `token` already exists.
    /// - Fails if the referenced `user_id` does not exist.
    /// - Returns the full inserted record, including any database-generated values.
    ///
    /// # Example
    /// ```
    /// let verification = EmailVerifications::new(&user, &token, &duration);
    /// let inserted = verification.insert(&pool).await?;
    /// assert_eq!(inserted.user_id, user.id);
    /// ```
    ///
    /// # Errors
    /// Returns an error if:
    /// - The database connection fails.
    /// - A unique or foreign key constraint is violated.
    /// - Any required field is missing or invalid.
    ///
    /// # Security
    /// This function does not validate the token; it assumes the token is already valid.
    ///
    /// # See Also
    /// - [`EmailVerifications::insert_batch`] for inserting multiple records in a transaction.
    /// - [`EmailVerifications::upsert`] for insert-or-update
    pub async fn insert(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, AuthenticationError> {
        let database_record = sqlx::query_as!(
            EmailVerifications,
            r#"
                INSERT INTO email_verifications (id, user_id, token, expires_at, is_used, created_at)
                VALUES ($1, $2, $3, $4, $5, $6)
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
        .await?;

        tracing::debug!(
            "Email verification database insert record: {database_record:#?}"
        );

        Ok(database_record)
    }

    /// Inserts multiple email verification records in a single database transaction.
    ///
    /// This function attempts to insert all provided `EmailVerifications` records into the
    /// `email_verifications` table atomically. If any insertion fails (for example, due to
    /// a constraint violation), the entire batch is rolled back and no records are inserted.
    ///
    /// # Arguments
    /// * `verifications` - A slice of `EmailVerifications` instances to insert.
    /// * `database` - A reference to the PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(Vec<EmailVerifications>)` - A vector of the inserted records as returned by the database.
    /// * `Err(AuthenticationError)` - If any insertion fails or the transaction cannot be completed.
    ///
    /// # Behaviour
    /// - All insertions are performed within a single transaction.
    /// - If any record fails to insert (e.g., duplicate ID or token, invalid user), the transaction is rolled back.
    /// - Returns all successfully inserted records if the batch succeeds.
    ///
    /// # Example
    /// ```
    /// let verifications = vec![
    ///     EmailVerifications::new(&user, &token1, &duration),
    ///     EmailVerifications::new(&user, &token2, &duration),
    /// ];
    /// let inserted = EmailVerifications::insert_batch(&verifications, &pool).await?;
    /// assert_eq!(inserted.len(), 2);
    /// ```
    ///
    /// # Errors
    /// Returns an error if:
    /// - Any record violates a unique or foreign key constraint.
    /// - The database connection fails.
    /// - The transaction cannot be committed.
    ///
    /// # Use Cases
    /// - Efficiently inserting multiple verification tokens for testing or bulk operations.
    /// - Ensuring all-or-nothing semantics for related verification records.
    ///
    /// # See Also
    /// - [`EmailVerifications::insert`] for inserting a single record.
    /// - [`EmailVerifications::upsert`] for insert-or-
    pub async fn insert_batch(
        verifications: &[EmailVerifications],
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<EmailVerifications>, AuthenticationError> {
        let mut tx = database.begin().await?;
        let mut inserted = Vec::with_capacity(verifications.len());

        for verification in verifications {
            let db_record = sqlx::query_as!(
                EmailVerifications,
                r#"
                    INSERT INTO email_verifications (id, user_id, token, expires_at, is_used, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6)
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

    /// Inserts or updates an email verification record in the database ("upsert").
    ///
    /// This function attempts to insert the current `EmailVerifications` instance into the
    /// `email_verifications` table. If a record with the same primary key (`id`) already exists,
    /// it updates the existing record with the new values provided. The resulting record as stored
    /// in the database is returned.
    ///
    /// # Arguments
    /// * `database` - A reference to the PostgreSQL connection pool.
    ///
    /// # Returns
    /// * `Ok(Self)` - The inserted or updated email verification record as returned by the database.
    /// * `Err(AuthenticationError)` - If the operation fails due to a database error or constraint violation.
    ///
    /// # Behaviour
    /// - If no record with the same `id` exists, a new record is inserted.
    /// - If a record with the same `id` exists, its fields are updated with the new values.
    /// - Unique and foreign key constraints are enforced.
    /// - Returns the full record as stored in the database after the operation.
    ///
    /// # Example
    /// ```
    /// let verification = EmailVerifications::new(&user, &token, &duration);
    /// let upserted = verification.upsert(&pool).await?;
    /// assert_eq!(upserted.id, verification.id);
    /// ```
    ///
    /// # Errors
    /// Returns an error if:
    /// - The database connection fails.
    /// - A unique or foreign key constraint is violated.
    /// - Any required field is missing or invalid.
    ///
    /// # Use Cases
    /// - Idempotent creation or update of verification tokens.
    /// - Ensuring a user has at most one active verification record per ID.
    /// - Simplifying logic where insert-or-update is needed in one call.
    ///
    /// # See Also
    /// - [`EmailVerifications::insert`] for inserting a single record.
    /// - [`EmailVerifications::insert_batch`] for inserting multiple records in
    pub async fn upsert(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, AuthenticationError> {
        let database_record = sqlx::query_as!(
        EmailVerifications,
        r#"
            INSERT INTO email_verifications (id, user_id, token, expires_at, is_used, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET user_id = EXCLUDED.user_id,
                token = EXCLUDED.token,
                expires_at = EXCLUDED.expires_at,
                is_used = EXCLUDED.is_used,
                created_at = EXCLUDED.created_at
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
    .await?;

        Ok(database_record)
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
}
