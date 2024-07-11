//-- ./src/rpc/auth.rs

//! Return a result containing a RPC Authentication Service

#![allow(unused)] // For development only

use std::sync::Arc;

use crate::configuration::Configuration;
use crate::domains::EmailAddress;
use crate::prelude::BackendError;
use crate::{database, domains};

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
	#[tracing::instrument(
		name = "Authenticate request"
		skip(self, request)
		// fields(
        // 	user_email = &request.into_inner().email,
    	// )
	)]
	async fn authenticate(
		&self,
		request: Request<AuthenticateRequest>,
	) -> Result<Response<AuthenticateResponse>, Status> {
		// Get the AuthenticateRequest from inside the Tonic Request
		let request = request.into_inner();

		// Parse the request email string into an EmailAddress
		let request_email = EmailAddress::parse(&request.email).map_err(|_| {
			BackendError::AuthenticationError("Authentication failed!".to_string())
		})?;

		tracing::info!("Request email: {}", request_email.as_ref());

		// Get the user from the database using the request email, so we can verify password hash
		let user = database::UserModel::from_user_email(&request_email, &self.database_ref())
			.await
			.map_err(|_| {
				tracing::info!("User email not found in database: {}", request_email.as_ref());
				BackendError::AuthenticationError("Authentication Failed!".to_string())
			})?;

		tracing::info!("User {} retrieved from the database.", user.id);

		// Wrap the Token Secret string in a Secret
		let token_secret = Secret::new(self.config.application.token_secret.clone());

		// Wrap request password in a Secret
		let password_secret = Secret::new(request.password);

		// Check password against stored hash
		match user.password_hash.verify_password(&password_secret)? {
			true => {
				tracing::info!("Password verified.");

				// Build JWT access token claim
				let access_token =
					domains::AccessToken::new(&token_secret, &user.id).await?;

				tracing::info!("Access Token: {}", access_token);

				// Build JWT refresh token claim
				let refresh_token =
					domains::RefreshToken::new(&token_secret, &user.id).await?;

				tracing::info!("Refresh Token: {}", refresh_token);

				// Build Authenticate Response with the token
				let response = AuthenticateResponse {
					access_token: access_token.to_string(),
					refresh_token: refresh_token.to_string(),
				};

				// Send Response
				Ok(Response::new(response))
			}
			false => {
				tracing::info!("Password incorrect.");
				Err(Status::unauthenticated("Authentication Failed!"))
			},
		}
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
		// let request = request.into_inner();
		// let email = EmailAddress::parse(request.email)?;
		// let original_password = Secret::new(request.original_password);
		// let new_password = Secret::new(request.new_password);

		// let user =
		// 	database::users::select_user_by_email(&email, &self.database).await?;

		// match verify_password_hash(&original_password, user.password_hash.as_ref())?
		// {
		// 	true => {
		// 		let new_password_hash = PasswordHash::parse(new_password)?;
		// 		let _ = update_password_by_id(
		// 			user.id,
		// 			new_password_hash,
		// 			&self.database,
		// 		)
		// 		.await?;

		// 		let response = AuthenticateResponse {
		// 			access_token: "Bearer some-auth-token".to_string(),
		// 			refresh_token: "Bearer some-auth-token".to_string(),
		// 		};

		// 		Ok(Response::new(response))
		// 	}
		// 	false => Err(Status::unauthenticated("Authentication failed!")),
		// }

		todo!()
	}

	async fn reset_password(
		&self,
		request: Request<ResetPasswordRequest>,
	) -> Result<Response<ResetPasswordResponse>, Status> {
		todo!()
	}

	async fn logout(
		&self,
		request: Request<LogoutRequest>,
	) -> Result<Response<Empty>, Status> {
		todo!()
	}
}
