//-- ./src/rpc/users.rs

//! RPC service for users endpoint
//!
//! Contains functions to managing the rpc service endpoints
//! ---

// #![allow(unused)] // For development only

use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use secrecy::SecretString;
use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::configuration::Configuration;
use crate::prelude::AuthenticationError;
use crate::rpc::proto::users_service_server::UsersService as Users;
use crate::rpc::proto::{
    CreateUserRequest, DeleteUserRequest, DeleteUserResponse, ReadUserRequest,
    UpdateUserRequest, UserIndexRequest, UserIndexResponse, UserResponse,
};
use crate::{database, domain};

/// User service containing a database pool
// #[derive(Debug)]
pub struct UsersService {
    database: Arc<Pool<Postgres>>,
    #[allow(dead_code)]
    config: Arc<Configuration>,
}

impl UsersService {
    /// Create a new UserService passing in the Arc for the Sqlx database pool
    pub fn new(database: Arc<Pool<Postgres>>, config: Arc<Configuration>) -> Self {
        Self { database, config }
    }

    /// Shorthand for reference to database pool
    // https://github.com/radhas-kitchen/radhas-kitchen/blob/fe0cc02ddd9275d9b6aa97300701a53618980c9f/src-grpc/src/services/auth.rs#L10
    fn database_ref(&self) -> &Pool<Postgres> {
        &self.database
    }

    #[allow(dead_code)]
    fn config_ref(&self) -> &Configuration {
        &self.config
    }
}

/// Convert a User Request message into a database::Users
impl TryFrom<CreateUserRequest> for database::Users {
    type Error = AuthenticationError;

    fn try_from(value: CreateUserRequest) -> Result<Self, Self::Error> {
        let id = Uuid::now_v7();
        let email = domain::EmailAddress::parse(value.email)?;
        let name = domain::UserName::parse(value.name)?;
        let password = SecretString::from(value.password);
        let password_hash = domain::PasswordHash::parse(password)?;
        let role = domain::UserRole::from_str(&value.role)?;
        let is_active = value.is_active;
        let is_verified = value.is_verified;
        let created_on = Utc::now();

        Ok(Self {
            id,
            email,
            name,
            password_hash,
            role,
            is_active,
            is_verified,
            created_on,
        })
    }
}

/// Convert a User Request message into a database::Users
impl TryFrom<UpdateUserRequest> for database::Users {
    type Error = AuthenticationError;

    fn try_from(value: UpdateUserRequest) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(value.id.as_str())?;
        let email = domain::EmailAddress::parse(value.email)?;
        // Password does not get updated
        let name = domain::UserName::parse(value.name)?;
        let password =
            SecretString::from("this-does-not-matter!-I-do-not-update3".to_string());
        let password_hash = domain::PasswordHash::parse(password)?;
        let role = domain::UserRole::from_str(&value.role)?;
        let is_active = value.is_active;
        let is_verified = value.is_verified;
        // I do not get updated
        let created_on = Utc::now();

        Ok(Self {
            id,
            email,
            name,
            password_hash,
            role,
            is_active,
            is_verified,
            created_on,
        })
    }
}

/// Convert a database::Users into a User Response message
impl From<database::Users> for UserResponse {
    fn from(value: database::Users) -> Self {
        let id: String = value.id.to_string();
        let email = value.email.to_string();
        let name = value.name.to_string();
        let role = value.role.to_string();
        let is_active = value.is_active;
        let is_verified = value.is_verified;
        let created_on = value.created_on.to_string();

        Self {
            id,
            email,
            name,
            role,
            is_active,
            is_verified,
            created_on,
        }
    }
}

#[tonic::async_trait]
impl Users for UsersService {
    /// Handle rpc requests to create a user in the database
    #[tracing::instrument(name = "Create User Request: ", skip(self, request))]
    async fn create(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        tracing::debug!("User request: {request:#?}");

        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        // Convert create user request message into a user instance
        let user: database::Users = request_message.try_into()?;

        // Insert user into the database
        let database_record = user.insert(self.database_ref()).await?;

        // Convert database user record into a user response message
        let response_message: UserResponse = database_record.into();

        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to read a user in the database
    #[tracing::instrument(
        name = "Read User Request: ",
        skip(self, request),
    // fields(
    // 	user_email = &request.into_inner().email,
    // )
    )]
    async fn read(
        &self,
        request: Request<ReadUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        let id = Uuid::parse_str(&request_message.id).map_err(|_| {
            tracing::error!("Unable to parse user id to UUID!");
            return AuthenticationError::Generic(
                "Unable to parse user id to UUID!".to_string(),
            );
        })?;

        let database_record =
            database::Users::from_user_id(&id, self.database_ref()).await?;

        // Convert database user record into a user response message
        let response_message: UserResponse = database_record.into();

        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to get a user index of the database
    #[tracing::instrument(
        name = "Read User Index Request: ",
        skip(self, request),
    // fields(
    // 	user_email = &request.into_inner().email,
    // )
    )]
    async fn index(
        &self,
        request: Request<UserIndexRequest>,
    ) -> Result<Response<UserIndexResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        // Offset, where to start the records from
        let offset = request_message.offset;

        // The number of users to be returned
        let limit = request_message.limit;

        // Query the database
        let database_records =
            database::Users::index(&limit, &offset, self.database_ref()).await?;

        // Convert database::Users into User Response within the vector
        let users_response: Vec<UserResponse> = database_records
            .into_iter()
            .map(|user| user.into())
            .collect();

        // Build tonic response from UserResponse vector
        let response = UserIndexResponse {
            users: users_response,
        };

        Ok(Response::new(response))
    }

    /// Handle rpc requests to update a user in the database
    #[tracing::instrument(
        name = "Update User Request: ",
        skip(self, request),
    // fields(
    // 	user_email = &request.into_inner().email,
    // )
    )]
    async fn update(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        // Convert create user request message into a user instance
        let user: database::Users = request_message.try_into()?;

        // Insert user into the database
        let database_record = user.update(self.database_ref()).await?;

        // Convert database user record into a user response message
        let response_message: UserResponse = database_record.into();

        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to delete a user in the database
    #[tracing::instrument(
        name = "Delete User Request: ",
        skip(self, request),
    // fields(
    // 	user_email = &request.into_inner().email,
    // )
    )]
    async fn delete(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        let id = Uuid::parse_str(&request_message.id).map_err(|_| {
            tracing::error!("Unable to parse user id to UUID!");
            return AuthenticationError::Generic(
                "Unable to parse user id to UUID!".to_string(),
            );
        })?;

        let database_record =
            database::Users::from_user_id(&id, self.database_ref()).await?;

        let rows_affected =
            database_record.delete(self.database_ref()).await? as i64;

        // Convert database user record into a user response message
        let response_message = DeleteUserResponse { rows_affected };

        Ok(Response::new(response_message))
    }
}
