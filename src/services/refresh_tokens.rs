//-- ./src/rpc/users.rs

//! RPC service for users endpoint
//!
//! Contains functions to managing the rpc service endpoints
//! ---

// #![allow(unused)] // For development only

use std::str::FromStr;
use std::sync::Arc;

use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{database, domain};
use crate::configuration::Configuration;
use crate::prelude::BackendError;
use crate::rpc::proto::{
    DeleteRefreshTokenRequest, DeleteUserRefreshTokensRequest, Empty,
    RefreshTokensResponse, RevokeRefreshTokenRequest,
    RevokeUserRefreshTokensRequest,
};
use crate::rpc::proto::refresh_tokens_server::RefreshTokens;

/// User service containing a database pool
// #[derive(Debug)]
pub struct RefreshTokensService {
    database: Arc<Pool<Postgres>>,
    config: Arc<Configuration>,
}

impl RefreshTokensService {
    /// Create a new UserService passing in the Arc for the Sqlx database pool
    pub fn new(database: Arc<Pool<Postgres>>, config: Arc<Configuration>) -> Self {
        Self { database, config }
    }

    /// Shorthand for reference to database pool
    fn database_ref(&self) -> &Pool<Postgres> {
        &self.database
    }

    /// Shorthand for reference to application configuration instance
    fn config_ref(&self) -> &Configuration {
        &self.config
    }
}

#[tonic::async_trait]
impl RefreshTokens for RefreshTokensService {
    /// Handle rpc requests to revoke a Refresh Token
    #[tracing::instrument(name = "Revoke a Refresh Token: ", skip(self, request))]
    async fn revoke(
        &self,
        request: Request<RevokeRefreshTokenRequest>,
    ) -> Result<Response<RefreshTokensResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, request_extensions, request_message) =
            request.into_parts();

        //-- 1. Check the token claim user role is admin
        // Get access token claim from request extension
        let access_token_claim =
            request_extensions.get::<domain::TokenClaim>().ok_or(
                BackendError::Static("Token Claim not found in request extension."),
            )?;

        // Parse Token Claim user role into domain type
        let requester_role = domain::UserRole::from_str(&access_token_claim.jur)?;

        // If the User Role in the Token Claim is not Admin return early with Tonic Status error
        if requester_role != domain::UserRole::Admin {
            tracing::error!(
                "User request admin endpoint: {}",
                &access_token_claim.sub
            );
            return Err(Status::unauthenticated("Admin access required!"));
        }

        // Parse the request message string into a Uuid
        let id = Uuid::parse_str(&request_message.id).map_err(|_| {
            tracing::error!("Unable to parse Refresh Token id to UUID!");
            return BackendError::Generic(
                "Unable to parse user id to UUID!".to_string(),
            );
        })?;

        // Revoke refresh token in database based on database row PK (id)
        let rows_affected =
            database::RefreshTokens::revoke_by_id(&id, self.database_ref()).await?
                as i64;

        // Build Refresh Token Response message
        let response_message = RefreshTokensResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to revoke all Refresh Tokens for a user
    #[tracing::instrument(
        name = "Revoke all Refresh Tokens of a User: ",
        skip(self, request)
    )]
    async fn revoke_user(
        &self,
        request: Request<RevokeUserRefreshTokensRequest>,
    ) -> Result<Response<RefreshTokensResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, request_extensions, request_message) =
            request.into_parts();

        //-- 1. Check the token claim user role is admin
        // Get access token claim from request extension
        let access_token_claim =
            request_extensions.get::<domain::TokenClaim>().ok_or(
                BackendError::Static("Token Claim not found in request extension."),
            )?;

        // Parse Token Claim user role into domain type
        let requester_role = domain::UserRole::from_str(&access_token_claim.jur)?;

        // If the User Role in the Token Claim is not Admin return early with Tonic Status error
        if requester_role != domain::UserRole::Admin {
            tracing::error!(
                "User request admin endpoint: {}",
                &access_token_claim.sub
            );
            return Err(Status::unauthenticated("Admin access required!"));
        }

        // Parse the request message string into a Uuid
        let user_id = Uuid::parse_str(&request_message.user_id).map_err(|_| {
            tracing::error!("Unable to parse User id to UUID!");
            return BackendError::Generic(
                "Unable to parse user id to UUID!".to_string(),
            );
        })?;

        // Revoke refresh token in database based on database row PK (id)
        let rows_affected =
            database::RefreshTokens::revoke_user_id(&user_id, self.database_ref())
                .await? as i64;

        // Build Refresh Token Response message
        let response_message = RefreshTokensResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to revoke all Refresh Tokens for a user
    #[tracing::instrument(
        name = "Revoke all Refresh Tokens in the database: ",
        skip(self, request)
    )]
    async fn revoke_all(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<RefreshTokensResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, request_extensions, _request_message) =
            request.into_parts();

        //-- 1. Check the token claim user role is admin
        // Get access token claim from request extension
        let access_token_claim =
            request_extensions.get::<domain::TokenClaim>().ok_or(
                BackendError::Static("Token Claim not found in request extension."),
            )?;

        // Parse Token Claim user role into domain type
        let requester_role = domain::UserRole::from_str(&access_token_claim.jur)?;

        // If the User Role in the Token Claim is not Admin return early with Tonic Status error
        if requester_role != domain::UserRole::Admin {
            tracing::error!(
                "User request admin endpoint: {}",
                &access_token_claim.sub
            );
            return Err(Status::unauthenticated("Admin access required!"));
        }

        // Revoke (set is_active = false) all Access Tokens in the database
        let rows_affected =
            database::RefreshTokens::revoke_all(self.database_ref()).await? as i64;

        // Build Refresh Token Response message
        let response_message = RefreshTokensResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to delete a Refresh Token
    #[tracing::instrument(name = "Delete a Refresh Token: ", skip(self, request))]
    async fn delete(
        &self,
        request: Request<DeleteRefreshTokenRequest>,
    ) -> Result<Response<RefreshTokensResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, request_extensions, request_message) =
            request.into_parts();

        //-- 1. Check the token claim user role is admin
        // Get access token claim from request extension
        let access_token_claim =
            request_extensions.get::<domain::TokenClaim>().ok_or(
                BackendError::Static("Token Claim not found in request extension."),
            )?;

        // Parse Token Claim user role into domain type
        let requester_role = domain::UserRole::from_str(&access_token_claim.jur)?;

        // If the User Role in the Token Claim is not Admin return early with Tonic Status error
        if requester_role != domain::UserRole::Admin {
            tracing::error!(
                "User request admin endpoint: {}",
                &access_token_claim.sub
            );
            return Err(Status::unauthenticated("Admin access required!"));
        }

        // Parse the request message string into a Uuid
        let id = Uuid::parse_str(&request_message.id).map_err(|_| {
            tracing::error!("Unable to parse Refresh Token id to UUID!");
            return BackendError::Generic(
                "Unable to parse user id to UUID!".to_string(),
            );
        })?;

        // Revoke refresh token in database based on database row PK (id)
        let rows_affected =
            database::RefreshTokens::delete_by_id(&id, self.database_ref()).await?
                as i64;

        // Build Refresh Token Response message
        let response_message = RefreshTokensResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to revoke all Refresh Tokens for a user
    #[tracing::instrument(
        name = "Delete all Refresh Tokens of a User: ",
        skip(self, request)
    )]
    async fn delete_user(
        &self,
        request: Request<DeleteUserRefreshTokensRequest>,
    ) -> Result<Response<RefreshTokensResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, request_extensions, request_message) =
            request.into_parts();

        //-- 1. Check the token claim user role is admin
        // Get access token claim from request extension
        let access_token_claim =
            request_extensions.get::<domain::TokenClaim>().ok_or(
                BackendError::Static("Token Claim not found in request extension."),
            )?;

        // Parse Token Claim user role into domain type
        let requester_role = domain::UserRole::from_str(&access_token_claim.jur)?;

        // If the User Role in the Token Claim is not Admin return early with Tonic Status error
        if requester_role != domain::UserRole::Admin {
            tracing::error!(
                "User request admin endpoint: {}",
                &access_token_claim.sub
            );
            return Err(Status::unauthenticated("Admin access required!"));
        }

        // Parse the request message string into a Uuid
        let user_id = Uuid::parse_str(&request_message.user_id).map_err(|_| {
            tracing::error!("Unable to parse User id to UUID!");
            return BackendError::Generic(
                "Unable to parse user id to UUID!".to_string(),
            );
        })?;

        // Revoke refresh token in database based on database row PK (id)
        let rows_affected = database::RefreshTokens::delete_all_user_id(
            &user_id,
            self.database_ref(),
        )
            .await? as i64;

        // Build Refresh Token Response message
        let response_message = RefreshTokensResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to revoke all Refresh Tokens for a user
    #[tracing::instrument(
        name = "Delete all Refresh Tokens in the database: ",
        skip(self, request)
    )]
    async fn delete_all(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<RefreshTokensResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, request_extensions, _request_message) =
            request.into_parts();

        //-- 1. Check the token claim user role is admin
        // Get access token claim from request extension
        let access_token_claim =
            request_extensions.get::<domain::TokenClaim>().ok_or(
                BackendError::Static("Token Claim not found in request extension."),
            )?;

        // Parse Token Claim user role into domain type
        let requester_role = domain::UserRole::from_str(&access_token_claim.jur)?;

        // If the User Role in the Token Claim is not Admin return early with Tonic Status error
        if requester_role != domain::UserRole::Admin {
            tracing::error!(
                "User request admin endpoint: {}",
                &access_token_claim.sub
            );
            return Err(Status::unauthenticated("Admin access required!"));
        }

        // Revoke (set is_active = false) all Access Tokens in the database
        let rows_affected =
            database::RefreshTokens::delete_all(self.database_ref()).await? as i64;

        // Build Refresh Token Response message
        let response_message = RefreshTokensResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }
}
