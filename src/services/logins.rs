//-- ./src/rpc/logins.rs

//! RPC service for Logins endpoint
//!
//! Contains functions to managing the rpc service endpoints
//! ---

// #![allow(unused)] // For development only

use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use secrecy::Secret;
use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::configuration::Configuration;
use crate::rpc::proto::logins_server::Logins;
use crate::rpc::proto::{
    self, LoginsCreateRequest, LoginsDeleteRequest, LoginsDeleteResponse,
    LoginsIndexRequest, LoginsIndexResponse, LoginsReadRequest, LoginsResponse,
    LoginsUpdateRequest,
};
use crate::{database, domain, BackendError};

/// User service containing a database pool
// #[derive(Debug)]
pub struct LoginsService {
    database: Arc<Pool<Postgres>>,
    config: Arc<Configuration>,
}

impl LoginsService {
    /// Create a new UserService passing in the Arc for the Sqlx database pool
    pub fn new(database: Arc<Pool<Postgres>>, config: Arc<Configuration>) -> Self {
        Self { database, config }
    }

    /// Shorthand for reference to database pool
    // https://github.com/radhas-kitchen/radhas-kitchen/blob/fe0cc02ddd9275d9b6aa97300701a53618980c9f/src-grpc/src/services/auth.rs#L10
    fn database_ref(&self) -> &Pool<Postgres> {
        &self.database
    }

    /// Shorthand for reference to Configuration instance
    fn config_ref(&self) -> &Configuration {
        &self.config
    }
}

impl From<database::Logins> for LoginsResponse {
    /// Convert from database::Logins to proto::LoginsResponse
    fn from(value: database::Logins) -> Self {
        let id = value.id.to_string();
        let user_id = value.user_id.to_string();
        let login_on = value.login_on.to_string();
        let login_ip = value.login_ip;

        Self {
            id,
            user_id,
            login_on,
            login_ip,
        }
    }
}

/// Convert a Logins Request message into a database::Logins
impl TryFrom<LoginsCreateRequest> for database::Logins {
    type Error = BackendError;

    /// Try to convert from proto::LoginCreateRequest to a database:Logins
    fn try_from(value: LoginsCreateRequest) -> Result<Self, Self::Error> {
        let id = Uuid::now_v7();
        let user_id = Uuid::parse_str(value.user_id.as_str())?;
        let login_on = Utc::now();
        let login_ip = value.login_ip;

        Ok(Self {
            id,
            user_id,
            login_on,
            login_ip,
        })
    }
}

/// Convert a Logins Request message into a database::Logins
impl TryFrom<LoginsUpdateRequest> for database::Logins {
    type Error = BackendError;

    /// Try to convert from proto::LoginCreateRequest to a database:Logins
    fn try_from(value: LoginsUpdateRequest) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&value.id)?;
        let user_id = Uuid::parse_str(&value.user_id)?;
        let login_on: DateTime<Utc> = value.login_on.parse()?;
        let login_ip = value.login_ip;

        Ok(Self {
            id,
            user_id,
            login_on,
            login_ip,
        })
    }
}

#[tonic::async_trait]
impl Logins for LoginsService {
    /// Handle rpc requests to create a login in the database
    #[tracing::instrument(name = "Create Login Request: ", skip_all)]
    async fn create(
        &self,
        request: Request<LoginsCreateRequest>,
    ) -> Result<Response<LoginsResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, request_extensions, request_message) =
            request.into_parts();
        // println!("{request_message:#?}");

        // Convert the LoginsCreateRequest into a database::Logins
        let login: database::Logins = request_message.try_into()?;
        // println!("{login:#?}");

        // Insert Logins into the database
        let database_record = login.insert(self.database_ref()).await?;
        // println!("{database_record:#?}");

        // Convert the database record into a LoginsResponse message
        let response_message: LoginsResponse = database_record.into();
        // println!("{response_message:#?}");

        // Send Tonic response with our response message
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to read a login in the database
    #[tracing::instrument(name = "Read Login Request: ", skip_all)]
    async fn read(
        &self,
        request: Request<LoginsReadRequest>,
    ) -> Result<Response<LoginsResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, request_extensions, request_message) =
            request.into_parts();

        // Parse response login id string into a Uuid
        let id = Uuid::parse_str(&request_message.id).map_err(|_| {
            tracing::error!("Unable to parse login id to UUID!");
            return BackendError::Generic(
                "Unable to parse login id to UUID!".to_string(),
            );
        })?;

        // Retrieve database for request login id
        let database_record =
            database::Logins::from_id(&id, self.database_ref()).await?;

        // Convert the database record into a LoginsResponse message
        let response_message: LoginsResponse = database_record.into();

        // Send Tonic response with our response message
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to get a user index of the database
    #[tracing::instrument(name = "Read Logins Index Request: ", skip_all)]
    async fn index(
        &self,
        request: Request<LoginsIndexRequest>,
    ) -> Result<Response<LoginsIndexResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, request_extensions, request_message) =
            request.into_parts();

        // TODO: Why does this need to be i64, could we use i32
        // Offset, where to start the records from
        let offset: i64 = request_message.offset.into();

        // The number of users to be returned
        let limit: i64 = request_message.limit.into();

        // Query the database
        let database_records =
            database::Logins::index(&limit, &offset, self.database_ref()).await?;

        // Convert database::Users into User Response within the vector
        let logins: Vec<LoginsResponse> = database_records
            .into_iter()
            .map(|login| login.into())
            .collect();

        // Build tonic response from UserResponse vector
        let response = LoginsIndexResponse { logins };

        Ok(Response::new(response))
    }

    /// Handle rpc requests to update a user in the database
    #[tracing::instrument(name = "Update Login Request: ", skip_all)]
    async fn update(
        &self,
        request: Request<LoginsUpdateRequest>,
    ) -> Result<Response<LoginsResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, request_extensions, request_message) =
            request.into_parts();
        // println!("{request_message:#?} request");

        // Convert the LoginsCreateRequest into a database::Logins
        let login: database::Logins = request_message.try_into()?;
        // println!("{login:#?} from request");

        // Insert Logins into the database
        let database_record = login.update(self.database_ref()).await?;
        // println!("{database_record:#?} database");

        // Convert the database record into a LoginsResponse message
        let response_message: LoginsResponse = database_record.into();
        // println!("{response_message:#?} response");

        // Send Tonic response with our response message
        Ok(Response::new(response_message))
    }

    /// Handle rpc requests to delete a user in the database
    #[tracing::instrument(name = "Delete Logins Request: ", skip_all)]
    async fn delete(
        &self,
        request: Request<LoginsDeleteRequest>,
    ) -> Result<Response<LoginsDeleteResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, request_extensions, request_message) =
            request.into_parts();

        // Parse response login id string into a Uuid
        let id = Uuid::parse_str(&request_message.id).map_err(|_| {
            tracing::error!("Unable to parse login id to UUID!");
            return BackendError::Generic(
                "Unable to parse login id to UUID!".to_string(),
            );
        })?;

        // Retrieve database for request login id
        let rows_affected =
            database::Logins::delete_by_id(&id, self.database_ref()).await? as i64;

        // Convert database user record into a user response message
        let response_message = LoginsDeleteResponse { rows_affected };

        Ok(Response::new(response_message))
    }
}
