//-- ./src/rpc/users.rs

//! RPC service for users endpoint
//!
//! Contains functions to managing the rpc service endpoints
//! ---

// #![allow(unused)] // For development only

use std::sync::Arc;

use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::configuration::Configuration;
use crate::database;
use crate::prelude::AuthenticationError;
use crate::rpc::proto::sessions_service_server::SessionsService as Sessions;
use crate::rpc::proto::{
    Empty, SessionsDeleteRequest, SessionsDeleteResponse, SessionsDeleteUserRequest,
    SessionsIndexRequest, SessionsIndexResponse, SessionsReadRequest,
    SessionsResponse, SessionsRevokeRequest, SessionsRevokeResponse,
    SessionsRevokeUserRequest,
};

/// User service containing a database pool
// #[derive(Debug)]
pub struct SessionsService {
    database: Arc<Pool<Postgres>>,
    #[allow(dead_code)]
    config: Arc<Configuration>,
}

impl SessionsService {
    /// Create a new UserService passing in the Arc for the Sqlx database pool
    pub fn new(database: Arc<Pool<Postgres>>, config: Arc<Configuration>) -> Self {
        Self { database, config }
    }

    /// Shorthand for reference to database pool
    fn database_ref(&self) -> &Pool<Postgres> {
        &self.database
    }

    /// Shorthand for reference to application configuration instance
    #[allow(dead_code)]
    fn config_ref(&self) -> &Configuration {
        &self.config
    }
}

impl From<database::Sessions> for SessionsResponse {
    /// Convert from database::Logins to proto::LoginsResponse
    fn from(value: database::Sessions) -> Self {
        let id = value.id.to_string();
        let user_id = value.user_id.to_string();
        let login_on = value.login_on.to_string();
        let login_ip = value.login_ip;
        let expires_on = value.expires_on.to_string();
        let refresh_token = value.refresh_token.to_string();
        let is_active = value.is_active;
        let logout_on = if value.logout_on.is_none() {
            None
        } else {
            Some(value.logout_on.unwrap().to_string())
        };
        let logout_ip = value.logout_ip;

        Self {
            id,
            user_id,
            login_on,
            login_ip,
            expires_on,
            refresh_token,
            is_active,
            logout_on,
            logout_ip,
        }
    }
}

#[tonic::async_trait]
impl Sessions for SessionsService {
    /// Handle rpc requests to revoke a Session
    #[tracing::instrument(name = "Read a Session: ", skip(self, request))]
    async fn read(
        &self,
        request: Request<SessionsReadRequest>,
    ) -> Result<Response<SessionsResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        // Parse the request message string into a Uuid
        let id = Uuid::parse_str(&request_message.id).map_err(|_| {
            tracing::error!("Unable to parse Session id to UUID!");
            return AuthenticationError::Generic(
                "Unable to parse user id to UUID!".to_string(),
            );
        })?;

        let database_record =
            database::Sessions::from_id(&id, self.database_ref()).await?;

        // Convert the database record into a LoginsResponse message
        let response_message: SessionsResponse = database_record.into();
        // println!("{response_message:#?}");

        // Send Tonic response with our response message
        Ok(Response::new(response_message))
    }

    #[tracing::instrument(name = "Index of : ", skip(self, request))]
    async fn index(
        &self,
        request: Request<SessionsIndexRequest>,
    ) -> Result<Response<SessionsIndexResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        // Offset, where to start the records from
        let offset: i64 = request_message.offset.into();

        // The number of users to be returned
        let limit: i64 = request_message.limit.into();

        // Query the database
        let database_records =
            database::Sessions::index(&limit, &offset, self.database_ref()).await?;

        // Convert database::Users into User Response within the vector
        let sessions: Vec<SessionsResponse> = database_records
            .into_iter()
            .map(|session| session.into())
            .collect();

        // Build tonic response from UserResponse vector
        let response = SessionsIndexResponse { sessions };

        Ok(Response::new(response))
    }

    /// Handle rpc requests to revoke a Session
    #[tracing::instrument(name = "Revoke a Session: ", skip(self, request))]
    async fn revoke(
        &self,
        request: Request<SessionsRevokeRequest>,
    ) -> Result<Response<SessionsRevokeResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        // Parse the request message string into a Uuid
        let id = Uuid::parse_str(&request_message.id).map_err(|_| {
            tracing::error!("Unable to parse Session id to UUID!");
            return AuthenticationError::Generic(
                "Unable to parse user id to UUID!".to_string(),
            );
        })?;

        // Revoke Session in database based on database row PK (id)
        let rows_affected =
            database::Sessions::revoke_by_id(&id, self.database_ref()).await? as i64;

        // Build Session Response message
        let response_message = SessionsRevokeResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to revoke all Sessions for a user
    #[tracing::instrument(
        name = "Revoke all Sessions of a User: ",
        skip(self, request)
    )]
    async fn revoke_user(
        &self,
        request: Request<SessionsRevokeUserRequest>,
    ) -> Result<Response<SessionsRevokeResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        // Parse the request message string into a Uuid
        let user_id = Uuid::parse_str(&request_message.user_id).map_err(|_| {
            tracing::error!("Unable to parse User id to UUID!");
            return AuthenticationError::Generic(
                "Unable to parse user id to UUID!".to_string(),
            );
        })?;

        // Revoke Sessions in database based on database row PK (id)
        let rows_affected =
            database::Sessions::revoke_user_id(&user_id, self.database_ref()).await?
                as i64;

        // Build Sessions Response message
        let response_message = SessionsRevokeResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to revoke all Sessions for a user
    #[tracing::instrument(
        name = "Revoke all Sessions in the database: ",
        skip(self, request)
    )]
    async fn revoke_all(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<SessionsRevokeResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, _request_message) =
            request.into_parts();

        // Revoke (set is_active = false) all Access Tokens in the database
        let rows_affected =
            database::Sessions::revoke_all(self.database_ref()).await? as i64;

        // Build Session Response message
        let response_message = SessionsRevokeResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to delete a Session
    #[tracing::instrument(name = "Delete a Session: ", skip(self, request))]
    async fn delete(
        &self,
        request: Request<SessionsDeleteRequest>,
    ) -> Result<Response<SessionsDeleteResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        // Parse the request message string into a Uuid
        let id = Uuid::parse_str(&request_message.id).map_err(|_| {
            tracing::error!("Unable to parse Sessionid to UUID!");
            return AuthenticationError::Generic(
                "Unable to parse user id to UUID!".to_string(),
            );
        })?;

        // Revoke Session in database based on database row PK (id)
        let rows_affected =
            database::Sessions::delete_by_id(&id, self.database_ref()).await? as i64;

        // Build Session Response message
        let response_message = SessionsDeleteResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to revoke all Session for a user
    #[tracing::instrument(
        name = "Delete all Sessions of a User: ",
        skip(self, request)
    )]
    async fn delete_user(
        &self,
        request: Request<SessionsDeleteUserRequest>,
    ) -> Result<Response<SessionsDeleteResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        // Parse the request message string into a Uuid
        let user_id = Uuid::parse_str(&request_message.user_id).map_err(|_| {
            tracing::error!("Unable to parse User id to UUID!");
            return AuthenticationError::Generic(
                "Unable to parse user id to UUID!".to_string(),
            );
        })?;

        // Revoke Session in database based on database row PK (id)
        let rows_affected =
            database::Sessions::delete_all_user(&user_id, self.database_ref())
                .await? as i64;

        // Build Session Response message
        let response_message = SessionsDeleteResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to revoke all Sessions for a user
    #[tracing::instrument(
        name = "Delete all Sessions in the database: ",
        skip(self, request)
    )]
    async fn delete_all(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<SessionsDeleteResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, _request_message) =
            request.into_parts();

        // Revoke (set is_active = false) all Access Tokens in the database
        let rows_affected =
            database::Sessions::delete_all(self.database_ref()).await? as i64;

        // Build Session Response message
        let response_message = SessionsDeleteResponse { rows_affected };

        // Send Tonic response
        Ok(Response::new(response_message))
    }
}
