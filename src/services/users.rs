//-- ./src/rpc/users.rs

//! Return a result containing a RPC Users service

#![allow(unused)] // For development only

use std::sync::Arc;

use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};

use crate::configuration::Configuration;
use crate::database;
use crate::rpc::ledger::users_server::Users;
use crate::rpc::ledger::{
    CreateUserRequest, DeleteUserResponse, ReadUserRequest, UpdateUserRequest,
    UserIndexRequest, UserIndexResponse, UserRequest, UserResponse,
};

/// User service containing a database pool
// #[derive(Debug)]
pub struct UsersService {
    database: Arc<Pool<Postgres>>,
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

    fn config_ref(&self) -> &Configuration {
        &self.config
    }
}

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

// mod create;

#[tonic::async_trait]
impl Users for UsersService {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        unimplemented!()
    }

    async fn read_user(
        &self,
        request: Request<ReadUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        unimplemented!()
    }

    async fn index_users(
        &self,
        request: Request<UserIndexRequest>,
    ) -> Result<Response<UserIndexResponse>, Status> {
        unimplemented!()
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        unimplemented!()
    }

    async fn delete_user(
        &self,
        request: Request<UserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        unimplemented!()
    }
}
