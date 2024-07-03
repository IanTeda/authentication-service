//-- ./src/rpc/users.rs

//! Return a result containing a RPC Users service

// #![allow(unused)] // For development only

use crate::database;

use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::database::users::UserModel;
use crate::rpc::ledger::users_server::Users;
use crate::rpc::ledger::{
	CreateUserRequest, DeleteUserResponse, UpdateUserRequest, UserIndexRequest, UserIndexResponse,
	UserRequest, UserResponse,
};

/// User service containing a database pool
#[derive(Debug)]
pub struct UsersService {
	database: Pool<Postgres>,
}

impl UsersService {
	pub fn new(database: Pool<Postgres>) -> Self {
		Self { database }
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
		let request = request.into_inner();
		let create_user_request: UserModel = request.try_into()?;

		let created_user = database::users::insert_user(&create_user_request, &self.database)
			.await
			.unwrap();

		let response = UserResponse::from(created_user);

		Ok(Response::new(response))
	}

	async fn read_user(
		&self,
		request: Request<UserRequest>,
	) -> Result<Response<UserResponse>, Status> {
		let user_request = request.into_inner();
		let request_id: &str = user_request.id.as_str();
		let id = Uuid::try_parse(request_id).unwrap();
		let user = database::users::select_user_by_id(&id, &self.database).await?;

		let response = UserResponse::from(user);

		Ok(Response::new(response))
	}

	async fn index_users(
		&self,
		request: Request<UserIndexRequest>,
	) -> Result<Response<UserIndexResponse>, Status> {
		// Step into request type
		let request = request.into_inner();

		// Get list of users using limit and offset
		let users =
			database::users::select_user_index(&request.limit, &request.offset, &self.database)
				.await?;

		// Initiate user response vector
		// let mut users_response: Vec<UserResponse> = Vec::new();
		// for user in users {
		// 	// Convert UserModel to UserResponse and push to response vector
		// 	users_response.push(UserResponse::from(user));
		// }
		// Iterate over vector and transform UserModel into UserResponse
		let users_response: Vec<UserResponse> = users.into_iter().map(|x| x.into()).collect();

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

		let updated_user =
			database::users::update_user_by_id(&update_user_request, &self.database).await?;

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
		let is_deleted = database::users::delete_user_by_id(&id, &self.database).await?;

		// Build tonic response from UserResponse vector
		let response = DeleteUserResponse { is_deleted };

		Ok(Response::new(response))
	}
}
