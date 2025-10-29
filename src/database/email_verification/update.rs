use uuid::Uuid;

use crate::{database::EmailVerifications, domain, AuthenticationError};

impl EmailVerifications {
    #[tracing::instrument(
        name = "Update a Email Verification in the database: ",
        skip(database),
        fields(
            email_verification = ?self,
        )
    )]
    pub async fn update(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, AuthenticationError> {
        let query = sqlx::query_as!(
            EmailVerifications,
            r#"
                UPDATE email_verifications 
                SET user_id = $2, 
                    token = $3, 
                    expires_at = $4, 
                    is_used = $5,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING *
            "#,
            self.id.into_uuid(),
            self.user_id,
            self.token.as_ref(),
            self.expires_at,
            self.is_used,
        )
        .fetch_one(database)
        .await;

        match query {
            Ok(email_verification) => Ok(email_verification),
            Err(e) => {
                tracing::error!(
                    "Failed to update email verification: {} (verification: {:?})",
                    e,
                    self
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(
        name = "Update a Email Verification as used: ",
        skip(database),
        fields(
            email_verification = ?self,
        )
    )]
    pub async fn revoke(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, AuthenticationError> {
        let query = sqlx::query_as!(
            EmailVerifications,
            r#"
                UPDATE email_verifications 
                SET is_used = false,
                    updated_at = NOW() 
                WHERE id = $1
                RETURNING *
            "#,
            self.id.into_uuid(),
        )
        .fetch_one(database)
        .await;

        match query {
            Ok(email_verification) => Ok(email_verification),
            Err(e) => {
                tracing::error!(
                    "Failed to revoke email verification: {} (verification: {:?})",
                    e,
                    self
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(
        name = "Update a Email Verification as used: ",
        skip(database),
        fields(
            id = ?id,
        )
    )]
    pub async fn revoke_by_id(
        id: &domain::RowID,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<Self, AuthenticationError> {
        let query = sqlx::query_as!(
            EmailVerifications,
            r#"
                UPDATE email_verifications 
                SET is_used = false,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING *
            "#,
            id.into_uuid(),
        )
        .fetch_one(database)
        .await;

        match query {
            Ok(email_verification) => Ok(email_verification),
            Err(e) => {
                tracing::error!(
                    "Failed to revoke email verification: {} (verification: {:?})",
                    e,
                    id
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(
        name = "Update Email Verification as used for associated users: ",
        skip(database),
        fields(
            user_id = ?self.user_id,
        )
    )]
    pub async fn revoke_associated(
        &self,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<usize, AuthenticationError> {
        let query = sqlx::query!(
            r#"
                UPDATE email_verifications 
                SET is_used = true,
                    updated_at = NOW()
                WHERE user_id = $1 AND is_used = false
            "#,
            self.user_id,
        )
        .execute(database)
        .await;

        match query {
            Ok(result) => {
                let rows_affected = result.rows_affected() as usize;
                tracing::info!(
                    "Revoked {} email verifications for user: {}",
                    rows_affected,
                    self.user_id
                );
                Ok(rows_affected)
            }
            Err(e) => {
                tracing::error!(
                    "Failed to revoke email verifications for user: {} (user_id: {})",
                    e,
                    self.user_id
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(
        name = "Update all Email Verification as used for associated user id: ",
        skip(database),
        fields(
            user_id = ?user_id,
        )
    )]
    pub async fn revoke_user_associated(
        user_id: &Uuid,
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<usize, AuthenticationError> {
        let query = sqlx::query!(
            r#"
                UPDATE email_verifications 
                SET is_used = true,
                    updated_at = NOW()
                WHERE user_id = $1 AND is_used = true
            "#,
            user_id,
        )
        .execute(database)
        .await;

        match query {
            Ok(result) => {
                let rows_affected = result.rows_affected() as usize;
                tracing::info!(
                    "Revoked {} email verifications for user: {}",
                    rows_affected,
                    user_id
                );
                Ok(rows_affected)
            }
            Err(e) => {
                tracing::error!(
                    "Failed to revoke email verifications for user: {} (user_id: {})",
                    e,
                    user_id
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(
        name = "Update all Email Verification as used for associated user id: ",
        skip(database),
        fields(
            user_id = ?user_id,
        )
    )]
    pub async fn revoke_all(
        database: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<usize, AuthenticationError> {
        let query = sqlx::query!(
            r#"
                UPDATE email_verifications 
                SET is_used = true,
                    updated_at = NOW()
            "#,
        )
        .execute(database)
        .await;

        match query {
            Ok(result) => {
                let rows_affected = result.rows_affected() as usize;
                tracing::info!(
                    "Revoked {} email verifications for user: {}",
                    rows_affected,
                    user_id
                );
                Ok(rows_affected)
            }
            Err(e) => {
                tracing::error!(
                    "Failed to revoke all email verifications: {})",
                    e
                );
                Err(AuthenticationError::DatabaseError(e.to_string()))
            }
        }
    }
}
