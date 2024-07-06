//-- ./src/rpc/auth.rs

//! Return a result containing a RPC Users service

#![allow(unused)] // For development only

use std::sync::Arc;

use crate::database;
use crate::database::users::update_password_by_id;
use crate::domains::{verify_password_hash, EmailAddress, Password};

use secrecy::Secret;
use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};

use crate::rpc::ledger::authentication_server::Authentication;
use crate::rpc::ledger::{
	AuthenticateRequest, AuthenticateResponse, Empty, LogoutRequest, RefreshAuthenticationRequest, ResetPasswordRequest, ResetPasswordResponse, UpdatePasswordRequest
};

/// Authentication service containing a database pool
#[derive(Debug)]
pub struct AuthenticationService {
	database: Arc<Pool<Postgres>>,
}

impl AuthenticationService {
	pub fn new(database: Arc<Pool<Postgres>>) -> Self {
		Self { database }
	}
}

#[tonic::async_trait]
impl Authentication for AuthenticationService {
	async fn authenticate(
		&self,
		request: Request<AuthenticateRequest>,
	) -> Result<Response<AuthenticateResponse>, Status> {
		let request = request.into_inner();
		let email = EmailAddress::parse(&request.email)?;
		let password = Secret::new(request.password);

		let user = database::users::select_user_by_email(&email, &self.database).await?;

		match verify_password_hash(&password, user.password_hash.as_ref())? {
			true => {
				let response = AuthenticateResponse {
					token: "Bearer some-auth-token".to_string(),
				};
				Ok(Response::new(response))
			}
			false => Err(Status::unauthenticated("Authentication failed!")),
		}
		// unimplemented!()
	}

	async fn refresh_authentication(
		&self,
		request: Request<RefreshAuthenticationRequest>,
	) -> Result<Response<AuthenticateResponse>, Status> {
	
		unimplemented!()
	}

	async fn update_password(
		&self,
		request: Request<UpdatePasswordRequest>,
	) -> Result<Response<AuthenticateResponse>, Status> {
		let request = request.into_inner();
		let email = EmailAddress::parse(request.email)?;
		let original_password = Secret::new(request.original_password);
		let new_password = Secret::new(request.new_password);

		let user = database::users::select_user_by_email(&email, &self.database).await?;

		match verify_password_hash(&original_password, user.password_hash.as_ref())? {
			true => {
				let new_password_hash = Password::parse(new_password)?;
				let _ = update_password_by_id(user.id, new_password_hash, &self.database).await?;

				let response = AuthenticateResponse {
					token: "Bearer some-auth-token".to_string(),
				};

				Ok(Response::new(response))
			}
			false => Err(Status::unauthenticated("Authentication failed!")),
		}

		// unimplemented!()
	}

	async fn reset_password(
		&self,
		request: Request<ResetPasswordRequest>,
	) -> Result<Response<ResetPasswordResponse>, Status> {
		unimplemented!()
	}

	async fn logout(&self, request: Request<LogoutRequest>) -> Result<Response<Empty>, Status> {
		unimplemented!()
	}
}
