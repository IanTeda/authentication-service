//-- ./src/rpc/users.rs

//! Return a result containing a RPC Users service

#![allow(unused)] // For development only

use std::sync::Arc;

use crate::database;

use crate::configuration::Configuration;
use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::database::users::UserModel;
use crate::rpc::ledger::users_server::Users;
use crate::rpc::ledger::{
	CreateUserRequest, DeleteUserResponse, UpdateUserRequest, UserIndexRequest,
	UserIndexResponse, UserRequest, UserResponse,
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

impl From<UserModel> for UserResponse {
	fn from(value: UserModel) -> Self {
		let id: String = value.id.to_string();
		let email = value.email.to_string();
		let user_name = value.user_name.to_string();
		let is_active = value.is_active;
		let created_on = value.created_on.to_string();

		Self {
			id,
			email,
			user_name,
			is_active,
			created_on,
		}
	}
}

// TODO: map tonic status to errors
#[tonic::async_trait]
impl Users for UsersService {
	async fn create_user(
		&self,
		request: Request<CreateUserRequest>,
	) -> Result<Response<UserResponse>, Status> {
		// Get the create user request
		let request = request.into_inner();

		// Convert to User Model type
		let create_user_request: database::UserModel = request.try_into()?;

		// Insert into database
		let database_record =
			create_user_request.insert(&self.database_ref()).await?;

		// Build Tonic User Response
		let response = UserResponse::from(database_record);

		Ok(Response::new(response))
	}

	async fn read_user(
		&self,
		request: Request<UserRequest>,
	) -> Result<Response<UserResponse>, Status> {
		// Get the create user request
		let user_request = request.into_inner();

		let request_id: &str = user_request.id.as_str();
		let id = Uuid::try_parse(request_id).unwrap();

		let database_record =
			database::UserModel::from_user_id(&id, self.database_ref()).await?;

		let response = UserResponse::from(database_record);

		Ok(Response::new(response))
	}

	async fn index_users(
		&self,
		request: Request<UserIndexRequest>,
	) -> Result<Response<UserIndexResponse>, Status> {
		// Step into request type
		let request = request.into_inner();

		// Get list of users using limit and offset
		let database_records = database::UserModel::index(
			&request.limit,
			&request.offset,
			self.database_ref(),
		)
		.await?;

		// Iterate over vector and transform UserModel into UserResponse
		let users_response: Vec<UserResponse> =
			database_records.into_iter().map(|x| x.into()).collect();

		// Build tonic response from UserResponse vector
		let response = UserIndexResponse {
			users: users_response,
		};

		Ok(Response::new(response))
	}

	async fn update_user(
		&self,
		request: Request<UpdateUserRequest>,
	) -> Result<Response<UserResponse>, Status> {
		// Step into request type
		let request = request.into_inner();

		let update_user_request: UserModel = request.try_into()?;

		let updated_user = update_user_request.update(self.database_ref()).await?;

		let response = UserResponse::from(updated_user);

		Ok(Response::new(response))
	}

	async fn delete_user(
		&self,
		request: Request<UserRequest>,
	) -> Result<Response<DeleteUserResponse>, Status> {
		let user_request = request.into_inner();
		let request_id: &str = user_request.id.as_str();
		let id = Uuid::try_parse(request_id).unwrap();
		let user =
			database::UserModel::from_user_id(&id, self.database_ref()).await?;
		let rows_affected = user.delete(self.database_ref()).await?;

		let is_deleted = rows_affected == 1;

		let response = DeleteUserResponse { is_deleted };

		Ok(Response::new(response))
	}
}
