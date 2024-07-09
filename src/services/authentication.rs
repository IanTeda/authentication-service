//-- ./src/rpc/auth.rs

//! Return a result containing a RPC Users service

#![allow(unused)] // For development only

use std::sync::Arc;

use crate::configuration::Configuration;
use crate::database;
use crate::database::refresh_token::RefreshTokenModel;
use crate::database::users::update_password_by_id;
use crate::domains::{
	verify_password_hash, AccessToken, EmailAddress, Password, RefreshToken,
};
use crate::prelude::BackendError;

use secrecy::Secret;
use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};

use crate::rpc::ledger::authentication_server::Authentication;
use crate::rpc::ledger::{
	AuthenticateRequest, AuthenticateResponse, Empty, LogoutRequest,
	RefreshAuthenticationRequest, ResetPasswordRequest, ResetPasswordResponse,
	UpdatePasswordRequest,
};

/// Authentication service containing a database pool
pub struct AuthenticationService {
	database: Arc<Pool<Postgres>>,
	config: Arc<Configuration>,
}

impl AuthenticationService {
	pub fn new(database: Arc<Pool<Postgres>>, config: Arc<Configuration>) -> Self {
		Self { database, config }
	}

	/// Shorthand for reference to database pool
	fn database_ref(&self) -> &Pool<Postgres> {
		&self.database
	}

	fn config_ref(&self) -> &Configuration {
		&self.config
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
		let email = EmailAddress::parse(&request.email).map_err(|_| {
			BackendError::AuthenticationError("Authentication failed!".to_string())
		})?;

		// Wrap the request password into a Secret to help avoid leaking the string
		let password_secret = Secret::new(request.password);

		let token_secret = &self.config.application.token_secret;
		let token_secret = Secret::new(token_secret.to_owned());

		// Get the user from the database to confirm hash
		let user = database::users::select_user_by_email(&email, &self.database)
			.await
			.map_err(|_| {
				BackendError::AuthenticationError(
					"Authentication failed!".to_string(),
				)
			})?;

		// Check password against stored hash
		match verify_password_hash(&password_secret, user.password_hash.as_ref())? {
			true => {
				// Build JWT access token claim
				let access_token = 
					AccessToken::new(&token_secret, &user.id).await?;
				// .map_err(|_| BackendError::AuthenticationError("Token authentication failed!".to_string()))?;

				// Build JWT refresh token claim
				let refresh_token =
					RefreshToken::new(&token_secret, &user.id).await?;
				// .map_err(|_| BackendError::AuthenticationError("Token authentication failed!".to_string()))?;

				//-- Add refresh token to database
				// let refresh_token_model = RefreshTokenModel::new(&user.id, &refresh_token);

				// Build Authenticate Response with the token
				let response = AuthenticateResponse {
					access_token: access_token.to_string(),
					refresh_token: refresh_token.to_string(),
				};

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

		let user =
			database::users::select_user_by_email(&email, &self.database).await?;

		match verify_password_hash(&original_password, user.password_hash.as_ref())?
		{
			true => {
				let new_password_hash = Password::parse(new_password)?;
				let _ = update_password_by_id(
					user.id,
					new_password_hash,
					&self.database,
				)
				.await?;

				let response = AuthenticateResponse {
					access_token: "Bearer some-auth-token".to_string(),
					refresh_token: "Bearer some-auth-token".to_string(),
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

	async fn logout(
		&self,
		request: Request<LogoutRequest>,
	) -> Result<Response<Empty>, Status> {
		unimplemented!()
	}
}
