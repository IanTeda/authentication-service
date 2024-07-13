//-- ./src/rpc/auth.rs

//! Return a result containing a RPC Authentication Service

#![allow(unused)] // For development only

use std::sync::Arc;

use crate::configuration::Configuration;
use crate::domains::{EmailAddress, TokenClaim};
use crate::prelude::*;
use crate::{database, domains};

use secrecy::Secret;
use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::rpc::ledger::authentication_server::Authentication;
use crate::rpc::ledger::{
    Empty, LoginRequest, LogoutRequest, RefreshRequest, ResetPasswordRequest,
    ResetPasswordResponse, SignUpRequest, TokenResponse, UpdatePasswordRequest,
};

/// Authentication service containing a database pool
pub struct AuthenticationService {
    /// Database Arc reference
    database: Arc<Pool<Postgres>>,
    /// Configuration Arc reference
    config: Arc<Configuration>,
}

impl AuthenticationService {
    /// Initiate a new Authentication Service
    pub fn new(database: Arc<Pool<Postgres>>, config: Arc<Configuration>) -> Self {
        Self { database, config }
    }

    /// Shorthand reference to database pool
    fn database_ref(&self) -> &Pool<Postgres> {
        &self.database
    }

    /// Shorthand reference to config
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
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        // Get the AuthenticateRequest from inside the Tonic Request
        let request = request.into_inner();

        // Parse the request email string into an EmailAddress
        let request_email = EmailAddress::parse(&request.email).map_err(|_| {
            BackendError::AuthenticationError("Authentication failed!".to_string())
        })?;

        tracing::debug!("Request email: {}", request_email.as_ref());

        // Get the user from the database using the request email, so we can verify password hash
        let user = database::UserModel::from_user_email(
            &request_email,
            &self.database_ref(),
        )
        .await
        .map_err(|_| {
            tracing::error!(
                "User email not found in database: {}",
                request_email.as_ref()
            );
            BackendError::AuthenticationError("Authentication Failed!".to_string())
        })?;

        tracing::debug!("User retrieved from the database: {}", user.id);

        // Wrap the Token Secret string in a Secret
        let token_secret = Secret::new(self.config.application.token_secret.clone());

        // Wrap request password in a Secret
        let password_secret = Secret::new(request.password);

        // Check password against stored hash
        match user.password_hash.verify_password(&password_secret)? {
            true => {
                tracing::info!("Password verified.");

                // Build an Access Token
                let access_token =
                    domains::AccessToken::new(&token_secret, &user.id).await?;

                tracing::debug!("Using Access Token: {}", access_token);

                // Build a Refresh Token
                let refresh_token =
                    domains::RefreshToken::new(&token_secret, &user.id).await?;

                // Build a new Refresh Token database instance
                let refresh_token_model =
                    database::RefreshTokenModel::new(&user.id, &refresh_token).await;

                // Add Refresh Token to database
                refresh_token_model.insert(&self.database_ref()).await?;

                tracing::debug!("Using Refresh Token: {}", refresh_token);

                // Build Authenticate Response with the token
                let response = TokenResponse {
                    access_token: access_token.to_string(),
                    refresh_token: refresh_token.to_string(),
                };

                // Send Response
                Ok(Response::new(response))
            }
            false => {
                tracing::error!("Password verification failed.");
                Err(Status::unauthenticated("Authentication Failed!"))
            }
        }
    }

    /// Get a new Access Token using the Refresh Token that has a longer life
    #[tracing::instrument(
		name = "Refresh Access Token Request"
		skip(self, request)
		// fields(
        // 	user_email = &request.into_inner().email,
    	// )
	)]
    async fn refresh(
        &self,
        request: Request<RefreshRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        //-- 1. Get the Refresh Token
        // Get the RefreshAuthenticationRequest from inside the Tonic Request
        let request = request.into_inner();
        let refresh_token = request.refresh_token;

        //-- 2. Get & Validate  the Refresh Token Claim
        // Get the Token Secret from config and wrap it in a Secret to help limit leaks
        let token_secret = &self.config_ref().application.token_secret;
        let token_secret = Secret::new(token_secret.to_owned());

        // Using the Token Secret decode the token into a Token Claim
        // This also validates the token expiration, not before and Issuer
        let refresh_token_claim =
            domains::TokenClaim::from_token(&refresh_token, &token_secret)
                .await
                .map_err(|_| {
                    tracing::error!("Refresh Token is invalid!");
                    BackendError::AuthenticationError(
                        "Authentication Failed!".to_string(),
                    )
                })?;

        //-- 3. Check Refresh Token status in database
        let database_record = database::RefreshTokenModel::from_token(
            &refresh_token,
            &self.database_ref(),
        )
        .await?;

        match database_record.is_active {
            true => {
                tracing::info!("Refresh Token is active.");

                //-- 4. Void all Refresh Tokens for associated user ID
                database_record
                    .revoke_all_associated(&self.database_ref())
                    .await?;

                let user_id =
                    Uuid::try_parse(&refresh_token_claim.sub).map_err(|_| {
                        tracing::error!("Unable to parse Uuid");
                        BackendError::AuthenticationError(
                            "Authentication Failed!".to_string(),
                        )
                    })?;

                //-- 5. Generate new Access and Refresh Tokens
                // Build an Access Token
                let access_token =
                    domains::AccessToken::new(&token_secret, &user_id).await?;

                tracing::debug!("Using Access Token: {}", access_token);

                // Build a Refresh Token
                let refresh_token =
                    domains::RefreshToken::new(&token_secret, &user_id).await?;

                // Build a new Refresh Token database instance
                let refresh_token_model =
                    database::RefreshTokenModel::new(&user_id, &refresh_token).await;

                // Add Refresh Token to database
                refresh_token_model.insert(&self.database_ref()).await?;

                tracing::debug!("Using Refresh Token: {}", refresh_token);

                //-- 5. Send new Access Token and Refresh Token
                // Build Authenticate Response with the token
                let response = TokenResponse {
                    access_token: access_token.to_string(),
                    refresh_token: refresh_token.to_string(),
                };

                // Send Response
                Ok(Response::new(response))
                // unimplemented!()
            }
            false => {
                tracing::error!("Refresh Token is not active");
                Err(Status::unauthenticated("Authentication Failed!"))
            }
        }
        //
    }

    async fn update_password(
        &self,
        request: Request<UpdatePasswordRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        todo!()
    }

    async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordResponse>, Status> {
        todo!()
    }

    async fn sign_up(
        &self,
        request: Request<SignUpRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        todo!()
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<Empty>, Status> {
        todo!()
    }
}
