//-- ./src/database/email_verification/delete.rs

//! Email Verification Deletion Functions
//!
//! This module provides asynchronous functions for deleting email verification records
//! from the database using `sqlx` and PostgreSQL. It supports the following deletion
//! operations:
//! 
//! - Deleting by unique ID, token, or user ID
//! - Batch deletion by multiple IDs
//! - Deleting all records, or all for a specific user
//! - Deleting expired or used tokens
//! - Deleting records older than a specified duration
//! - Instance-based deletion via the `EmailVerifications` struct
//! 
//! # Usage
//! These functions are intended for use in account management, security, and maintenance
//! workflows where email verification tokens must be invalidated or cleaned up.
//! 
//! # Safety & Compliance
//! All deletions are permanent and cannot be undone.
//! 
//! # Example
//! ```rust
//! // Delete all expired email verification tokens
//! let deleted = EmailVerifications::delete_expired(&pool).await?;
//! tracing::info!("Deleted {} expired verification tokens", deleted);
//! ```
//! 
//! # Related Modules
//! - [`crate::database::EmailVerifications`]: The main model for email verification records
//! - [`crate::domain::EmailVerificationToken`]: The token type used for verification
//! 
//! # See Also
//! - [`delete_by_id`], [`delete_by_token`], [`delete_all_user_id`], [`delete_expired`],

use uuid::Uuid;

use crate::{database::EmailVerifications, domain, AuthenticationError};

impl EmailVerifications {
    /// Deletes this email verification instance from the database.
    ///
    /// This method removes the specific email verification entry represented by the current instance (`self`).
    /// It is useful for object-oriented workflows where you have an `EmailVerifications` instance and want to
    /// delete its corresponding record from the database.
    ///
    /// # Arguments
    /// * `database` - PostgreSQL connection pool for database operations.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of records successfully deleted (should be 0 or 1).
    /// * `Err(AuthenticationError)` - Database connection error or query failure.
    ///
    /// # Behaviour
    /// - Deletes the record in the `email_verifications` table where `id` matches `self.id`.
    /// - Operation is permanent and cannot be undone.
    /// - Uses a single SQL `DELETE ... WHERE id = $1` statement.
    ///
    /// # Example
    /// ```rust
    /// let verification = EmailVerifications::find_by_id(&row_id, &pool).await?;
    /// let deleted = verification.delete(&pool).await?;
    /// tracing::info!("Deleted {} email verification record(s) with ID {}", deleted, verification.id);
    /// ```
    ///
    /// # Use Cases
    /// - **Object-Oriented Deletion**: Remove a record directly from its struct instance.
    /// - **Cleanup**: Delete a verification after it has been used or is no longer needed.
    /// - **Administrative Actions**: Allow admins to remove specific verification records.
    ///
    /// # Performance Notes
    /// - Efficient if `id` is the primary key or indexed.
    /// - Should only affect a single record per call.
    ///
    /// # Security & Compliance
    /// - Ensure only authorized actions can trigger this deletion.
    /// - Log the operation for audit and traceability.
    ///
    /// # Related Functions
    /// - [`delete_by_id()`]: Delete a token by its unique ID.
    /// - [`delete_all_user_id()`]: Delete all tokens for a specific user.
    /// - [`delete_expired()`]: Delete all expired tokens.
    #[tracing::instrument(
      name = "Delete Email Verification instance from the database: ",
      skip(database),
      fields(
          id = ?self.id
      )
    )]
    pub async fn delete(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
          DELETE FROM email_verifications
          WHERE id = $1
        "#,
            self.id.into_uuid()
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Email verification rows deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Deletes an email verification record by its token string.
    ///
    /// This function removes the email verification record that matches the provided token.
    /// It is useful when you have the token value (such as from a verification link) but not the record's ID,
    /// allowing you to invalidate or clean up a specific verification attempt.
    ///
    /// # Arguments
    /// * `token` - The `EmailVerificationToken` representing the token to delete.
    /// * `database` - PostgreSQL connection pool for database operations.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of records successfully deleted (should be 0 or 1).
    /// * `Err(AuthenticationError)` - Database connection error or query failure.
    ///
    /// # Behaviour
    /// - Deletes the record in the `email_verifications` table where `token` matches the provided value.
    /// - Operation is permanent and cannot be undone.
    /// - Uses a single SQL `DELETE ... WHERE token = $1` statement.
    ///
    /// # Example
    /// ```rust
    /// let deleted = EmailVerifications::delete_by_token(&token, &pool).await?;
    /// tracing::info!("Deleted {} email verification record(s) with token {}", deleted, token);
    /// ```
    ///
    /// # Use Cases
    /// - **Token Invalidation**: Remove a specific token after use or upon user request.
    /// - **Security**: Invalidate a token if it is suspected to be compromised.
    /// - **Cleanup**: Remove tokens that are no longer needed, using only the token value.
    ///
    /// # Performance Notes
    /// - Efficient if the `token` column is indexed.
    /// - Should only affect a single record per call if tokens are unique.
    ///
    /// # Security & Compliance
    /// - Ensure only authorized actions can trigger this deletion.
    /// - Log the operation for audit and traceability.
    ///
    /// # Related Functions
    /// - [`delete_by_id()`]: Delete a token by its unique ID.
    /// - [`delete_all_user_id()`]: Delete all tokens for a specific user.
    /// - [`delete_expired()`]: Delete all expired tokens.
    #[tracing::instrument(
        name = "Delete email verification by token",
        skip(database, token),
        fields(token_preview = %format!("{}...", &token.as_ref()[..12.min(token.as_ref().len())]))
    )]
    pub async fn delete_by_token(
        token: &domain::EmailVerificationToken,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                DELETE FROM email_verifications
                WHERE token = $1
            "#,
            token.as_ref()
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Email verification deleted by token: {rows_affected}");

        Ok(rows_affected)
    }

    /// Deletes a single email verification record by its unique ID.
    ///
    /// This function removes the email verification token that matches the provided `RowID`.
    /// It is useful for targeted deletion of a specific token, such as when invalidating
    /// a single verification attempt or cleaning up after a specific operation.
    ///
    /// # Arguments
    /// * `id` - The `RowID` of the email verification record to delete.
    /// * `database` - PostgreSQL connection pool for database operations.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of records successfully deleted (should be 0 or 1).
    /// * `Err(AuthenticationError)` - Database connection error or query failure.
    ///
    /// # Behaviour
    /// - Deletes the record in the `email_verifications` table where `id` matches the provided value.
    /// - Operation is permanent and cannot be undone.
    /// - Uses a single SQL `DELETE ... WHERE id = $1` statement.
    ///
    /// # Example
    /// ```rust
    /// let deleted = EmailVerifications::delete_by_id(&row_id, &pool).await?;
    /// tracing::info!("Deleted {} email verification record(s) with ID {}", deleted, row_id);
    /// ```
    ///
    /// # Use Cases
    /// - **Token Invalidation**: Remove a specific token after use or upon user request.
    /// - **Administrative Actions**: Allow admins to delete a problematic or compromised token.
    /// - **Cleanup**: Remove orphaned or obsolete tokens by ID.
    ///
    /// # Performance Notes
    /// - Efficient if `id` is the primary key or indexed.
    /// - Should only affect a single record per call.
    ///
    /// # Security & Compliance
    /// - Ensure only authorized actions can trigger this deletion.
    /// - Log the operation for audit and traceability.
    ///
    /// # Related Functions
    /// - [`delete_by_ids()`]: Delete multiple tokens by a list of IDs.
    /// - [`delete_all_user_id()`]: Delete all tokens for a specific user.
    /// - [`delete_expired()`]: Delete all expired tokens.
    #[tracing::instrument(
        name = "Delete Email Verification from the database using RowID: ",
        skip(database),
        fields(
            id = ?id
        )
      )]
    pub async fn delete_by_id(
        id: &domain::RowID,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                DELETE FROM email_verifications
                WHERE id = $1
            "#,
            id.into_uuid()
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Email verification rows deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Deletes all email verification records for a specific user.
    ///
    /// This function removes every email verification token associated with the given user ID.
    /// It is useful for cleaning up tokens when a user is deleted, when resetting a user's
    /// verification state, or as part of account management operations.
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user whose email verification records should be deleted.
    /// * `database` - PostgreSQL connection pool for database operations.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of records successfully deleted.
    /// * `Err(AuthenticationError)` - Database connection error or query failure.
    ///
    /// # Behaviour
    /// - Deletes all records in the `email_verifications` table where `user_id` matches the provided value.
    /// - Operation is permanent and cannot be undone.
    /// - Uses a single SQL `DELETE ... WHERE user_id = $1` statement for efficiency.
    ///
    /// # Example
    /// ```rust
    /// let deleted = EmailVerifications::delete_all_user_id(&user.id, &pool).await?;
    /// tracing::info!("Deleted {} email verification records for user {}", deleted, user.id);
    /// ```
    ///
    /// # Use Cases
    /// - **User Deletion**: Clean up all tokens when a user account is removed.
    /// - **Account Reset**: Remove all pending or used tokens for a user.
    /// - **Security**: Invalidate all tokens for a user after a security event.
    ///
    /// # Performance Notes
    /// - Efficient if `user_id` is indexed in the table.
    /// - For users with many tokens, consider monitoring query performance.
    ///
    /// # Security & Compliance
    /// - Ensure only authorized actions can trigger this deletion.
    /// - Log the operation for audit and traceability.
    ///
    /// # Related Functions
    /// - [`delete_by_ids()`]: Delete specific tokens by ID.
    /// - [`delete_expired()`]: Delete expired tokens for all users.
    /// - [`delete_all()`]: Delete all tokens for all users.
    #[tracing::instrument(
        name = "Delete Email Verification from the database using User ID: ",
        skip(database),
        fields(
            user_id = ?user_id
        )
      )]
    pub async fn delete_all_user_id(
        user_id: &Uuid,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                DELETE FROM email_verifications
                WHERE user_id = $1
            "#,
            user_id
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Email verification rows deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Deletes all email verification records from the database.
    ///
    /// This function removes every record from the `email_verifications` table, regardless of
    /// user, status, or expiration. It is intended for administrative or testing purposes where
    /// a complete cleanup of all email verification tokens is required.
    ///
    /// # Arguments
    /// * `database` - PostgreSQL connection pool for database operations.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of records successfully deleted.
    /// * `Err(AuthenticationError)` - Database connection error or query failure.
    ///
    /// # Behaviour
    /// - Deletes all records in the `email_verifications` table.
    /// - Operation is permanent and cannot be undone.
    /// - Uses a single SQL `DELETE FROM email_verifications` statement.
    ///
    /// # Example
    /// ```rust
    /// // Remove all email verification tokens (use with caution!)
    /// let deleted = EmailVerifications::delete_all(&pool).await?;
    /// tracing::info!("Deleted {} email verification records", deleted);
    /// ```
    ///
    /// # Use Cases
    /// - **Testing**: Clean up the table between test runs.
    /// - **Administrative Reset**: Wipe all tokens during a system reset or migration.
    /// - **Emergency Cleanup**: Remove all tokens in response to a security incident.
    ///
    /// # Warnings
    /// - **Irreversible**: All email verification data will be lost.
    /// - **Production Risk**: Use extreme caution in production environments.
    /// - **Audit**: Consider logging this operation for compliance and traceability.
    ///
    /// # Related Functions
    /// - [`delete_expired()`]: Delete only expired tokens.
    /// - [`delete_used()`]: Delete only used tokens.
    /// - [`delete_by_ids()`]: Delete specific tokens by ID.
    #[tracing::instrument(name = "Delete all Email Verification: ", skip(database))]
    pub async fn delete_all(
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                DELETE FROM email_verifications
            "#,
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Email verification rows deleted: {rows_affected:#?}");

        Ok(rows_affected)
    }

    /// Deletes multiple email verification records by their IDs.
    ///
    /// This function allows for batch deletion of email verification records, which is useful
    /// for efficiently removing a set of tokens (for example, as part of a bulk cleanup or
    /// administrative action). All records with IDs matching those in the provided slice will
    /// be deleted in a single database operation.
    ///
    /// # Arguments
    /// * `ids` - A slice of `RowID` values representing the email verification records to delete.
    /// * `database` - PostgreSQL connection pool for database operations.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of records successfully deleted.
    /// * `Err(AuthenticationError)` - Database connection error or query failure.
    ///
    /// # Behaviour
    /// - Deletes all records where `id` matches any value in the provided `ids` slice.
    /// - Operation is permanent and cannot be undone.
    /// - Uses a single SQL `DELETE ... WHERE id = ANY($1)` statement for efficiency.
    ///
    /// # Example
    /// ```rust
    /// let ids = vec![row_id1, row_id2, row_id3];
    /// let deleted = EmailVerifications::delete_by_ids(&ids, &pool).await?;
    /// tracing::info!("Deleted {} email verification records", deleted);
    /// ```
    ///
    /// # Use Cases
    /// - **Bulk Cleanup**: Remove multiple tokens at once (e.g., after a user purge).
    /// - **Administrative Actions**: Allow admins to delete selected tokens.
    /// - **Automated Maintenance**: Remove tokens matching certain criteria in batches.
    ///
    /// # Performance Notes
    /// - Efficient for moderate batch sizes; for very large batches, consider chunking.
    /// - Ensure `id` is indexed for optimal performance.
    ///
    /// # Security & Compliance
    /// - Ensure only authorized users can perform bulk deletions.
    /// - Log batch deletions for audit purposes.
    ///
    /// # Related Functions
    /// - [`delete_all_user_id()`]: Delete all tokens for a specific user.
    /// - [`delete_expired()`]: Delete all expired tokens.
    /// - [`delete_older_than()`]: Delete tokens based on creation date.

    #[tracing::instrument(
        name = "Delete email verifications by IDs",
        skip(database),
        fields(count = ids.len())
    )]
    pub async fn delete_by_ids(
        ids: &[domain::RowID],
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let uuids: Vec<Uuid> = ids.iter().map(|id| id.into_uuid()).collect();

        let rows_affected = sqlx::query!(
            r#"
                DELETE FROM email_verifications
                WHERE id = ANY($1)
            "#,
            &uuids
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Email verification records deleted: {rows_affected}");

        Ok(rows_affected)
    }

    /// Deletes all expired email verification records from the database.
    ///
    /// This function removes email verification tokens whose `expires_at` timestamp
    /// is earlier than the current time. It is intended for regular cleanup of tokens
    /// that are no longer valid for verification, helping to keep the database lean
    /// and performant.
    ///
    /// # Arguments
    /// * `database` - PostgreSQL connection pool for database operations.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of expired verification records successfully deleted
    /// * `Err(AuthenticationError)` - Database connection error or query failure
    ///
    /// # Behaviour
    /// - Deletes records where `expires_at < NOW()`
    /// - Does not consider `used` or `created_at` fields
    /// - Operation is permanent and cannot be undone
    ///
    /// # Use Cases
    /// - **Routine Maintenance**: Remove tokens that can no longer be used
    /// - **Storage Optimisation**: Free up space from obsolete records
    /// - **Security**: Reduce the risk of old tokens being leaked or misused
    ///
    /// # Example
    /// ```rust
    /// // Delete all expired email verification tokens
    /// let deleted = EmailVerifications::delete_expired(&pool).await?;
    /// tracing::info!("Deleted {} expired verification tokens", deleted);
    /// ```
    ///
    /// # Performance Notes
    /// - Efficient for tables with an index on `expires_at`
    /// - Consider running during off-peak hours for large datasets
    ///
    /// # Related Functions
    /// - [`delete_used()`]: Removes tokens that have been used, regardless of expiration
    /// - [`delete_older_than()`]: Removes tokens based on creation date
    ///
    /// # Compliance & Audit
    /// - Ensure deletion aligns with your data retention and audit policies
    /// - Consider logging the number of deleted records for audit trails

    #[tracing::instrument(
        name = "Delete expired email verifications",
        skip(database)
    )]
    pub async fn delete_expired(
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                DELETE FROM email_verifications
                WHERE expires_at < NOW()
            "#,
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!(
            "Expired email verification records deleted: {rows_affected}"
        );

        Ok(rows_affected)
    }

    /// Deletes all email verification records that have been marked as used.
    ///
    /// This function removes email verification tokens that have already been consumed
    /// for account verification. It's designed for cleanup operations to maintain database
    /// hygiene by removing tokens that are no longer needed for verification purposes.
    ///
    /// # Arguments
    /// * `database` - PostgreSQL connection pool for database operations.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of used verification records successfully deleted
    /// * `Err(AuthenticationError)` - Database connection error or query failure
    ///
    /// # Behaviour
    /// - Deletes records where `used = true`
    /// - Does not consider `expires_at`, `created_at`, or other fields
    /// - Operation is permanent and cannot be undone
    /// - Removes successful verification history from the database
    #[tracing::instrument(name = "Delete used email verifications", skip(database))]
    pub async fn delete_used(
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let rows_affected = sqlx::query!(
            r#"
                DELETE FROM email_verifications
                WHERE used = true
            "#,
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!("Used email verification records deleted: {rows_affected}");

        Ok(rows_affected)
    }

    /// Deletes email verification records older than the specified duration.
    ///
    /// This function is designed for data retention and cleanup operations, removing
    /// email verification records that exceed the given date. It deletes records based
    /// on their `created_at` timestamp, regardless of their expiration or usage status.
    ///
    /// # Arguments
    /// * `older_than` - The age threshold as a `chrono::Duration`. Records created
    ///   before `now - older_than` will be deleted.
    /// * `database` - PostgreSQL connection pool for database operations.
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of records successfully deleted
    /// * `Err(AuthenticationError)` - Database connection error or query failure
    ///
    /// # Behaviour
    /// - Deletes records where `created_at < (current_time - older_than)`
    /// - Does not consider `expires_at`, `used` status, or other fields
    /// - Operation is permanent and cannot be undone
    /// - Uses a single SQL DELETE statement for efficiency
    ///
    /// # Data Retention Examples
    /// ```rust
    /// use chrono::Duration;
    ///
    /// // Delete records older than 30 days (common retention policy)
    /// let deleted = EmailVerifications::delete_older_than(
    ///     Duration::days(30),
    ///     &pool
    /// ).await?;
    ///
    /// // Delete records older than 1 year (long-term cleanup)
    /// let deleted = EmailVerifications::delete_older_than(
    ///     Duration::days(365),
    ///     &pool
    /// ).await?;
    ///
    /// // Delete records older than 6 months
    /// let deleted = EmailVerifications::delete_older_than(
    ///     Duration::days(180),
    ///     &pool
    /// ).await?;
    /// ```
    #[tracing::instrument(
        name = "Delete old email verifications",
        skip(database),
        fields(older_than_days = older_than.num_days())
    )]
    pub async fn delete_older_than(
        older_than: chrono::Duration,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<u64, AuthenticationError> {
        let cutoff_time = chrono::Utc::now() - older_than;

        let rows_affected = sqlx::query!(
            r#"
                DELETE FROM email_verifications
                WHERE created_at < $1
            "#,
            cutoff_time
        )
        .execute(database)
        .await?
        .rows_affected();

        tracing::debug!(
            "Email verification records older than {} days deleted: {rows_affected}",
            older_than.num_days()
        );

        Ok(rows_affected)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::database::Users;
//     use chrono::Duration;
//     use sqlx::PgPool;
//     use uuid::Uuid;

//     // Helper functions for test setup
//     async fn setup_test_user(pool: &PgPool) -> Users {
//         let user = Users::mock_data().unwrap();
//         user.insert(pool).await.unwrap();
//         user
//     }

//     async fn setup_test_verification(
//         pool: &PgPool,
//         user: &Users,
//         duration: Duration,
//     ) -> EmailVerifications {
//         let token = mock_token();
//         let verification = EmailVerifications::new(user, &token, &duration);
//         verification.insert(pool).await.unwrap();
//         verification
//     }

//     async fn setup_expired_verification(pool: &PgPool, user: &Users) -> EmailVerifications {
//         let mut verification = setup_test_verification(pool, user, Duration::hours(-1)).await;
//         verification.expires_at = chrono::Utc::now() - Duration::hours(1);
//         verification.update(pool).await.unwrap();
//         verification
//     }

//     async fn setup_used_verification(pool: &PgPool, user: &Users) -> EmailVerifications {
//         let mut verification = setup_test_verification(pool, user, Duration::hours(24)).await;
//         verification.used = true;
//         verification.update(pool).await.unwrap();
//         verification
//     }

//     fn mock_token() -> domain::EmailVerificationToken {
//         use fake::{faker::company::en::CompanyName, Fake, Faker};
//         use secrecy::SecretString;

//         let issuer = SecretString::new(CompanyName().fake::<String>().into_boxed_str());
//         let secret = SecretString::new(Faker.fake::<String>().into_boxed_str());
//         let user = Users::mock_data().unwrap();
//         let duration = Duration::hours(24);
//         let token_type = &domain::TokenType::EmailVerification;

//         let claim = domain::TokenClaimNew::new(&issuer, &duration, &user, token_type);
//         domain::EmailVerificationToken::try_from_claim(claim, &secret)
//             .expect("Failed to generate mock token")
//     }

//     #[sqlx::test]
//     async fn test_delete_by_id_success(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;

//         let rows_affected = EmailVerifications::delete_by_id(&verification.id, &pool)
//             .await
//             .unwrap();

//         assert_eq!(rows_affected, 1);

//         // Verify deletion
//         let result = EmailVerifications::find_by_id(&verification.id, &pool).await;
//         assert!(result.is_err());
//     }

//     #[sqlx::test]
//     async fn test_delete_by_id_nonexistent(pool: PgPool) {
//         let fake_id = domain::RowID::new();
//         let rows_affected = EmailVerifications::delete_by_id(&fake_id, &pool)
//             .await
//             .unwrap();

//         assert_eq!(rows_affected, 0);
//     }

//     #[sqlx::test]
//     async fn test_delete_by_token_success(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;

//         let rows_affected = EmailVerifications::delete_by_token(&verification.token, &pool)
//             .await
//             .unwrap();

//         assert_eq!(rows_affected, 1);

//         // Verify deletion
//         let result = EmailVerifications::find_by_id(&verification.id, &pool).await;
//         assert!(result.is_err());
//     }

//     #[sqlx::test]
//     async fn test_delete_by_token_nonexistent(pool: PgPool) {
//         let fake_token = mock_token();
//         let rows_affected = EmailVerifications::delete_by_token(&fake_token, &pool)
//             .await
//             .unwrap();

//         assert_eq!(rows_affected, 0);
//     }

//     #[sqlx::test]
//     async fn test_delete_all_user_id_success(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let verification1 = setup_test_verification(&pool, &user, Duration::hours(24)).await;
//         let verification2 = setup_test_verification(&pool, &user, Duration::hours(48)).await;

//         let rows_affected = EmailVerifications::delete_all_user_id(&user.id, &pool)
//             .await
//             .unwrap();

//         assert_eq!(rows_affected, 2);

//         // Verify both deletions
//         let result1 = EmailVerifications::find_by_id(&verification1.id, &pool).await;
//         let result2 = EmailVerifications::find_by_id(&verification2.id, &pool).await;
//         assert!(result1.is_err());
//         assert!(result2.is_err());
//     }

//     #[sqlx::test]
//     async fn test_delete_all_user_id_preserves_other_users(pool: PgPool) {
//         let user1 = setup_test_user(&pool).await;
//         let user2 = setup_test_user(&pool).await;
//         let verification1 = setup_test_verification(&pool, &user1, Duration::hours(24)).await;
//         let verification2 = setup_test_verification(&pool, &user2, Duration::hours(24)).await;

//         let rows_affected = EmailVerifications::delete_all_user_id(&user1.id, &pool)
//             .await
//             .unwrap();

//         assert_eq!(rows_affected, 1);

//         // Verify user1's verification is deleted, user2's is preserved
//         let result1 = EmailVerifications::find_by_id(&verification1.id, &pool).await;
//         let result2 = EmailVerifications::find_by_id(&verification2.id, &pool).await;
//         assert!(result1.is_err());
//         assert!(result2.is_ok());
//     }

//     #[sqlx::test]
//     async fn test_delete_all_user_id_nonexistent_user(pool: PgPool) {
//         let fake_user_id = Uuid::new_v4();
//         let rows_affected = EmailVerifications::delete_all_user_id(&fake_user_id, &pool)
//             .await
//             .unwrap();

//         assert_eq!(rows_affected, 0);
//     }

//     #[sqlx::test]
//     async fn test_delete_expired_success(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let _active_verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;
//         let _expired_verification = setup_expired_verification(&pool, &user).await;

//         let rows_affected = EmailVerifications::delete_expired(&pool).await.unwrap();

//         assert!(rows_affected >= 1); // At least the expired one we created
//     }

//     #[sqlx::test]
//     async fn test_delete_expired_preserves_active(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let active_verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;
//         let _expired_verification = setup_expired_verification(&pool, &user).await;

//         EmailVerifications::delete_expired(&pool).await.unwrap();

//         // Verify active verification is preserved
//         let result = EmailVerifications::find_by_id(&active_verification.id, &pool).await;
//         assert!(result.is_ok());
//     }

//     #[sqlx::test]
//     async fn test_delete_used_success(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let _unused_verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;
//         let _used_verification = setup_used_verification(&pool, &user).await;

//         let rows_affected = EmailVerifications::delete_used(&pool).await.unwrap();

//         assert!(rows_affected >= 1); // At least the used one we created
//     }

//     #[sqlx::test]
//     async fn test_delete_used_preserves_unused(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let unused_verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;
//         let _used_verification = setup_used_verification(&pool, &user).await;

//         EmailVerifications::delete_used(&pool).await.unwrap();

//         // Verify unused verification is preserved
//         let result = EmailVerifications::find_by_id(&unused_verification.id, &pool).await;
//         assert!(result.is_ok());
//     }

//     #[sqlx::test]
//     async fn test_delete_older_than_success(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let mut old_verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;
//         let _recent_verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;

//         // Make one verification old
//         old_verification.created_at = chrono::Utc::now() - Duration::days(2);
//         old_verification.update(&pool).await.unwrap();

//         let rows_affected = EmailVerifications::delete_older_than(Duration::days(1), &pool)
//             .await
//             .unwrap();

//         assert!(rows_affected >= 1); // At least the old one we created
//     }

//     #[sqlx::test]
//     async fn test_delete_older_than_preserves_recent(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let mut old_verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;
//         let recent_verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;

//         // Make one verification old
//         old_verification.created_at = chrono::Utc::now() - Duration::days(2);
//         old_verification.update(&pool).await.unwrap();

//         EmailVerifications::delete_older_than(Duration::days(1), &pool)
//             .await
//             .unwrap();

//         // Verify recent verification is preserved
//         let result = EmailVerifications::find_by_id(&recent_verification.id, &pool).await;
//         assert!(result.is_ok());
//     }

//     #[sqlx::test]
//     async fn test_delete_by_ids_success(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let verification1 = setup_test_verification(&pool, &user, Duration::hours(24)).await;
//         let verification2 = setup_test_verification(&pool, &user, Duration::hours(24)).await;
//         let verification3 = setup_test_verification(&pool, &user, Duration::hours(24)).await;

//         let ids = vec![verification1.id, verification2.id];
//         let rows_affected = EmailVerifications::delete_by_ids(&ids, &pool)
//             .await
//             .unwrap();

//         assert_eq!(rows_affected, 2);

//         // Verify specific deletions
//         let result1 = EmailVerifications::find_by_id(&verification1.id, &pool).await;
//         let result2 = EmailVerifications::find_by_id(&verification2.id, &pool).await;
//         let result3 = EmailVerifications::find_by_id(&verification3.id, &pool).await;
//         assert!(result1.is_err());
//         assert!(result2.is_err());
//         assert!(result3.is_ok()); // Should be preserved
//     }

//     #[sqlx::test]
//     async fn test_delete_by_ids_empty_list(pool: PgPool) {
//         let ids: Vec<domain::RowID> = vec![];
//         let rows_affected = EmailVerifications::delete_by_ids(&ids, &pool)
//             .await
//             .unwrap();

//         assert_eq!(rows_affected, 0);
//     }

//     #[sqlx::test]
//     async fn test_delete_by_ids_nonexistent_ids(pool: PgPool) {
//         let fake_ids = vec![domain::RowID::new(), domain::RowID::new()];
//         let rows_affected = EmailVerifications::delete_by_ids(&fake_ids, &pool)
//             .await
//             .unwrap();

//         assert_eq!(rows_affected, 0);
//     }

//     #[sqlx::test]
//     async fn test_delete_all_success(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let _verification1 = setup_test_verification(&pool, &user, Duration::hours(24)).await;
//         let _verification2 = setup_test_verification(&pool, &user, Duration::hours(24)).await;

//         let rows_affected = EmailVerifications::delete_all(&pool).await.unwrap();

//         assert!(rows_affected >= 2); // At least the two we created
//     }

//     #[sqlx::test]
//     async fn test_delete_instance_success(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;

//         let rows_affected = verification.delete(&pool).await.unwrap();

//         assert_eq!(rows_affected, 1);

//         // Verify deletion
//         let result = EmailVerifications::find_by_id(&verification.id, &pool).await;
//         assert!(result.is_err());
//     }

//     #[sqlx::test]
//     async fn test_delete_instance_already_deleted(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;

//         // Delete once
//         verification.delete(&pool).await.unwrap();

//         // Try to delete again
//         let rows_affected = verification.delete(&pool).await.unwrap();
//         assert_eq!(rows_affected, 0);
//     }

//     #[sqlx::test]
//     async fn test_multiple_deletion_methods_consistency(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let verification1 = setup_test_verification(&pool, &user, Duration::hours(24)).await;
//         let verification2 = setup_test_verification(&pool, &user, Duration::hours(24)).await;

//         // Delete using different methods
//         let rows1 = EmailVerifications::delete_by_id(&verification1.id, &pool)
//             .await
//             .unwrap();
//         let rows2 = verification2.delete(&pool).await.unwrap();

//         assert_eq!(rows1, 1);
//         assert_eq!(rows2, 1);

//         // Verify both are deleted
//         let result1 = EmailVerifications::find_by_id(&verification1.id, &pool).await;
//         let result2 = EmailVerifications::find_by_id(&verification2.id, &pool).await;
//         assert!(result1.is_err());
//         assert!(result2.is_err());
//     }

//     #[sqlx::test]
//     async fn test_delete_operations_are_isolated(pool: PgPool) {
//         let user1 = setup_test_user(&pool).await;
//         let user2 = setup_test_user(&pool).await;
//         let verification1 = setup_test_verification(&pool, &user1, Duration::hours(24)).await;
//         let verification2 = setup_test_verification(&pool, &user2, Duration::hours(24)).await;

//         // Delete user1's verification
//         EmailVerifications::delete_all_user_id(&user1.id, &pool)
//             .await
//             .unwrap();

//         // Verify user2's verification is unaffected
//         let result = EmailVerifications::find_by_id(&verification2.id, &pool).await;
//         assert!(result.is_ok());
//     }

//     #[sqlx::test]
//     async fn test_delete_with_concurrent_operations(pool: PgPool) {
//         let user = setup_test_user(&pool).await;
//         let verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;

//         // Simulate concurrent deletion attempts
//         let delete_future1 = EmailVerifications::delete_by_id(&verification.id, &pool);
//         let delete_future2 = verification.delete(&pool);

//         let (result1, result2) = tokio::join!(delete_future1, delete_future2);

//         // One should succeed with 1 row, the other with 0 rows
//         let rows1 = result1.unwrap();
//         let rows2 = result2.unwrap();
//         assert_eq!(rows1 + rows2, 1);
//     }

//     #[sqlx::test]
//     async fn test_delete_error_handling_with_invalid_database(pool: PgPool) {
//         // This test would require a way to simulate database errors
//         // For now, we'll test with valid operations
//         let user = setup_test_user(&pool).await;
//         let verification = setup_test_verification(&pool, &user, Duration::hours(24)).await;

//         // Normal operation should succeed
//         let result = verification.delete(&pool).await;
//         assert!(result.is_ok());
//     }
// }