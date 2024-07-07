//-- ./src/rpc/auth.rs

//! Return a result containing a RPC Users service

#![allow(unused)] // For development only

use std::sync::Arc;

use crate::database;
use crate::database::users::update_password_by_id;
use crate::domains::{verify_password_hash, EmailAddress, Password};
use crate::prelude::BackendError;
use crate::utilities::jwt::{Claims, JwtKeys, JWT_DURATION, JWT_ISSUER};

use secrecy::Secret;
use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};

use crate::rpc::ledger::authentication_server::Authentication;
use crate::rpc::ledger::{
	AuthenticateRequest, AuthenticateResponse, Empty, LogoutRequest, RefreshAuthenticationRequest,
	ResetPasswordRequest, ResetPasswordResponse, UpdatePasswordRequest,
};

/// Authentication service containing a database pool
pub struct AuthenticationService {
	database: Arc<Pool<Postgres>>,
	jwt_keys: Arc<JwtKeys>,
}

impl AuthenticationService {
	pub fn new(database: Arc<Pool<Postgres>>, jwt_keys: Arc<JwtKeys>) -> Self {
		Self { database, jwt_keys }
	}
}

#[tonic::async_trait]
impl Authentication for AuthenticationService {
	async fn authenticate(
		&self,
		request: Request<AuthenticateRequest>,
	) -> Result<Response<AuthenticateResponse>, Status> {
		// Get the AuthenticateRequest
		let request = request.into_inner();

		// Parse the request string into an EmailAddress
		let email = 
			EmailAddress::parse(&request.email)
			.map_err(|e| BackendError::AuthenticationError)?;

		// Wrap the request password into a Secret to help avoid leaking the string
		let password = Secret::new(request.password);

		// Get the user from the database to confirm hash
		let user = 
			database::users::select_user_by_email(&email, &self.database)
			.await
			.map_err(|e| BackendError::AuthenticationError)?;

		// Check password against stored hash
		match verify_password_hash(&password, user.password_hash.as_ref())? {
			true => {
				// Build Json Web Token claim
				let claim = Claims::new(JWT_ISSUER.to_owned(), user.id.to_string(), JWT_DURATION);

				// Build Json Web Token
				let token = 
					claim.to_jwt(&self.jwt_keys)
					.map_err(|e| BackendError::AuthenticationError)?;

				// Build Authenticate Response with the token
				let response = AuthenticateResponse { token };

				// Send Response
				Ok(Response::new(response))
			}
			false => Err(Status::unauthenticated("Authentication failed!")),
			// false => Err(BackendError::AuthenticationError),
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
