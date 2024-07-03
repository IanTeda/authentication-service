//-- ./src/rpc/users.rs

//! Return a result containing a RPC Users server

// #![allow(unused)] // For development only

use crate::database;

use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::database::users::UserModel;
use crate::rpc::ledger::users_server::Users;
use crate::rpc::ledger::{CreateUserRequest, UserRequest, UserResponse};

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

#[tonic::async_trait]
impl Users for UsersService {
	async fn create_user(
		&self,
		request: Request<CreateUserRequest>,
	) -> Result<Response<UserResponse>, Status> {
		let create_user_request: UserModel = request.into_inner().try_into().unwrap();

		let created_user = database::users::insert_user(&create_user_request, &self.database)
			.await
			.unwrap();

		let response = UserResponse::from(created_user);

		// Send back our ping response.
		Ok(Response::new(response))
	}

	async fn read_user(
		&self,
		request: Request<UserRequest>
	) -> Result<Response<UserResponse>, Status> {
		let user_request = request.into_inner();
		let request_id: &str = user_request.id.as_str();
		let id = Uuid::try_parse(request_id).unwrap();
		let user = database::users::select_user_by_id(&id, &self.database)
			.await
			.unwrap();

		let response = UserResponse::from(user);

		Ok(Response::new(response))
	}

	// #[tonic::async_trait]
	// async fn index_users(
	// 	&self,
	// 	request: Request<UserIndexRequest>
	// ) -> Result<Response<UserIndexResponse>, Status> {

	// }

	// #[tonic::async_trait]
	// async fn update_user(
	// 	&self,
	// 	request: Request<CreateUserRequest>
	// ) -> Result<Response<UserResponse>, Status> {

	// }

	// #[tonic::async_trait]
	// async fn delete_user(
	// 	&self,
	// 	request: Request<UserRequest>
	// ) -> Result<Response<DeleteUserResponse>, Status> {

	// }
}
