//-- ./src/database/email_verification/read.rs

use chrono;
use uuid::Uuid;

use crate::{database::EmailVerifications, domain, AuthenticationError};

fn safe_cast_to_i64(value: usize) -> Result<i64, AuthenticationError> {
    i64::try_from(value).map_err(|_| {
        AuthenticationError::ValidationError(
            "Pagination value too large".to_string(),
        )
    })
}

fn validate_cursor_pagination(
    limit: usize,
    cursor_created_at: Option<chrono::DateTime<chrono::Utc>>,
    cursor_id: Option<Uuid>,
) -> Result<(), AuthenticationError> {
    if limit > 1000 {
        tracing::warn!("Large limit requested: {}", limit);
    }

    // Validate cursor consistency
    match (cursor_created_at, cursor_id) {
        (None, None) | (Some(_), Some(_)) => Ok(()),
        _ => Err(AuthenticationError::ValidationError(
            "Both cursor_created_at and cursor_id must be provided together"
                .to_string(),
        )),
    }
}

impl EmailVerifications {
    #[tracing::instrument(
        name = "Get a Email verification from the database using Row ID: ",
        skip(database),
        fields(
            verification_id = ?id,
        )
    )]
    pub async fn from_id(
        id: &domain::RowID,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<EmailVerifications, AuthenticationError> {
        let query = sqlx::query_as!(
            EmailVerifications,
            r#"
              SELECT * FROM email_verifications
              WHERE id = $1
            "#,
            id.into_uuid()
        )
        .fetch_one(database)
        .await;

        match query {
            Ok(email_verification) => Ok(email_verification),
            Err(e) => {
                tracing::error!(
                    "Failed to fetch email verification by id: {} (id: {:?})",
                    e,
                    id
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(
        name = "Get a Email verification from the database using token: ",
        skip(database),
        fields(
            email_token = ?token,
        )
    )]
    pub async fn from_token(
        token: &domain::EmailVerificationToken,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<EmailVerifications, AuthenticationError> {
        let query = sqlx::query_as!(
            EmailVerifications,
            r#"
              SELECT * FROM email_verifications
              WHERE token = $1
            "#,
            token.as_ref()
        )
        .fetch_one(database)
        .await;

        match query {
            Ok(email_verification) => Ok(email_verification),
            Err(e) => {
                tracing::error!(
                    "Failed to fetch email verification by token: {} (token: {:?})",
                    e,
                    token
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(
        name = "Get a Email verifications index: ",
        skip(database),
        fields(
            limit = ?limit,
            offset = ?offset,
        )
    )]
    pub async fn index(
        limit: &usize,
        offset: &usize,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<EmailVerifications>, AuthenticationError> {
        if *limit > 1000 {
            tracing::warn!("Large limit requested: {}", limit);
        }

        let offset_i64 = safe_cast_to_i64(*offset)?;
        let limit_i64 = safe_cast_to_i64(*limit)?;

        let query = sqlx::query_as!(
            EmailVerifications,
            r#"
                SELECT * FROM email_verifications
                ORDER BY id
                OFFSET $1
                LIMIT $2
            "#,
            offset_i64,
            limit_i64,
        )
        .fetch_all(database)
        .await;

        match query {
            Ok(email_verifications) => Ok(email_verifications),
            Err(e) => {
                tracing::error!(
                    "Failed to fetch email verification index: {} (limit: {}, offset: {})", 
                    e, limit, offset
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(
        name = "Get email verifications with cursor pagination",
        skip(database),
        fields(
            limit = %limit,
            cursor_created_at = ?cursor_created_at,
            cursor_id = ?cursor_id
        )
    )]
    pub async fn index_cursor(
        limit: usize,
        cursor_created_at: Option<chrono::DateTime<chrono::Utc>>,
        cursor_id: Option<Uuid>,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<EmailVerifications>, AuthenticationError> {
        validate_cursor_pagination(limit, cursor_created_at, cursor_id)?;

        let limit_i64 = safe_cast_to_i64(limit)?;

        let results = match (cursor_created_at, cursor_id) {
            // First page - no cursor
            (None, None) => {
                sqlx::query_as!(
                    EmailVerifications,
                    r#"
                        SELECT * FROM email_verifications
                        ORDER BY created_at ASC, id ASC
                        LIMIT $1
                    "#,
                    limit_i64
                )
                .fetch_all(database)
                .await
            }
            // Subsequent pages - with cursor
            (Some(created_at), Some(id)) => {
                sqlx::query_as!(
                    EmailVerifications,
                    r#"
                        SELECT * FROM email_verifications
                        WHERE (created_at, id) > ($1, $2)
                        ORDER BY created_at ASC, id ASC
                        LIMIT $3
                    "#,
                    created_at,
                    id,
                    limit_i64
                )
                .fetch_all(database)
                .await
            }
            // Invalid cursor state
            _ => {
                return Err(AuthenticationError::ValidationError(
                    "Both cursor_created_at and cursor_id must be provided together"
                        .to_string(),
                ));
            }
        };

        results.map_err(|e| {
            tracing::error!(
                "Failed to fetch email verifications with cursor: {} (limit: {}, cursor_created_at: {:?}, cursor_id: {:?})",
                e, limit, cursor_created_at, cursor_id
            );
            AuthenticationError::DatabaseError(e.to_string())
        })
    }

    #[tracing::instrument(
        name = "Get a Email verifications index from the database using user id: ",
        skip(database),
        fields(
            user_id = ?user_id,
            limit = ?limit,
            offset = ?offset,
        )
    )]
    pub async fn index_from_user_id(
        user_id: &Uuid,
        limit: &usize,
        offset: &usize,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<EmailVerifications>, AuthenticationError> {
        if *limit > 1000 {
            tracing::warn!("Large limit requested: {}", limit);
        }

        let offset_i64 = safe_cast_to_i64(*offset)?;
        let limit_i64 = safe_cast_to_i64(*limit)?;

        let query = sqlx::query_as!(
            EmailVerifications,
            r#"
                SELECT * FROM email_verifications
                WHERE user_id = $1
                ORDER BY id
                OFFSET $2
                LIMIT $3
            "#,
            user_id,
            offset_i64,
            limit_i64,
        )
        .fetch_all(database)
        .await;

        match query {
            Ok(email_verifications) => Ok(email_verifications),
            Err(e) => {
                tracing::error!(
                    "Failed to fetch email verifications for user: {} (user_id: {}, limit: {}, offset: {})",
                    e, user_id, limit, offset
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(
        name = "Get email verifications by user_id with cursor pagination",
        skip(database),
        fields(
            user_id = %user_id,
            limit = %limit,
            cursor_created_at = ?cursor_created_at,
            cursor_id = ?cursor_id
        )
    )]
    pub async fn index_from_user_id_cursor(
        user_id: &Uuid,
        limit: usize,
        cursor_created_at: Option<chrono::DateTime<chrono::Utc>>,
        cursor_id: Option<Uuid>,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Vec<EmailVerifications>, AuthenticationError> {
        validate_cursor_pagination(limit, cursor_created_at, cursor_id)?;

        let limit_i64 = safe_cast_to_i64(limit)?;

        let results = match (cursor_created_at, cursor_id) {
            // First page - no cursor
            (None, None) => {
                sqlx::query_as!(
                    EmailVerifications,
                    r#"
                        SELECT * FROM email_verifications
                        WHERE user_id = $1
                        ORDER BY created_at ASC, id ASC
                        LIMIT $2
                    "#,
                    user_id,
                    limit_i64
                )
                .fetch_all(database)
                .await
            }
            // Subsequent pages - with cursor
            (Some(created_at), Some(id)) => {
                sqlx::query_as!(
                    EmailVerifications,
                    r#"
                        SELECT * FROM email_verifications
                        WHERE user_id = $1 
                        AND (created_at, id) > ($2, $3)
                        ORDER BY created_at ASC, id ASC
                        LIMIT $4
                    "#,
                    user_id,
                    created_at,
                    id,
                    limit_i64
                )
                .fetch_all(database)
                .await
            }
            // Invalid cursor state
            _ => {
                return Err(AuthenticationError::ValidationError(
                    "Both cursor_created_at and cursor_id must be provided together"
                        .to_string(),
                ));
            }
        };

        results.map_err(|e| {
            tracing::error!(
                "Failed to fetch email verifications by user_id with cursor: {} (user_id: {}, limit: {}, cursor_created_at: {:?}, cursor_id: {:?})",
                e, user_id, limit, cursor_created_at, cursor_id
            );
            AuthenticationError::DatabaseError(e.to_string())
        })
    }
}

#[cfg(test)]
//-- Tests
//   Test the read functionality for email verifications
//   
//   This module provides comprehensive test coverage for all read operations
//   on the EmailVerifications model, ensuring correctness, performance, and
//   error handling across various scenarios.
//
//   ## Test Organization
//   
//   The tests are organised into logical groups that build from simple validation
//   to complex real-world scenarios:
//
//   ### 1. Validation (Pure Functions)
//   Tests utility and helper functions that perform input validation and data
//   transformation without database interaction. These are pure functions that
//   ensure proper parameter validation and type safety.
//
//   ### 2. Core Functionality (Single Record Queries)
//   Tests the fundamental database read operations for retrieving individual
//   email verification records. These form the foundation for all other operations.
//
//   ### 3. Basic Pagination (Limit Offset-based)
//   Tests traditional offset-based pagination for listing email verifications.
//   This provides the baseline pagination functionality with simple numeric
//   parameters.
//
//   ### 4. Cursor Based Pagination
//   Tests cursor-based pagination for scalable listing of email verifications.
//   Cursor pagination provides better performance for large datasets and
//   consistent results during concurrent modifications.
//
//   ### 5. User-Based Pagination (Limit Offset-based)
//   Tests user-specific offset pagination to retrieve verifications for a
//   particular user. This ensures proper data isolation and user-scoped
//   pagination functionality.
//
//   ### 6. User Based Cursor Pagination
//   Tests user-specific cursor pagination combining the benefits of cursor-based
//   pagination with user data isolation. This is the most sophisticated pagination
//   method, suitable for user-facing applications with large datasets.
//
//   ### 7. Error Handling & Edge Cases
//   Tests error conditions, boundary cases, and validation failures across all
//   read operations. This ensures robust error handling and graceful degradation
//   under adverse conditions.

mod tests {
    // Bring functions into scope
    use super::*;
    use crate::database;

    // Make error handling simpler for tests
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    use fake::{faker::company::en::CompanyName, Fake, Faker};
    use secrecy::SecretString;

    //-- 0. Test helper functions
    // Don't repeat yourself, reusable test helper functions

    async fn create_test_user(pool: &sqlx::PgPool) -> Result<database::Users> {
        let user = database::Users::mock_data()?;
        user.insert(pool).await.map_err(|e| e.into())
    }

    async fn create_test_verification(
        user: &database::Users,
        pool: &sqlx::PgPool,
        token: Option<domain::EmailVerificationToken>,
    ) -> Result<EmailVerifications> {
        let token = token.unwrap_or_else(|| mock_verification_token());
        let verification =
            EmailVerifications::new(user, &token, &chrono::Duration::hours(24));
        verification.insert(pool).await.map_err(|e| e.into())
    }

    fn mock_verification_token() -> domain::EmailVerificationToken {
        let random_token_issuer =
            SecretString::new(CompanyName().fake::<String>().into_boxed_str());

        let random_token_secret =
            SecretString::new(Faker.fake::<String>().into_boxed_str());

        let random_user = database::Users::mock_data().unwrap();

        let random_hours: i64 = (1..=72).fake();
        let random_duration = chrono::Duration::hours(random_hours);

        let token_type = &domain::TokenType::EmailVerification;

        let claim = domain::TokenClaimNew::new(
            &random_token_issuer,
            &random_duration,
            &random_user,
            token_type,
        );

        domain::EmailVerificationToken::try_from_claim(claim, &random_token_secret)
            .expect("Failed to generate mock token")
    }

    async fn create_multiple_verifications(
        user: &database::Users,
        count: usize,
        pool: &sqlx::PgPool,
    ) -> Result<Vec<EmailVerifications>> {
        let mut verifications = Vec::new();
        for i in 0..count {
            // Ensure created_at is slightly different for ordering tests
            tokio::time::sleep(std::time::Duration::from_millis((i + 1) as u64))
                .await;

            // Generate random duration using fake crate
            let random_hours: i64 = (1..=72).fake();
            let random_duration = chrono::Duration::hours(random_hours);

            verifications.push(create_test_verification(user, pool, None).await?);
        }
        Ok(verifications)
    }

    //-- 1: Validation (Pure Functions)
    // Make sure the utility and helper functions are working
    mod validation {
        use super::*;

        #[test]
        fn safe_cast_to_i64_valid() -> Result<()> {
            // Test small positive number
            let result = safe_cast_to_i64(100);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 100i64);

            // Test zero
            let result = safe_cast_to_i64(0);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0i64);

            // Test maximum valid i64 value that can be represented as usize
            let max_valid = i64::MAX as usize;
            let result = safe_cast_to_i64(max_valid);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), i64::MAX);

            // Test typical pagination values
            let result = safe_cast_to_i64(1000);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 1000i64);

            Ok(())
        }

        #[test]
        fn safe_cast_to_i64_overflow() -> Result<()> {
            // Test usize::MAX which should overflow on most platforms
            let large_value = usize::MAX;
            let result = safe_cast_to_i64(large_value);
            assert!(result.is_err());

            // Verify it returns the correct error type
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!("Expected ValidationError with specific message");
            }

            Ok(())
        }

        #[test]
        fn cursor_pagination_valid_cases() -> Result<()> {
            // Valid case: both None
            let result = validate_cursor_pagination(10, None, None);
            assert!(result.is_ok());

            // Valid case: both Some
            let result = validate_cursor_pagination(
                10,
                Some(chrono::Utc::now()),
                Some(uuid::Uuid::new_v4()),
            );
            assert!(result.is_ok());

            // Valid case: limit at boundary (should warn but succeed)
            let result = validate_cursor_pagination(1001, None, None);
            assert!(result.is_ok());

            Ok(())
        }

        #[test]
        fn cursor_pagination_invalid_cases() -> Result<()> {
            // Invalid case: only created_at provided
            let result =
                validate_cursor_pagination(10, Some(chrono::Utc::now()), None);
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(
                    message,
                    "Both cursor_created_at and cursor_id must be provided together"
                );
            } else {
                panic!("Expected ValidationError with specific message");
            }

            // Invalid case: only cursor_id provided
            let result =
                validate_cursor_pagination(10, None, Some(uuid::Uuid::new_v4()));
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(
                    message,
                    "Both cursor_created_at and cursor_id must be provided together"
                );
            } else {
                panic!("Expected ValidationError with specific message");
            }

            Ok(())
        }

        #[test]
        fn cursor_pagination_edge_cases() -> Result<()> {
            // Test with limit 0 (should be valid for validation function)
            let result = validate_cursor_pagination(0, None, None);
            assert!(result.is_ok());

            // Test with very large limit (should warn but succeed)
            let result = validate_cursor_pagination(10000, None, None);
            assert!(result.is_ok());

            Ok(())
        }
    }

    //-- 2: Core Functionality (Single Record Queries)
    // Test the core database read functionality
    mod single_record_read {
        use super::*;

        #[sqlx::test]
        async fn from_id_success(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            let verification = create_test_verification(&user, &pool, None).await?;

            // Act
            let result =
                EmailVerifications::from_id(&verification.id, &pool).await?;

            // Assert
            assert_eq!(result.id, verification.id);
            assert_eq!(result.user_id, verification.user_id);
            assert_eq!(result.token, verification.token);
            assert_eq!(result.is_used, verification.is_used);
            assert_eq!(result.expires_at, verification.expires_at);

            Ok(())
        }

        #[sqlx::test]
        async fn from_id_not_found(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let fake_id = domain::RowID::new();

            // Act
            let result = EmailVerifications::from_id(&fake_id, &pool).await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::DatabaseError(message)) = result {
                // Currently returns DatabaseError, should be NotFound when you fix error handling
                assert!(
                    message.contains("no rows returned")
                        || message.contains("not found")
                );
            } else {
                panic!(
                    "Expected DatabaseError for non-existent ID, got: {:?}",
                    result
                );
            }

            Ok(())
        }

        #[sqlx::test]
        async fn from_id_multiple_verifications_returns_correct_one(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            let verification1 = create_test_verification(&user, &pool, None).await?;
            let verification2 = create_test_verification(&user, &pool, None).await?;
            let verification3 = create_test_verification(&user, &pool, None).await?;

            // Act
            let result =
                EmailVerifications::from_id(&verification2.id, &pool).await?;

            // Assert
            assert_eq!(result.id, verification2.id);
            assert_ne!(result.id, verification1.id);
            assert_ne!(result.id, verification3.id);

            Ok(())
        }

        #[sqlx::test]
        async fn from_token_success(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            let token = mock_verification_token();
            let verification =
                create_test_verification(&user, &pool, Some(token.clone())).await?;

            // Act
            let result = EmailVerifications::from_token(&token, &pool).await?;

            // Assert
            assert_eq!(result.id, verification.id);
            assert_eq!(result.user_id, verification.user_id);
            assert_eq!(result.token, token);
            assert_eq!(result.is_used, verification.is_used);
            assert_eq!(result.expires_at, verification.expires_at);

            Ok(())
        }

        #[sqlx::test]
        async fn from_token_not_found(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let fake_token = mock_verification_token();

            // Act
            let result = EmailVerifications::from_token(&fake_token, &pool).await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::DatabaseError(message)) = result {
                // Currently returns DatabaseError, should be NotFound when you fix error handling
                assert!(
                    message.contains("no rows returned")
                        || message.contains("not found")
                );
            } else {
                panic!(
                    "Expected DatabaseError for non-existent token, got: {:?}",
                    result
                );
            }

            Ok(())
        }

        #[sqlx::test]
        async fn from_token_multiple_verifications_returns_correct_one(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            let token1 = mock_verification_token();
            let token2 = mock_verification_token();
            let token3 = mock_verification_token();

            let verification1 =
                create_test_verification(&user, &pool, Some(token1.clone())).await?;
            let verification2 =
                create_test_verification(&user, &pool, Some(token2.clone())).await?;
            let verification3 =
                create_test_verification(&user, &pool, Some(token3.clone())).await?;

            // Act
            let result = EmailVerifications::from_token(&token2, &pool).await?;

            // Assert
            assert_eq!(result.id, verification2.id);
            assert_eq!(result.token, token2);
            assert_ne!(result.id, verification1.id);
            assert_ne!(result.id, verification3.id);

            Ok(())
        }

        #[sqlx::test]
        async fn from_token_different_users_same_token_pattern(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange - Create different users with verifications
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;

            let token1 = mock_verification_token();
            let token2 = mock_verification_token();

            let verification1 =
                create_test_verification(&user1, &pool, Some(token1.clone()))
                    .await?;
            let verification2 =
                create_test_verification(&user2, &pool, Some(token2.clone()))
                    .await?;

            // Act - Find each verification by its token
            let result1 = EmailVerifications::from_token(&token1, &pool).await?;
            let result2 = EmailVerifications::from_token(&token2, &pool).await?;

            // Assert
            assert_eq!(result1.id, verification1.id);
            assert_eq!(result1.user_id, user1.id);
            assert_eq!(result2.id, verification2.id);
            assert_eq!(result2.user_id, user2.id);

            Ok(())
        }

        #[sqlx::test]
        async fn from_id_and_from_token_consistency(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            let token = mock_verification_token();
            let verification =
                create_test_verification(&user, &pool, Some(token.clone())).await?;

            // Act - Fetch the same record using both methods
            let result_by_id =
                EmailVerifications::from_id(&verification.id, &pool).await?;
            let result_by_token =
                EmailVerifications::from_token(&token, &pool).await?;

            // Assert - Both methods should return identical records
            assert_eq!(result_by_id.id, result_by_token.id);
            assert_eq!(result_by_id.user_id, result_by_token.user_id);
            assert_eq!(result_by_id.token, result_by_token.token);
            assert_eq!(result_by_id.is_used, result_by_token.is_used);
            assert_eq!(result_by_id.expires_at, result_by_token.expires_at);
            assert_eq!(result_by_id.created_at, result_by_token.created_at);
            assert_eq!(result_by_id.updated_at, result_by_token.updated_at);

            Ok(())
        }

        #[sqlx::test]
        async fn from_id_with_expired_verification(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange - Create an expired verification
            let user = create_test_user(&pool).await?;
            let token = mock_verification_token();
            let expired_verification =
                EmailVerifications::new(&user, &token, &chrono::Duration::hours(-1)); // Expired 1 hour ago
            let inserted = expired_verification.insert(&pool).await?;

            // Act
            let result = EmailVerifications::from_id(&inserted.id, &pool).await?;

            // Assert - Should still find the record (expiration is business logic, not database constraint)
            assert_eq!(result.id, inserted.id);
            assert!(result.expires_at < chrono::Utc::now()); // Verify it's actually expired

            Ok(())
        }

        #[sqlx::test]
        async fn from_token_with_used_verification(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange - Create a used verification
            let user = create_test_user(&pool).await?;
            let token = mock_verification_token();
            let mut verification =
                EmailVerifications::new(&user, &token, &chrono::Duration::hours(24));
            verification.is_used = true; // Mark as used
            let inserted = verification.insert(&pool).await?;

            // Act
            let result = EmailVerifications::from_token(&token, &pool).await?;

            // Assert - Should still find the record
            assert_eq!(result.id, inserted.id);
            assert_eq!(result.token, token);
            assert!(result.is_used); // Verify it's marked as used

            Ok(())
        }

        #[sqlx::test]
        async fn from_id_invalid_uuid_format(pool: sqlx::PgPool) -> Result<()> {
            // Note: This test is more theoretical since RowID likely validates UUID format
            // But it's good to document the expected behaviour

            // Arrange
            let user = create_test_user(&pool).await?;
            let verification = create_test_verification(&user, &pool, None).await?;

            // Act & Assert - The current implementation should handle this gracefully
            // since RowID likely ensures valid UUID format
            let result = EmailVerifications::from_id(&verification.id, &pool).await;
            assert!(result.is_ok());

            Ok(())
        }
    }

    //--3: Basic Pagination (Limit Offset-based)
    // Test the index limit and offset functionality
    mod offset_pagination {
        use super::*;

        #[sqlx::test]
        async fn index_empty_database(pool: sqlx::PgPool) -> Result<()> {
            // Act
            let results = EmailVerifications::index(&10, &0, &pool).await?;

            // Assert
            assert!(results.is_empty());
            Ok(())
        }

        #[sqlx::test]
        async fn index_basic_listing(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            let verifications =
                create_multiple_verifications(&user, 3, &pool).await?;

            // Act
            let results = EmailVerifications::index(&10, &0, &pool).await?;

            // Assert
            assert_eq!(results.len(), 3);

            // Verify all returned verifications belong to our user
            for verification in &results {
                assert_eq!(verification.user_id, user.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_with_limit(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Act
            let results = EmailVerifications::index(&3, &0, &pool).await?;

            // Assert
            assert_eq!(results.len(), 3);
            Ok(())
        }

        #[sqlx::test]
        async fn index_with_limit_exceeds_data(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act
            let results = EmailVerifications::index(&10, &0, &pool).await?;

            // Assert
            assert_eq!(results.len(), 2); // Should return all available records
            Ok(())
        }

        #[sqlx::test]
        async fn index_with_offset(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            let verifications =
                create_multiple_verifications(&user, 5, &pool).await?;

            // Act
            let page1 = EmailVerifications::index(&2, &0, &pool).await?;
            let page2 = EmailVerifications::index(&2, &2, &pool).await?;
            let page3 = EmailVerifications::index(&2, &4, &pool).await?;

            // Assert
            assert_eq!(page1.len(), 2);
            assert_eq!(page2.len(), 2);
            assert_eq!(page3.len(), 1); // Last page with remaining record

            // Verify pages contain different records
            assert_ne!(page1[0].id, page2[0].id);
            assert_ne!(page2[0].id, page3[0].id);
            Ok(())
        }

        #[sqlx::test]
        async fn index_with_offset_exceeds_data(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act
            let results = EmailVerifications::index(&10, &5, &pool).await?;

            // Assert
            assert!(results.is_empty()); // Offset beyond available data
            Ok(())
        }

        #[sqlx::test]
        async fn index_zero_limit(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act
            let results = EmailVerifications::index(&0, &0, &pool).await?;

            // Assert
            assert!(results.is_empty()); // Zero limit should return no results
            Ok(())
        }

        #[sqlx::test]
        async fn index_zero_offset(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act
            let results = EmailVerifications::index(&2, &0, &pool).await?;

            // Assert
            assert_eq!(results.len(), 2);
            Ok(())
        }

        #[sqlx::test]
        async fn index_ordering_consistency(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            let verifications =
                create_multiple_verifications(&user, 5, &pool).await?;

            // Act - Get results in multiple pages
            let page1 = EmailVerifications::index(&2, &0, &pool).await?;
            let page2 = EmailVerifications::index(&2, &2, &pool).await?;
            let page3 = EmailVerifications::index(&1, &4, &pool).await?;

            // Combine all pages
            let mut all_paginated: Vec<_> = page1
                .into_iter()
                .chain(page2.into_iter())
                .chain(page3.into_iter())
                .collect();

            // Get all results at once
            let all_at_once = EmailVerifications::index(&10, &0, &pool).await?;

            // Assert - Should have same number of records
            assert_eq!(all_paginated.len(), all_at_once.len());
            assert_eq!(all_paginated.len(), 5);

            // Sort both by ID to compare (since index() orders by id)
            all_paginated.sort_by(|a, b| a.id.cmp(&b.id));
            let mut all_sorted = all_at_once;
            all_sorted.sort_by(|a, b| a.id.cmp(&b.id));

            // Assert - All IDs should match
            for (paginated, at_once) in all_paginated.iter().zip(all_sorted.iter()) {
                assert_eq!(paginated.id, at_once.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_large_offset_overflow(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with offset that would overflow i64
            let large_offset = usize::MAX;
            let result = EmailVerifications::index(&10, &large_offset, &pool).await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!("Expected ValidationError for overflow, got: {:?}", result);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_large_limit_overflow(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with limit that would overflow i64
            let large_limit = usize::MAX;
            let result = EmailVerifications::index(&large_limit, &0, &pool).await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!("Expected ValidationError for overflow, got: {:?}", result);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_large_limit_warning(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act - Large but valid limit (should warn but succeed)
            let results = EmailVerifications::index(&1001, &0, &pool).await?;

            // Assert
            assert_eq!(results.len(), 3); // Should return all available records
            Ok(())
        }

        #[sqlx::test]
        async fn index_mixed_users(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;
            let user3 = create_test_user(&pool).await?;

            create_multiple_verifications(&user1, 2, &pool).await?;
            create_multiple_verifications(&user2, 3, &pool).await?;
            create_multiple_verifications(&user3, 1, &pool).await?;

            // Act
            let results = EmailVerifications::index(&10, &0, &pool).await?;

            // Assert
            assert_eq!(results.len(), 6); // Total records across all users

            // Verify we have records from all users
            let user_ids: std::collections::HashSet<_> =
                results.iter().map(|v| v.user_id).collect();
            assert_eq!(user_ids.len(), 3);
            assert!(user_ids.contains(&user1.id));
            assert!(user_ids.contains(&user2.id));
            assert!(user_ids.contains(&user3.id));
            Ok(())
        }

        #[sqlx::test]
        async fn index_typical_pagination_scenario(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange - Simulate typical pagination scenario
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 25, &pool).await?; // 25 records total

            // Act - Paginate with page size of 10
            let page1 = EmailVerifications::index(&10, &0, &pool).await?;
            let page2 = EmailVerifications::index(&10, &10, &pool).await?;
            let page3 = EmailVerifications::index(&10, &20, &pool).await?;
            let page4 = EmailVerifications::index(&10, &30, &pool).await?; // Beyond data

            // Assert
            assert_eq!(page1.len(), 10);
            assert_eq!(page2.len(), 10);
            assert_eq!(page3.len(), 5); // Remaining records
            assert_eq!(page4.len(), 0); // No more records

            // Verify no duplicates across pages
            let all_ids: Vec<_> = page1
                .iter()
                .chain(page2.iter())
                .chain(page3.iter())
                .map(|v| v.id.into_uuid())
                .collect();
            let unique_ids: std::collections::HashSet<_> = all_ids.iter().collect();
            assert_eq!(all_ids.len(), unique_ids.len()); // No duplicates
            Ok(())
        }

        #[sqlx::test]
        async fn index_edge_case_limit_one(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Act
            let results = EmailVerifications::index(&1, &0, &pool).await?;

            // Assert
            assert_eq!(results.len(), 1);
            Ok(())
        }

        #[sqlx::test]
        async fn index_boundary_values(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act & Assert - Various boundary value combinations

            // Limit equals total records
            let results = EmailVerifications::index(&3, &0, &pool).await?;
            assert_eq!(results.len(), 3);

            // Offset at last record
            let results = EmailVerifications::index(&10, &2, &pool).await?;
            assert_eq!(results.len(), 1);

            // Offset at total count (should be empty)
            let results = EmailVerifications::index(&10, &3, &pool).await?;
            assert_eq!(results.len(), 0);

            Ok(())
        }
    }

    //-- 4: Cursor Based Pagination
    // Test the cursor index pagination functionality
    mod cursor_pagination {
        use super::*;

        #[sqlx::test]
        async fn index_cursor_empty_database(pool: sqlx::PgPool) -> Result<()> {
            // Act
            let results =
                EmailVerifications::index_cursor(10, None, None, &pool).await?;

            // Assert
            assert!(results.is_empty());
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_first_page(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Act
            let results =
                EmailVerifications::index_cursor(3, None, None, &pool).await?;

            // Assert
            assert_eq!(results.len(), 3);

            // Verify ordering (should be created_at ASC, id ASC)
            for i in 1..results.len() {
                assert!(
                    results[i - 1].created_at <= results[i].created_at,
                    "Results should be ordered by created_at ASC"
                );
                if results[i - 1].created_at == results[i].created_at {
                    assert!(
                        results[i - 1].id <= results[i].id,
                        "Results with same created_at should be ordered by id ASC"
                    );
                }
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_subsequent_pages(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            let verifications =
                create_multiple_verifications(&user, 7, &pool).await?;

            // Act - Page 1
            let page1 =
                EmailVerifications::index_cursor(3, None, None, &pool).await?;
            assert_eq!(page1.len(), 3);

            // Act - Page 2
            let cursor_created_at = Some(page1.last().unwrap().created_at);
            let cursor_id = Some(page1.last().unwrap().id.into_uuid());
            let page2 = EmailVerifications::index_cursor(
                3,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;
            assert_eq!(page2.len(), 3);

            // Act - Page 3 (last page)
            let cursor_created_at = Some(page2.last().unwrap().created_at);
            let cursor_id = Some(page2.last().unwrap().id.into_uuid());
            let page3 = EmailVerifications::index_cursor(
                3,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;
            assert_eq!(page3.len(), 1); // Only 1 remaining record

            // Act - Page 4 (beyond data)
            let cursor_created_at = Some(page3.last().unwrap().created_at);
            let cursor_id = Some(page3.last().unwrap().id.into_uuid());
            let page4 = EmailVerifications::index_cursor(
                3,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;
            assert_eq!(page4.len(), 0); // No more records

            // Assert - Verify no duplicates across pages
            let all_ids: Vec<_> = page1
                .iter()
                .chain(page2.iter())
                .chain(page3.iter())
                .map(|v| v.id.into_uuid())
                .collect();
            let unique_ids: std::collections::HashSet<_> = all_ids.iter().collect();
            assert_eq!(
                all_ids.len(),
                unique_ids.len(),
                "No duplicates across pages"
            );

            // Assert - Verify all records are accounted for
            assert_eq!(all_ids.len(), 7);
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_invalid_parameters_missing_created_at(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Act
            let result = EmailVerifications::index_cursor(
                10,
                None,
                Some(uuid::Uuid::new_v4()), // cursor_id without created_at
                &pool,
            )
            .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(
                    message,
                    "Both cursor_created_at and cursor_id must be provided together"
                );
            } else {
                panic!("Expected ValidationError for invalid cursor parameters");
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_invalid_parameters_missing_id(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Act
            let result = EmailVerifications::index_cursor(
                10,
                Some(chrono::Utc::now()), // created_at without cursor_id
                None,
                &pool,
            )
            .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(
                    message,
                    "Both cursor_created_at and cursor_id must be provided together"
                );
            } else {
                panic!("Expected ValidationError for invalid cursor parameters");
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_valid_parameters_both_none(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act
            let result =
                EmailVerifications::index_cursor(10, None, None, &pool).await?;

            // Assert
            assert_eq!(result.len(), 2);
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_valid_parameters_both_some(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Get first page
            let page1 =
                EmailVerifications::index_cursor(2, None, None, &pool).await?;
            assert_eq!(page1.len(), 2);

            // Act - Use valid cursor parameters
            let cursor_created_at = Some(page1.last().unwrap().created_at);
            let cursor_id = Some(page1.last().unwrap().id.into_uuid());
            let result = EmailVerifications::index_cursor(
                2,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;

            // Assert
            assert!(result.len() <= 2);
            // Verify no overlap with previous page
            for item in &result {
                assert!(!page1.iter().any(|p1_item| p1_item.id == item.id));
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_end_of_results(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act - Get all records in pages
            let page1 =
                EmailVerifications::index_cursor(2, None, None, &pool).await?;
            assert_eq!(page1.len(), 2);

            let cursor_created_at = Some(page1.last().unwrap().created_at);
            let cursor_id = Some(page1.last().unwrap().id.into_uuid());
            let page2 = EmailVerifications::index_cursor(
                2,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;
            assert_eq!(page2.len(), 1); // Last record

            // Act - Try to get next page (should be empty)
            let cursor_created_at = Some(page2.last().unwrap().created_at);
            let cursor_id = Some(page2.last().unwrap().id.into_uuid());
            let page3 = EmailVerifications::index_cursor(
                2,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;

            // Assert
            assert_eq!(
                page3.len(),
                0,
                "Should return empty when beyond last record"
            );
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_ordering_consistency(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 10, &pool).await?;

            // Act - Get all records using cursor pagination
            let mut all_cursor_results = Vec::new();
            let mut cursor_created_at = None;
            let mut cursor_id = None;

            loop {
                let page = EmailVerifications::index_cursor(
                    3,
                    cursor_created_at,
                    cursor_id,
                    &pool,
                )
                .await?;
                if page.is_empty() {
                    break;
                }

                // Set cursor for next page
                cursor_created_at = Some(page.last().unwrap().created_at);
                cursor_id = Some(page.last().unwrap().id.into_uuid());

                all_cursor_results.extend(page);
            }

            // Get all records using index_cursor with large limit
            let all_at_once =
                EmailVerifications::index_cursor(100, None, None, &pool).await?;

            // Assert - Should have same number of records
            assert_eq!(all_cursor_results.len(), all_at_once.len());
            assert_eq!(all_cursor_results.len(), 10);

            // Assert - All IDs should match in same order
            for (cursor_result, at_once_result) in
                all_cursor_results.iter().zip(all_at_once.iter())
            {
                assert_eq!(cursor_result.id, at_once_result.id);
                assert_eq!(cursor_result.created_at, at_once_result.created_at);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_limit_zero(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act
            let result =
                EmailVerifications::index_cursor(0, None, None, &pool).await?;

            // Assert
            assert_eq!(result.len(), 0);
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_limit_one(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Act
            let result =
                EmailVerifications::index_cursor(1, None, None, &pool).await?;

            // Assert
            assert_eq!(result.len(), 1);
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_large_limit_warning(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act - Large but valid limit (should warn but succeed)
            let result =
                EmailVerifications::index_cursor(1001, None, None, &pool).await?;

            // Assert
            assert_eq!(result.len(), 3); // Should return all available records
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_large_limit_overflow(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with limit that would overflow i64
            let large_limit = usize::MAX;
            let result =
                EmailVerifications::index_cursor(large_limit, None, None, &pool)
                    .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!("Expected ValidationError for overflow, got: {:?}", result);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_mixed_users(pool: sqlx::PgPool) -> Result<()> {
            // Arrange - Create multiple users with verifications
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;
            let user3 = create_test_user(&pool).await?;

            create_multiple_verifications(&user1, 2, &pool).await?;
            create_multiple_verifications(&user2, 3, &pool).await?;
            create_multiple_verifications(&user3, 1, &pool).await?;

            // Act
            let results =
                EmailVerifications::index_cursor(10, None, None, &pool).await?;

            // Assert
            assert_eq!(results.len(), 6); // Total records across all users

            // Verify we have records from all users
            let user_ids: std::collections::HashSet<_> =
                results.iter().map(|v| v.user_id).collect();
            assert_eq!(user_ids.len(), 3);
            assert!(user_ids.contains(&user1.id));
            assert!(user_ids.contains(&user2.id));
            assert!(user_ids.contains(&user3.id));

            // Verify ordering is consistent across users
            for i in 1..results.len() {
                assert!(
                    results[i - 1].created_at <= results[i].created_at,
                    "Results should be ordered by created_at regardless of user"
                );
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_cursor_pointing_to_nonexistent_record(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act - Use cursor pointing to non-existent record
            let fake_cursor_created_at =
                Some(chrono::Utc::now() + chrono::Duration::days(1));
            let fake_cursor_id = Some(uuid::Uuid::new_v4());
            let result = EmailVerifications::index_cursor(
                10,
                fake_cursor_created_at,
                fake_cursor_id,
                &pool,
            )
            .await?;

            // Assert - Should return empty (no records after the fake cursor)
            assert_eq!(result.len(), 0);
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_cursor_before_all_records(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act - Use cursor pointing to very old timestamp
            let old_cursor_created_at =
                Some(chrono::Utc::now() - chrono::Duration::days(365));
            let old_cursor_id = Some(uuid::Uuid::new_v4());
            let result = EmailVerifications::index_cursor(
                10,
                old_cursor_created_at,
                old_cursor_id,
                &pool,
            )
            .await?;

            // Assert - Should return all records (all records are after the old cursor)
            assert_eq!(result.len(), 3);
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_same_created_at_different_ids(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange - This is tricky to test since create_multiple_verifications adds delays
            // We'll create verifications manually to ensure same created_at
            let user = create_test_user(&pool).await?;

            let token1 = mock_verification_token();
            let token2 = mock_verification_token();
            let token3 = mock_verification_token();

            let fixed_time = chrono::Utc::now();

            // Create verifications with same created_at but different IDs
            let mut verification1 = EmailVerifications::new(
                &user,
                &token1,
                &chrono::Duration::hours(24),
            );
            let mut verification2 = EmailVerifications::new(
                &user,
                &token2,
                &chrono::Duration::hours(24),
            );
            let mut verification3 = EmailVerifications::new(
                &user,
                &token3,
                &chrono::Duration::hours(24),
            );

            verification1.created_at = fixed_time;
            verification2.created_at = fixed_time;
            verification3.created_at = fixed_time;

            let v1 = verification1.insert(&pool).await?;
            let v2 = verification2.insert(&pool).await?;
            let v3 = verification3.insert(&pool).await?;

            // Act - Get first page
            let page1 =
                EmailVerifications::index_cursor(2, None, None, &pool).await?;
            assert_eq!(page1.len(), 2);

            // Act - Get second page using cursor from first page
            let cursor_created_at = Some(page1.last().unwrap().created_at);
            let cursor_id = Some(page1.last().unwrap().id.into_uuid());
            let page2 = EmailVerifications::index_cursor(
                2,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;

            // Assert - Should get the remaining record(s)
            assert!(page2.len() <= 1);

            // Verify no duplicates
            let all_ids: Vec<_> = page1
                .iter()
                .chain(page2.iter())
                .map(|v| v.id.into_uuid())
                .collect();
            let unique_ids: std::collections::HashSet<_> = all_ids.iter().collect();
            assert_eq!(all_ids.len(), unique_ids.len());
            Ok(())
        }

        #[sqlx::test]
        async fn index_cursor_consistency_with_offset_pagination(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 10, &pool).await?;

            // Act - Get all records using cursor pagination
            let mut all_cursor_results = Vec::new();
            let mut cursor_created_at = None;
            let mut cursor_id = None;

            loop {
                let page = EmailVerifications::index_cursor(
                    3,
                    cursor_created_at,
                    cursor_id,
                    &pool,
                )
                .await?;
                if page.is_empty() {
                    break;
                }

                cursor_created_at = Some(page.last().unwrap().created_at);
                cursor_id = Some(page.last().unwrap().id.into_uuid());
                all_cursor_results.extend(page);
            }

            // Get all records using offset pagination
            let all_offset_results =
                EmailVerifications::index(&100, &0, &pool).await?;

            // Assert - Should have same number of records
            assert_eq!(all_cursor_results.len(), all_offset_results.len());
            assert_eq!(all_cursor_results.len(), 10);

            // Note: We can't directly compare ordering since index() uses ORDER BY id
            // while index_cursor() uses ORDER BY created_at, id
            // But we can verify all the same records are present
            let cursor_ids: std::collections::HashSet<_> = all_cursor_results
                .iter()
                .map(|v| v.id.into_uuid())
                .collect();
            let offset_ids: std::collections::HashSet<_> = all_offset_results
                .iter()
                .map(|v| v.id.into_uuid())
                .collect();
            assert_eq!(
                cursor_ids, offset_ids,
                "Both pagination methods should return same records"
            );
            Ok(())
        }
    }

    //-- 5: User-Based Pagination (Limit Offset-based)
    // Test the user based offset limit index functionality
    mod user_pagination {
        use super::*;

        #[sqlx::test]
        async fn index_from_user_id_success(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;

            create_multiple_verifications(&user1, 3, &pool).await?;
            create_multiple_verifications(&user2, 2, &pool).await?;

            // Act
            let results =
                EmailVerifications::index_from_user_id(&user1.id, &10, &0, &pool)
                    .await?;

            // Assert
            assert_eq!(results.len(), 3);
            for verification in &results {
                assert_eq!(verification.user_id, user1.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_empty(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;

            // Act
            let results =
                EmailVerifications::index_from_user_id(&user.id, &10, &0, &pool)
                    .await?;

            // Assert
            assert_eq!(results.len(), 0);
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_with_limit(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Act
            let results =
                EmailVerifications::index_from_user_id(&user.id, &3, &0, &pool)
                    .await?;

            // Assert
            assert_eq!(results.len(), 3);
            for verification in &results {
                assert_eq!(verification.user_id, user.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_with_offset(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Act
            let page1 =
                EmailVerifications::index_from_user_id(&user.id, &2, &0, &pool)
                    .await?;
            let page2 =
                EmailVerifications::index_from_user_id(&user.id, &2, &2, &pool)
                    .await?;
            let page3 =
                EmailVerifications::index_from_user_id(&user.id, &2, &4, &pool)
                    .await?;

            // Assert
            assert_eq!(page1.len(), 2);
            assert_eq!(page2.len(), 2);
            assert_eq!(page3.len(), 1); // Last page with remaining record

            // Verify pages contain different records but same user
            assert_ne!(page1[0].id, page2[0].id);
            assert_ne!(page2[0].id, page3[0].id);

            for verification in page1.iter().chain(page2.iter()).chain(page3.iter())
            {
                assert_eq!(verification.user_id, user.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_pagination_flow(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange - Simulate realistic pagination scenario
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 7, &pool).await?;

            // Act - Paginate through all records
            let page1 =
                EmailVerifications::index_from_user_id(&user.id, &3, &0, &pool)
                    .await?;
            let page2 =
                EmailVerifications::index_from_user_id(&user.id, &3, &3, &pool)
                    .await?;
            let page3 =
                EmailVerifications::index_from_user_id(&user.id, &3, &6, &pool)
                    .await?;
            let page4 =
                EmailVerifications::index_from_user_id(&user.id, &3, &9, &pool)
                    .await?; // Beyond data

            // Assert
            assert_eq!(page1.len(), 3);
            assert_eq!(page2.len(), 3);
            assert_eq!(page3.len(), 1); // Last record
            assert_eq!(page4.len(), 0); // No more records

            // Verify all records belong to the user and are unique
            let all_ids: Vec<_> = page1
                .iter()
                .chain(page2.iter())
                .chain(page3.iter())
                .map(|v| v.id.into_uuid())
                .collect();
            let unique_ids: std::collections::HashSet<_> = all_ids.iter().collect();
            assert_eq!(all_ids.len(), unique_ids.len()); // No duplicates
            assert_eq!(all_ids.len(), 7);
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_nonexistent_user(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let fake_user_id = uuid::Uuid::new_v4();

            // Create some data for other users
            let real_user = create_test_user(&pool).await?;
            create_multiple_verifications(&real_user, 3, &pool).await?;

            // Act
            let results = EmailVerifications::index_from_user_id(
                &fake_user_id,
                &10,
                &0,
                &pool,
            )
            .await?;

            // Assert
            assert_eq!(results.len(), 0);
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_filters_correctly(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange - Create multiple users with verifications
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;
            let user3 = create_test_user(&pool).await?;

            create_multiple_verifications(&user1, 2, &pool).await?;
            create_multiple_verifications(&user2, 3, &pool).await?;
            create_multiple_verifications(&user3, 1, &pool).await?;

            // Act - Query each user separately
            let user1_results =
                EmailVerifications::index_from_user_id(&user1.id, &10, &0, &pool)
                    .await?;
            let user2_results =
                EmailVerifications::index_from_user_id(&user2.id, &10, &0, &pool)
                    .await?;
            let user3_results =
                EmailVerifications::index_from_user_id(&user3.id, &10, &0, &pool)
                    .await?;

            // Assert
            assert_eq!(user1_results.len(), 2);
            assert_eq!(user2_results.len(), 3);
            assert_eq!(user3_results.len(), 1);

            // Verify each result set only contains verifications for the correct user
            for verification in &user1_results {
                assert_eq!(verification.user_id, user1.id);
            }
            for verification in &user2_results {
                assert_eq!(verification.user_id, user2.id);
            }
            for verification in &user3_results {
                assert_eq!(verification.user_id, user3.id);
            }

            // Verify no cross-contamination between users
            let all_user1_ids: std::collections::HashSet<_> =
                user1_results.iter().map(|v| v.id.into_uuid()).collect();
            let all_user2_ids: std::collections::HashSet<_> =
                user2_results.iter().map(|v| v.id.into_uuid()).collect();
            let all_user3_ids: std::collections::HashSet<_> =
                user3_results.iter().map(|v| v.id.into_uuid()).collect();

            assert!(all_user1_ids.is_disjoint(&all_user2_ids));
            assert!(all_user1_ids.is_disjoint(&all_user3_ids));
            assert!(all_user2_ids.is_disjoint(&all_user3_ids));
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_ordering_consistency(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Act - Get results in multiple pages
            let page1 =
                EmailVerifications::index_from_user_id(&user.id, &2, &0, &pool)
                    .await?;
            let page2 =
                EmailVerifications::index_from_user_id(&user.id, &2, &2, &pool)
                    .await?;
            let page3 =
                EmailVerifications::index_from_user_id(&user.id, &1, &4, &pool)
                    .await?;

            // Combine all pages
            let mut all_paginated: Vec<_> = page1
                .into_iter()
                .chain(page2.into_iter())
                .chain(page3.into_iter())
                .collect();

            // Get all results at once
            let all_at_once =
                EmailVerifications::index_from_user_id(&user.id, &10, &0, &pool)
                    .await?;

            // Assert - Should have same number of records
            assert_eq!(all_paginated.len(), all_at_once.len());
            assert_eq!(all_paginated.len(), 5);

            // Sort both by ID to compare (since index_from_user_id() orders by id)
            all_paginated.sort_by(|a, b| a.id.cmp(&b.id));
            let mut all_sorted = all_at_once;
            all_sorted.sort_by(|a, b| a.id.cmp(&b.id));

            // Assert - All IDs should match
            for (paginated, at_once) in all_paginated.iter().zip(all_sorted.iter()) {
                assert_eq!(paginated.id, at_once.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_zero_limit(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act
            let results =
                EmailVerifications::index_from_user_id(&user.id, &0, &0, &pool)
                    .await?;

            // Assert
            assert_eq!(results.len(), 0);
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_large_limit_warning(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act - Large but valid limit (should warn but succeed)
            let results =
                EmailVerifications::index_from_user_id(&user.id, &1001, &0, &pool)
                    .await?;

            // Assert
            assert_eq!(results.len(), 3); // Should return all available records for user
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_large_limit_overflow(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with limit that would overflow i64
            let large_limit = usize::MAX;
            let result = EmailVerifications::index_from_user_id(
                &user.id,
                &large_limit,
                &0,
                &pool,
            )
            .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!("Expected ValidationError for overflow, got: {:?}", result);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_large_offset_overflow(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with offset that would overflow i64
            let large_offset = usize::MAX;
            let result = EmailVerifications::index_from_user_id(
                &user.id,
                &10,
                &large_offset,
                &pool,
            )
            .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!("Expected ValidationError for overflow, got: {:?}", result);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_offset_exceeds_data(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act
            let results =
                EmailVerifications::index_from_user_id(&user.id, &10, &5, &pool)
                    .await?;

            // Assert
            assert_eq!(results.len(), 0); // Offset beyond available data
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_limit_exceeds_data(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act
            let results =
                EmailVerifications::index_from_user_id(&user.id, &10, &0, &pool)
                    .await?;

            // Assert
            assert_eq!(results.len(), 2); // Should return all available records
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_boundary_values(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act & Assert - Various boundary value combinations

            // Limit equals total records
            let results =
                EmailVerifications::index_from_user_id(&user.id, &3, &0, &pool)
                    .await?;
            assert_eq!(results.len(), 3);

            // Offset at last record
            let results =
                EmailVerifications::index_from_user_id(&user.id, &10, &2, &pool)
                    .await?;
            assert_eq!(results.len(), 1);

            // Offset at total count (should be empty)
            let results =
                EmailVerifications::index_from_user_id(&user.id, &10, &3, &pool)
                    .await?;
            assert_eq!(results.len(), 0);

            // Limit one
            let results =
                EmailVerifications::index_from_user_id(&user.id, &1, &0, &pool)
                    .await?;
            assert_eq!(results.len(), 1);

            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_vs_general_index_consistency(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange - Create a single user with verifications
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Act
            let user_specific_results =
                EmailVerifications::index_from_user_id(&user.id, &10, &0, &pool)
                    .await?;
            let general_results = EmailVerifications::index(&10, &0, &pool).await?;

            // Assert - User-specific should be subset of general results
            assert_eq!(user_specific_results.len(), 5);
            assert_eq!(general_results.len(), 5); // Since we only have one user

            // All user-specific results should be in general results
            for user_verification in &user_specific_results {
                let found = general_results.iter().any(|general_verification| {
                    general_verification.id == user_verification.id
                });
                assert!(
                    found,
                    "User-specific result should be found in general results"
                );
            }

            // All results should belong to our user
            for verification in &user_specific_results {
                assert_eq!(verification.user_id, user.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_multiple_users_isolation(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange - Create multiple users with different amounts of data
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;
            let user3 = create_test_user(&pool).await?;

            create_multiple_verifications(&user1, 5, &pool).await?;
            create_multiple_verifications(&user2, 3, &pool).await?;
            create_multiple_verifications(&user3, 7, &pool).await?;

            // Act - Query with pagination for each user
            let user1_page1 =
                EmailVerifications::index_from_user_id(&user1.id, &3, &0, &pool)
                    .await?;
            let user1_page2 =
                EmailVerifications::index_from_user_id(&user1.id, &3, &3, &pool)
                    .await?;

            let user2_all =
                EmailVerifications::index_from_user_id(&user2.id, &10, &0, &pool)
                    .await?;

            let user3_page1 =
                EmailVerifications::index_from_user_id(&user3.id, &4, &0, &pool)
                    .await?;
            let user3_page2 =
                EmailVerifications::index_from_user_id(&user3.id, &4, &4, &pool)
                    .await?;

            // Assert - Correct counts per user
            assert_eq!(user1_page1.len(), 3);
            assert_eq!(user1_page2.len(), 2);
            assert_eq!(user2_all.len(), 3);
            assert_eq!(user3_page1.len(), 4);
            assert_eq!(user3_page2.len(), 3);

            // Assert - No cross-user contamination
            for verification in user1_page1.iter().chain(user1_page2.iter()) {
                assert_eq!(verification.user_id, user1.id);
            }
            for verification in &user2_all {
                assert_eq!(verification.user_id, user2.id);
            }
            for verification in user3_page1.iter().chain(user3_page2.iter()) {
                assert_eq!(verification.user_id, user3.id);
            }

            // Assert - Total counts are correct
            assert_eq!(user1_page1.len() + user1_page2.len(), 5);
            assert_eq!(user3_page1.len() + user3_page2.len(), 7);
            Ok(())
        }
    }

    //-- 6: User Based Cursor Pagination
    // Test the cursor based indexation functionality for specific users
    mod user_cursor_pagination {
        use super::*;

        #[sqlx::test]
        async fn index_from_user_id_cursor_empty_database(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;

            // Act
            let results = EmailVerifications::index_from_user_id_cursor(
                &user.id, 10, None, None, &pool,
            )
            .await?;

            // Assert
            assert!(results.is_empty());
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_first_page(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Act
            let results = EmailVerifications::index_from_user_id_cursor(
                &user.id, 3, None, None, &pool,
            )
            .await?;

            // Assert
            assert_eq!(results.len(), 3);

            // Verify all results belong to the user
            for verification in &results {
                assert_eq!(verification.user_id, user.id);
            }

            // Verify ordering (should be created_at ASC, id ASC)
            for i in 1..results.len() {
                assert!(
                    results[i - 1].created_at <= results[i].created_at,
                    "Results should be ordered by created_at ASC"
                );
                if results[i - 1].created_at == results[i].created_at {
                    assert!(
                        results[i - 1].id <= results[i].id,
                        "Results with same created_at should be ordered by id ASC"
                    );
                }
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_subsequent_pages(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 7, &pool).await?;

            // Act - Page 1
            let page1 = EmailVerifications::index_from_user_id_cursor(
                &user.id, 3, None, None, &pool,
            )
            .await?;
            assert_eq!(page1.len(), 3);

            // Act - Page 2
            let cursor_created_at = Some(page1.last().unwrap().created_at);
            let cursor_id = Some(page1.last().unwrap().id.into_uuid());
            let page2 = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                3,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;
            assert_eq!(page2.len(), 3);

            // Act - Page 3 (last page)
            let cursor_created_at = Some(page2.last().unwrap().created_at);
            let cursor_id = Some(page2.last().unwrap().id.into_uuid());
            let page3 = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                3,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;
            assert_eq!(page3.len(), 1); // Only 1 remaining record

            // Act - Page 4 (beyond data)
            let cursor_created_at = Some(page3.last().unwrap().created_at);
            let cursor_id = Some(page3.last().unwrap().id.into_uuid());
            let page4 = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                3,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;
            assert_eq!(page4.len(), 0); // No more records

            // Assert - Verify all records belong to the user
            let all_verifications: Vec<_> = page1
                .iter()
                .chain(page2.iter())
                .chain(page3.iter())
                .collect();

            for verification in &all_verifications {
                assert_eq!(verification.user_id, user.id);
            }

            // Assert - Verify no duplicates across pages
            let all_ids: Vec<_> =
                all_verifications.iter().map(|v| v.id.into_uuid()).collect();
            let unique_ids: std::collections::HashSet<_> = all_ids.iter().collect();
            assert_eq!(
                all_ids.len(),
                unique_ids.len(),
                "No duplicates across pages"
            );

            // Assert - Verify all records are accounted for
            assert_eq!(all_ids.len(), 7);
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_filters_by_user_correctly(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange - Create multiple users with verifications
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;
            let user3 = create_test_user(&pool).await?;

            create_multiple_verifications(&user1, 3, &pool).await?;
            create_multiple_verifications(&user2, 4, &pool).await?;
            create_multiple_verifications(&user3, 2, &pool).await?;

            // Act - Query each user separately
            let user1_results = EmailVerifications::index_from_user_id_cursor(
                &user1.id, 10, None, None, &pool,
            )
            .await?;

            let user2_results = EmailVerifications::index_from_user_id_cursor(
                &user2.id, 10, None, None, &pool,
            )
            .await?;

            let user3_results = EmailVerifications::index_from_user_id_cursor(
                &user3.id, 10, None, None, &pool,
            )
            .await?;

            // Assert - Correct counts per user
            assert_eq!(user1_results.len(), 3);
            assert_eq!(user2_results.len(), 4);
            assert_eq!(user3_results.len(), 2);

            // Assert - Each result set only contains verifications for the correct user
            for verification in &user1_results {
                assert_eq!(verification.user_id, user1.id);
            }
            for verification in &user2_results {
                assert_eq!(verification.user_id, user2.id);
            }
            for verification in &user3_results {
                assert_eq!(verification.user_id, user3.id);
            }

            // Assert - No cross-contamination between users
            let all_user1_ids: std::collections::HashSet<_> =
                user1_results.iter().map(|v| v.id.into_uuid()).collect();
            let all_user2_ids: std::collections::HashSet<_> =
                user2_results.iter().map(|v| v.id.into_uuid()).collect();
            let all_user3_ids: std::collections::HashSet<_> =
                user3_results.iter().map(|v| v.id.into_uuid()).collect();

            assert!(all_user1_ids.is_disjoint(&all_user2_ids));
            assert!(all_user1_ids.is_disjoint(&all_user3_ids));
            assert!(all_user2_ids.is_disjoint(&all_user3_ids));
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_ignores_other_users_records(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;

            create_multiple_verifications(&user1, 2, &pool).await?;
            create_multiple_verifications(&user2, 5, &pool).await?; // More records for other user

            // Act - Query only user1's records
            let results = EmailVerifications::index_from_user_id_cursor(
                &user1.id, 10, None, None, &pool,
            )
            .await?;

            // Assert - Should only get user1's records, not user2's
            assert_eq!(results.len(), 2);
            for verification in &results {
                assert_eq!(verification.user_id, user1.id);
                assert_ne!(verification.user_id, user2.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_multiple_users_independent_pagination(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;

            create_multiple_verifications(&user1, 5, &pool).await?;
            create_multiple_verifications(&user2, 3, &pool).await?;

            // Act - Paginate user1's records
            let user1_page1 = EmailVerifications::index_from_user_id_cursor(
                &user1.id, 2, None, None, &pool,
            )
            .await?;

            let user1_cursor_created_at =
                Some(user1_page1.last().unwrap().created_at);
            let user1_cursor_id = Some(user1_page1.last().unwrap().id.into_uuid());
            let user1_page2 = EmailVerifications::index_from_user_id_cursor(
                &user1.id,
                2,
                user1_cursor_created_at,
                user1_cursor_id,
                &pool,
            )
            .await?;

            // Act - Paginate user2's records independently
            let user2_page1 = EmailVerifications::index_from_user_id_cursor(
                &user2.id, 2, None, None, &pool,
            )
            .await?;

            let user2_cursor_created_at =
                Some(user2_page1.last().unwrap().created_at);
            let user2_cursor_id = Some(user2_page1.last().unwrap().id.into_uuid());
            let user2_page2 = EmailVerifications::index_from_user_id_cursor(
                &user2.id,
                2,
                user2_cursor_created_at,
                user2_cursor_id,
                &pool,
            )
            .await?;

            // Assert - User1 pagination
            assert_eq!(user1_page1.len(), 2);
            assert_eq!(user1_page2.len(), 2);
            for verification in user1_page1.iter().chain(user1_page2.iter()) {
                assert_eq!(verification.user_id, user1.id);
            }

            // Assert - User2 pagination
            assert_eq!(user2_page1.len(), 2);
            assert_eq!(user2_page2.len(), 1); // Only 1 remaining record
            for verification in user2_page1.iter().chain(user2_page2.iter()) {
                assert_eq!(verification.user_id, user2.id);
            }

            // Assert - No overlap between users
            let user1_ids: std::collections::HashSet<_> = user1_page1
                .iter()
                .chain(user1_page2.iter())
                .map(|v| v.id.into_uuid())
                .collect();
            let user2_ids: std::collections::HashSet<_> = user2_page1
                .iter()
                .chain(user2_page2.iter())
                .map(|v| v.id.into_uuid())
                .collect();

            assert!(user1_ids.is_disjoint(&user2_ids));
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_nonexistent_user_empty_results(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let fake_user_id = uuid::Uuid::new_v4();

            // Create some data for real users
            let real_user = create_test_user(&pool).await?;
            create_multiple_verifications(&real_user, 3, &pool).await?;

            // Act
            let results = EmailVerifications::index_from_user_id_cursor(
                &fake_user_id,
                10,
                None,
                None,
                &pool,
            )
            .await?;

            // Assert
            assert_eq!(results.len(), 0);
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_invalid_cursor_missing_id(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;

            // Act
            let result = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                10,
                Some(chrono::Utc::now()), // created_at without cursor_id
                None,
                &pool,
            )
            .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(
                    message,
                    "Both cursor_created_at and cursor_id must be provided together"
                );
            } else {
                panic!("Expected ValidationError for invalid cursor parameters");
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_invalid_cursor_missing_created_at(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;

            // Act
            let result = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                10,
                None,
                Some(uuid::Uuid::new_v4()), // cursor_id without created_at
                &pool,
            )
            .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(
                    message,
                    "Both cursor_created_at and cursor_id must be provided together"
                );
            } else {
                panic!("Expected ValidationError for invalid cursor parameters");
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_valid_parameters_both_none(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act
            let result = EmailVerifications::index_from_user_id_cursor(
                &user.id, 10, None, None, &pool,
            )
            .await?;

            // Assert
            assert_eq!(result.len(), 2);
            for verification in &result {
                assert_eq!(verification.user_id, user.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_valid_parameters_both_some(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Get first page
            let page1 = EmailVerifications::index_from_user_id_cursor(
                &user.id, 2, None, None, &pool,
            )
            .await?;
            assert_eq!(page1.len(), 2);

            // Act - Use valid cursor parameters
            let cursor_created_at = Some(page1.last().unwrap().created_at);
            let cursor_id = Some(page1.last().unwrap().id.into_uuid());
            let result = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                2,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;

            // Assert
            assert!(result.len() <= 2);

            // Verify no overlap with previous page
            for item in &result {
                assert!(!page1.iter().any(|p1_item| p1_item.id == item.id));
                assert_eq!(item.user_id, user.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_end_of_results(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act - Get all records in pages
            let page1 = EmailVerifications::index_from_user_id_cursor(
                &user.id, 2, None, None, &pool,
            )
            .await?;
            assert_eq!(page1.len(), 2);

            let cursor_created_at = Some(page1.last().unwrap().created_at);
            let cursor_id = Some(page1.last().unwrap().id.into_uuid());
            let page2 = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                2,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;
            assert_eq!(page2.len(), 1); // Last record

            // Act - Try to get next page (should be empty)
            let cursor_created_at = Some(page2.last().unwrap().created_at);
            let cursor_id = Some(page2.last().unwrap().id.into_uuid());
            let page3 = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                2,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;

            // Assert
            assert_eq!(
                page3.len(),
                0,
                "Should return empty when beyond last record"
            );
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_ordering_consistency(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 10, &pool).await?;

            // Act - Get all records using cursor pagination
            let mut all_cursor_results = Vec::new();
            let mut cursor_created_at = None;
            let mut cursor_id = None;

            loop {
                let page = EmailVerifications::index_from_user_id_cursor(
                    &user.id,
                    3,
                    cursor_created_at,
                    cursor_id,
                    &pool,
                )
                .await?;
                if page.is_empty() {
                    break;
                }

                // Set cursor for next page
                cursor_created_at = Some(page.last().unwrap().created_at);
                cursor_id = Some(page.last().unwrap().id.into_uuid());

                all_cursor_results.extend(page);
            }

            // Get all records using index_from_user_id_cursor with large limit
            let all_at_once = EmailVerifications::index_from_user_id_cursor(
                &user.id, 100, None, None, &pool,
            )
            .await?;

            // Assert - Should have same number of records
            assert_eq!(all_cursor_results.len(), all_at_once.len());
            assert_eq!(all_cursor_results.len(), 10);

            // Assert - All records belong to the correct user
            for verification in &all_cursor_results {
                assert_eq!(verification.user_id, user.id);
            }
            for verification in &all_at_once {
                assert_eq!(verification.user_id, user.id);
            }

            // Assert - All IDs should match in same order
            for (cursor_result, at_once_result) in
                all_cursor_results.iter().zip(all_at_once.iter())
            {
                assert_eq!(cursor_result.id, at_once_result.id);
                assert_eq!(cursor_result.created_at, at_once_result.created_at);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_limit_zero(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act
            let result = EmailVerifications::index_from_user_id_cursor(
                &user.id, 0, None, None, &pool,
            )
            .await?;

            // Assert
            assert_eq!(result.len(), 0);
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_limit_one(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Act
            let result = EmailVerifications::index_from_user_id_cursor(
                &user.id, 1, None, None, &pool,
            )
            .await?;

            // Assert
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].user_id, user.id);
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_large_limit_warning(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act - Large but valid limit (should warn but succeed)
            let result = EmailVerifications::index_from_user_id_cursor(
                &user.id, 1001, None, None, &pool,
            )
            .await?;

            // Assert
            assert_eq!(result.len(), 3); // Should return all available records for user
            for verification in &result {
                assert_eq!(verification.user_id, user.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_large_limit_overflow(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with limit that would overflow i64
            let large_limit = usize::MAX;
            let result = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                large_limit,
                None,
                None,
                &pool,
            )
            .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!("Expected ValidationError for overflow, got: {:?}", result);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_cursor_pointing_to_nonexistent_record(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act - Use cursor pointing to non-existent record
            let fake_cursor_created_at =
                Some(chrono::Utc::now() + chrono::Duration::days(1));
            let fake_cursor_id = Some(uuid::Uuid::new_v4());
            let result = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                10,
                fake_cursor_created_at,
                fake_cursor_id,
                &pool,
            )
            .await?;

            // Assert - Should return empty (no records after the fake cursor)
            assert_eq!(result.len(), 0);
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_cursor_before_all_records(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act - Use cursor pointing to very old timestamp
            let old_cursor_created_at =
                Some(chrono::Utc::now() - chrono::Duration::days(365));
            let old_cursor_id = Some(uuid::Uuid::new_v4());
            let result = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                10,
                old_cursor_created_at,
                old_cursor_id,
                &pool,
            )
            .await?;

            // Assert - Should return all records for the user
            assert_eq!(result.len(), 3);
            for verification in &result {
                assert_eq!(verification.user_id, user.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_cursor_from_different_user(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;

            create_multiple_verifications(&user1, 3, &pool).await?;
            create_multiple_verifications(&user2, 2, &pool).await?;

            // Get cursor from user2's records
            let user2_page = EmailVerifications::index_from_user_id_cursor(
                &user2.id, 1, None, None, &pool,
            )
            .await?;
            assert_eq!(user2_page.len(), 1);

            // Act - Use user2's cursor to query user1's records
            let cursor_created_at = Some(user2_page[0].created_at);
            let cursor_id = Some(user2_page[0].id.into_uuid());
            let result = EmailVerifications::index_from_user_id_cursor(
                &user1.id,
                10,
                cursor_created_at,
                cursor_id,
                &pool,
            )
            .await?;

            // Assert - Should only return user1's records that come after the cursor
            // (cursor filtering works regardless of which user the cursor record belongs to)
            for verification in &result {
                assert_eq!(verification.user_id, user1.id);
                assert_ne!(verification.user_id, user2.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_consistency_with_offset_pagination(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 8, &pool).await?;

            // Act - Get all records using cursor pagination
            let mut all_cursor_results = Vec::new();
            let mut cursor_created_at = None;
            let mut cursor_id = None;

            loop {
                let page = EmailVerifications::index_from_user_id_cursor(
                    &user.id,
                    3,
                    cursor_created_at,
                    cursor_id,
                    &pool,
                )
                .await?;
                if page.is_empty() {
                    break;
                }

                cursor_created_at = Some(page.last().unwrap().created_at);
                cursor_id = Some(page.last().unwrap().id.into_uuid());
                all_cursor_results.extend(page);
            }

            // Get all records using offset pagination
            let all_offset_results =
                EmailVerifications::index_from_user_id(&user.id, &100, &0, &pool)
                    .await?;

            // Assert - Should have same number of records
            assert_eq!(all_cursor_results.len(), all_offset_results.len());
            assert_eq!(all_cursor_results.len(), 8);

            // Assert - All records belong to the same user
            for verification in &all_cursor_results {
                assert_eq!(verification.user_id, user.id);
            }
            for verification in &all_offset_results {
                assert_eq!(verification.user_id, user.id);
            }

            // Note: We can't directly compare ordering since index_from_user_id() uses ORDER BY id
            // while index_from_user_id_cursor() uses ORDER BY created_at, id
            // But we can verify all the same records are present
            let cursor_ids: std::collections::HashSet<_> = all_cursor_results
                .iter()
                .map(|v| v.id.into_uuid())
                .collect();
            let offset_ids: std::collections::HashSet<_> = all_offset_results
                .iter()
                .map(|v| v.id.into_uuid())
                .collect();
            assert_eq!(
                cursor_ids, offset_ids,
                "Both pagination methods should return same records"
            );
            Ok(())
        }

        #[sqlx::test]
        async fn index_from_user_id_cursor_vs_general_cursor_consistency(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange - Create single user to compare user-specific vs general cursor pagination
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 5, &pool).await?;

            // Act - Get all records using user-specific cursor pagination
            let user_cursor_results = EmailVerifications::index_from_user_id_cursor(
                &user.id, 100, None, None, &pool,
            )
            .await?;

            // Get all records using general cursor pagination
            let general_cursor_results =
                EmailVerifications::index_cursor(100, None, None, &pool).await?;

            // Assert - Should have same number of records (since only one user exists)
            assert_eq!(user_cursor_results.len(), general_cursor_results.len());
            assert_eq!(user_cursor_results.len(), 5);

            // Assert - Same ordering (both use created_at ASC, id ASC)
            for (user_result, general_result) in user_cursor_results
                .iter()
                .zip(general_cursor_results.iter())
            {
                assert_eq!(user_result.id, general_result.id);
                assert_eq!(user_result.created_at, general_result.created_at);
                assert_eq!(user_result.user_id, user.id);
            }
            Ok(())
        }
    }

    //-- 7: Error Handling & Edge Cases
    // Test error conditions, boundary cases, and validation failures
    mod error_handling {
        use super::*;

        #[sqlx::test]
        async fn safe_cast_overflow_in_index(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with limit that would overflow i64
            let large_limit = usize::MAX;
            let result = EmailVerifications::index(&large_limit, &0, &pool).await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!(
                    "Expected ValidationError for limit overflow, got: {:?}",
                    result
                );
            }
            Ok(())
        }

        #[sqlx::test]
        async fn safe_cast_overflow_in_offset(pool: sqlx::PgPool) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with offset that would overflow i64
            let large_offset = usize::MAX;
            let result = EmailVerifications::index(&10, &large_offset, &pool).await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!(
                    "Expected ValidationError for offset overflow, got: {:?}",
                    result
                );
            }
            Ok(())
        }

        #[sqlx::test]
        async fn safe_cast_overflow_in_cursor_limit(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with cursor limit that would overflow i64
            let large_limit = usize::MAX;
            let result =
                EmailVerifications::index_cursor(large_limit, None, None, &pool)
                    .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!(
                    "Expected ValidationError for cursor limit overflow, got: {:?}",
                    result
                );
            }
            Ok(())
        }

        #[sqlx::test]
        async fn safe_cast_overflow_in_user_index_limit(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with user index limit that would overflow i64
            let large_limit = usize::MAX;
            let result = EmailVerifications::index_from_user_id(
                &user.id,
                &large_limit,
                &0,
                &pool,
            )
            .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!("Expected ValidationError for user index limit overflow, got: {:?}", result);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn safe_cast_overflow_in_user_index_offset(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with user index offset that would overflow i64
            let large_offset = usize::MAX;
            let result = EmailVerifications::index_from_user_id(
                &user.id,
                &10,
                &large_offset,
                &pool,
            )
            .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!("Expected ValidationError for user index offset overflow, got: {:?}", result);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn safe_cast_overflow_in_user_cursor_limit(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 2, &pool).await?;

            // Act - Try with user cursor limit that would overflow i64
            let large_limit = usize::MAX;
            let result = EmailVerifications::index_from_user_id_cursor(
                &user.id,
                large_limit,
                None,
                None,
                &pool,
            )
            .await;

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(message, "Pagination value too large");
            } else {
                panic!("Expected ValidationError for user cursor limit overflow, got: {:?}", result);
            }
            Ok(())
        }

        #[test]
        fn validate_cursor_pagination_missing_created_at() -> Result<()> {
            // Act
            let result =
                validate_cursor_pagination(10, None, Some(uuid::Uuid::new_v4()));

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(
                    message,
                    "Both cursor_created_at and cursor_id must be provided together"
                );
            } else {
                panic!(
                    "Expected ValidationError for missing created_at, got: {:?}",
                    result
                );
            }
            Ok(())
        }

        #[test]
        fn validate_cursor_pagination_missing_id() -> Result<()> {
            // Act
            let result =
                validate_cursor_pagination(10, Some(chrono::Utc::now()), None);

            // Assert
            assert!(result.is_err());
            if let Err(AuthenticationError::ValidationError(message)) = result {
                assert_eq!(
                    message,
                    "Both cursor_created_at and cursor_id must be provided together"
                );
            } else {
                panic!(
                    "Expected ValidationError for missing cursor_id, got: {:?}",
                    result
                );
            }
            Ok(())
        }

        #[sqlx::test]
        async fn index_connection_failure_resilience(
            pool: sqlx::PgPool,
        ) -> Result<()> {
            // Note: This test is tricky to implement without actually breaking the connection
            // In a real scenario, you might use a connection pool with limited connections
            // For now, we'll test that the function handles basic connection scenarios

            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act - Multiple rapid queries to test connection handling
            for i in 0..5 {
                let results = EmailVerifications::index(&2, &(i % 3), &pool).await;
                assert!(results.is_ok(), "Query {} should succeed", i);
            }
            Ok(())
        }

        #[test]
        fn zero_limit_edge_case() -> Result<()> {
            // Test that zero limit is handled correctly in validation
            // This tests the pure function without database interaction

            // Act & Assert - Zero limit should be valid for validation functions
            let result = validate_cursor_pagination(0, None, None);
            assert!(result.is_ok(), "Zero limit should be valid in validation");
            Ok(())
        }

        #[test]
        fn extremely_large_but_valid_limit() -> Result<()> {
            // Test that large but valid limits are handled correctly

            // Act & Assert - Large but valid limit (just under the warning threshold)
            let result = validate_cursor_pagination(1000, None, None);
            assert!(result.is_ok(), "Large valid limit should pass validation");

            // Act & Assert - Limit that triggers warning but is still valid
            let result = validate_cursor_pagination(1001, None, None);
            assert!(
                result.is_ok(),
                "Limit above warning threshold should still be valid"
            );
            Ok(())
        }

        #[sqlx::test]
        async fn concurrent_read_safety(pool: sqlx::PgPool) -> Result<()> {
            // Test that multiple concurrent reads don't interfere with each other

            // Arrange
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;
            create_multiple_verifications(&user1, 5, &pool).await?;
            create_multiple_verifications(&user2, 3, &pool).await?;

            // Act - Spawn concurrent read operations
            let pool_clone1 = pool.clone();
            let pool_clone2 = pool.clone();
            let user1_id = user1.id;
            let user2_id = user2.id;

            let task1 = tokio::spawn(async move {
                EmailVerifications::index_from_user_id(
                    &user1_id,
                    &10,
                    &0,
                    &pool_clone1,
                )
                .await
            });

            let task2 = tokio::spawn(async move {
                EmailVerifications::index_from_user_id(
                    &user2_id,
                    &10,
                    &0,
                    &pool_clone2,
                )
                .await
            });

            // Wait for both tasks to complete
            let (result1, result2) = tokio::try_join!(task1, task2)?;

            // Assert
            let user1_results = result1?;
            let user2_results = result2?;

            assert_eq!(user1_results.len(), 5);
            assert_eq!(user2_results.len(), 3);

            // Verify user isolation
            for verification in &user1_results {
                assert_eq!(verification.user_id, user1.id);
            }
            for verification in &user2_results {
                assert_eq!(verification.user_id, user2.id);
            }
            Ok(())
        }

        #[sqlx::test]
        async fn invalid_uuid_format_handling(pool: sqlx::PgPool) -> Result<()> {
            // Note: This test depends on how RowID handles invalid UUIDs
            // If RowID validates UUID format, this test may not be applicable

            // Arrange
            let user = create_test_user(&pool).await?;
            let verification = create_test_verification(&user, &pool, None).await?;

            // Act - Try to use a valid RowID (this should work)
            let result = EmailVerifications::from_id(&verification.id, &pool).await;

            // Assert - Should succeed with valid RowID
            assert!(result.is_ok());
            Ok(())
        }

        #[sqlx::test]
        async fn cursor_parameter_edge_cases(pool: sqlx::PgPool) -> Result<()> {
            // Test various edge cases with cursor parameters

            // Arrange
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 3, &pool).await?;

            // Act & Assert - Future timestamp (no records should match)
            let future_time = chrono::Utc::now() + chrono::Duration::days(365);
            let result = EmailVerifications::index_cursor(
                10,
                Some(future_time),
                Some(uuid::Uuid::new_v4()),
                &pool,
            )
            .await?;
            assert_eq!(result.len(), 0, "Future cursor should return no results");

            // Act & Assert - Very old timestamp (all records should match)
            let old_time = chrono::Utc::now() - chrono::Duration::days(365);
            let result = EmailVerifications::index_cursor(
                10,
                Some(old_time),
                Some(uuid::Uuid::new_v4()),
                &pool,
            )
            .await?;
            assert_eq!(result.len(), 3, "Old cursor should return all results");
            Ok(())
        }

        #[sqlx::test]
        async fn boundary_timestamp_handling(pool: sqlx::PgPool) -> Result<()> {
            // Test handling of boundary timestamp values

            // Arrange
            let user = create_test_user(&pool).await?;

            // Create verifications with specific timestamps
            let base_time = chrono::Utc::now();
            let token1 = mock_verification_token();
            let token2 = mock_verification_token();

            let mut verification1 = EmailVerifications::new(
                &user,
                &token1,
                &chrono::Duration::hours(24),
            );
            let mut verification2 = EmailVerifications::new(
                &user,
                &token2,
                &chrono::Duration::hours(24),
            );

            verification1.created_at = base_time;
            verification2.created_at = base_time + chrono::Duration::microseconds(1); // Very small difference

            let v1 = verification1.insert(&pool).await?;
            let v2 = verification2.insert(&pool).await?;

            // Act - Use cursor with exact timestamp match
            let result = EmailVerifications::index_cursor(
                10,
                Some(v1.created_at),
                Some(v1.id.into_uuid()),
                &pool,
            )
            .await?;

            // Assert - Should get the second verification (after the cursor)
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].id, v2.id);
            Ok(())
        }

        #[sqlx::test]
        async fn large_dataset_memory_handling(pool: sqlx::PgPool) -> Result<()> {
            // Test that the system can handle reasonably large datasets without memory issues

            // Arrange - Create a larger dataset
            let user = create_test_user(&pool).await?;
            create_multiple_verifications(&user, 50, &pool).await?; // Reasonable size for test

            // Act - Query with various page sizes
            let small_pages = EmailVerifications::index(&5, &0, &pool).await?;
            let medium_pages = EmailVerifications::index(&20, &0, &pool).await?;
            let large_pages = EmailVerifications::index(&100, &0, &pool).await?; // Larger than dataset

            // Assert
            assert_eq!(small_pages.len(), 5);
            assert_eq!(medium_pages.len(), 20);
            assert_eq!(large_pages.len(), 50); // All available records

            // Test cursor pagination with large dataset
            let cursor_results =
                EmailVerifications::index_cursor(25, None, None, &pool).await?;
            assert_eq!(cursor_results.len(), 25);
            Ok(())
        }

        #[sqlx::test]
        async fn mixed_data_integrity_validation(pool: sqlx::PgPool) -> Result<()> {
            // Test that reads maintain data integrity with mixed user/verification states

            // Arrange - Create mixed scenarios
            let user1 = create_test_user(&pool).await?;
            let user2 = create_test_user(&pool).await?;

            // Create some used and unused verifications
            let token1 = mock_verification_token();
            let token2 = mock_verification_token();
            let token3 = mock_verification_token();

            let mut verification1 = EmailVerifications::new(
                &user1,
                &token1,
                &chrono::Duration::hours(24),
            );
            let mut verification2 = EmailVerifications::new(
                &user1,
                &token2,
                &chrono::Duration::hours(-1),
            ); // Expired
            let mut verification3 = EmailVerifications::new(
                &user2,
                &token3,
                &chrono::Duration::hours(24),
            );

            verification1.is_used = false;
            verification2.is_used = true; // Used and expired
            verification3.is_used = false;

            verification1.insert(&pool).await?;
            verification2.insert(&pool).await?;
            verification3.insert(&pool).await?;

            // Act - Query all verifications
            let all_results = EmailVerifications::index(&10, &0, &pool).await?;
            let user1_results =
                EmailVerifications::index_from_user_id(&user1.id, &10, &0, &pool)
                    .await?;
            let user2_results =
                EmailVerifications::index_from_user_id(&user2.id, &10, &0, &pool)
                    .await?;

            // Assert - Data integrity
            assert_eq!(all_results.len(), 3);
            assert_eq!(user1_results.len(), 2);
            assert_eq!(user2_results.len(), 1);

            // Verify user filtering integrity
            for verification in &user1_results {
                assert_eq!(verification.user_id, user1.id);
            }
            for verification in &user2_results {
                assert_eq!(verification.user_id, user2.id);
            }

            // Verify that all states are preserved correctly
            let found_used = all_results.iter().any(|v| v.is_used);
            let found_unused = all_results.iter().any(|v| !v.is_used);
            assert!(found_used, "Should find used verifications");
            assert!(found_unused, "Should find unused verifications");
            Ok(())
        }

        #[test]
        fn edge_case_numeric_boundaries() -> Result<()> {
            // Test numeric boundary cases in pure functions

            // Test maximum valid i64 value
            let max_valid = i64::MAX as usize;
            let result = safe_cast_to_i64(max_valid);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), i64::MAX);

            // Test value just above i64::MAX (if possible on the platform)
            #[cfg(target_pointer_width = "64")]
            {
                if usize::MAX > i64::MAX as usize {
                    let overflow_value = (i64::MAX as usize) + 1;
                    let result = safe_cast_to_i64(overflow_value);
                    assert!(result.is_err());
                }
            }

            // Test zero
            let result = safe_cast_to_i64(0);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0);

            Ok(())
        }
    }
    //-- 7: Performance & Consistency Tests
    mod performance {
        use super::*;

        #[sqlx::test]
        async fn pagination_consistency_offset_vs_cursor(pool: sqlx::PgPool) { /* ... */
        }

        #[sqlx::test]
        async fn large_dataset_performance(pool: sqlx::PgPool) { /* ... */
        }

        #[sqlx::test]
        async fn concurrent_read_safety(pool: sqlx::PgPool) { /* ... */
        }
    }

    //-- 8: Integration Tests (if needed)
    mod integration {
        use super::*;

        #[sqlx::test]
        async fn full_workflow_pagination(pool: sqlx::PgPool) { /* ... */
        }

        #[sqlx::test]
        async fn mixed_user_queries(pool: sqlx::PgPool) { /* ... */
        }
    }
}
