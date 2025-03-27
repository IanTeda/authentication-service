//-- ./src/rpc/auth.rs

// #![allow(unused)] // For development only

//! Return a result containing an RPC Authentication Service

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time;

use cookie::Cookie;
use http::header::{HeaderMap, SET_COOKIE};
use secrecy::Secret;
use sqlx::{Pool, Postgres};
use tonic::metadata::{MetadataMap, MetadataValue};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::configuration::Configuration;
use crate::prelude::*;
use crate::rpc::proto::authentication_service_server::AuthenticationService as Authentication;
use crate::rpc::proto::{
    AuthenticationRequest, AuthenticationResponse, Empty, LogoutResponse,
    RegisterRequest, ResetPasswordRequest, ResetPasswordResponse,
    UpdatePasswordRequest, UpdatePasswordResponse, UserResponse,
};
use crate::{database, domain};

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
    /// # Authentication Service
    /// 
    /// Authenticate a user using their email and password
    /// 
    /// This function takes a tonic AuthenticationRequest, confirms the user is in
    /// the database, confirms the store password hash matches the password.
    /// Domain types are used to sanities the email and password before checking
    /// the database.
    /// Once the password is verfied the user is check to if they are active and 
    /// verified. Following this a access token is generated and a session istance
    /// is saved to the database.
    /// The access token and refresh token from the sessions instance is sent 
    /// in response. With the refresh token being sent as a httponly cookie header
    #[tracing::instrument(name = "Authenticate Request: ", skip_all, fields(
        src_address=%request.remote_addr().unwrap(),
    ))]
    async fn authentication(
        &self,
        request: Request<AuthenticationRequest>,
    ) -> Result<Response<AuthenticationResponse>, Status> {
        let socket_address = request.remote_addr().unwrap();

        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        // Parse the request email string into an EmailAddress
        let request_email = domain::EmailAddress::parse(&request_message.email)
            .map_err(|_| {
                tracing::error!(
                    "Error parsing authentication request email address: {}",
                    request_message.email
                );
                BackendError::AuthenticationError(
                    "Authentication failed!".to_string(),
                )
            })?;

        tracing::debug!("Request email: {}", request_email.as_ref());

        // Get the user from the database using the request email, so we can verify the password hash
        let user =
            database::Users::from_user_email(&request_email, self.database_ref())
                .await
                .map_err(|_| {
                    tracing::error!(
                        "User email not found in database: {}",
                        request_email.as_ref()
                    );
                    BackendError::AuthenticationError(
                        "Authentication Failed!".to_string(),
                    )
                })?;

        tracing::debug!("User retrieved from the database: {}", user.id);

        // Wrap the Token Secret string in a Secret type to limit accidental exposure
        let token_secret = self.config.application.token_secret.clone();

        // Wrap request password in a Secret type to limit accidental exposure
        let password_secret = Secret::new(request_message.password);

        // Verify the password hash using the password secret.
        // This will return a boolean indicating if the password is valid.
        let is_password_valid = user
            .password_hash
            .verify_password(&password_secret)
            .map_err(|_| {
                tracing::error!("Password verification failed.");
                BackendError::AuthenticationError(
                    "Authentication Failed!".to_string(),
                )
            })?;
        tracing::debug!("Password verified: {}", is_password_valid);

        // If the password is not valid, return an error
        if is_password_valid == false {
            tracing::error!("Password verification failed.");
            return Err(Status::unauthenticated("Authentication Failed!"));
        }

        // Check if the user is active
        if user.is_active == false {
            tracing::error!("User is not active: {}", user.id);
            return Err(Status::unauthenticated("Authentication Failed!"));
        }
        tracing::debug!("User is active in the database: {}", user.id);

        // Get the ip address from the request socket
        let login_ip = socket_address.ip();

        // IpAddress is an enum with two types, so we need to handle both IP cases
        let login_ip = match login_ip {
            IpAddr::V4(ipv4) => Some(ipv4),
            IpAddr::V6(ipv6) => None,
        };

        // Build a new database Login
        let login = database::Logins::new(&user.id, login_ip);

        // Insert Login into the database
        let login = login.insert(self.database_ref()).await?;

        tracing::debug!("Login added to the database: {}", login.id);

        // Set the JWT issuer as the ip address of the server
        let issuer = &self.config.application.ip_address;

        // Build access token duration seconds from config minutes
        let at_duration = time::Duration::new(
            (&self.config.application.access_token_duration_minutes * 60),
            0,
        );

        // Build a new Access Token
        let access_token =
            domain::AccessToken::new(&token_secret, &issuer, &at_duration, &user)?;

        tracing::debug!("Using Access Token: {}", access_token);

        // Build refresh token duration seconds from config minutes
        let rt_duration = time::Duration::new(
            (&self.config.application.refresh_token_duration_minutes * 60),
            0,
        );

        // Build a new Session
        let session =
            database::Sessions::new(&token_secret, &issuer, &rt_duration, &user)?;

        // Insert Session into the database
        let session = session.insert(self.database_ref()).await?;

        tracing::debug!("Session added to the database: {}", session.id);
        tracing::debug!("Using Refresh Token: {}", session.refresh_token);

        // Cast user instance into a UserResponse instance
        // This is a conversion from the database user instance to the RPC user instance
        let user_response_message: UserResponse = user.into();

        // Build Authenticate response message
        let response_message = AuthenticationResponse {
            access_token: access_token.to_string(),
            user: Some(user_response_message),
        };

        // Create a new mutable Tonic response. It is mutable because we need to add the set-cookie header
        let mut response = Response::new(response_message);

        // Set the domain for the cookie
        // TODO: Move this to the config file
        let domain = format!(
            "http://{}:{}",
            self.config.application.ip_address, self.config.application.port
        );

        // Build the refresh cookie
        let refresh_cookie =
            session.refresh_token.build_cookie(&domain, &rt_duration);

        // Create a new http header map
        let mut http_header = HeaderMap::new();

        // Add refresh cookie to the http header map
        // TODO: I think this can be done using Tonic without the need for a header map
        http_header.insert(SET_COOKIE, refresh_cookie.to_string().parse().unwrap());

        // Add the http header to the rpc response
        *response.metadata_mut() = MetadataMap::from_headers(http_header);

        tracing::info!("The response is: {:?}", response);

        // Send Response
        Ok(response)
    }

    /// # Refresh Service
    /// 
    /// Get a new Access Token using the Refresh Token that has a longer life.
    /// 
    /// This service takes an empty request with a refresh token in the header. The
    /// Refresh Token is validated and Token Claim parsed. The Token Claim id is then
    /// checked in sessions database table to confirm it is registered and current.
    /// Following verificatoin a new access token is gernated and AuthenticationResponse
    /// is sent back with the same Refresh Token and User, but a different access
    /// token.
    #[tracing::instrument(
        name = "Refresh Access Token Request: ",
        skip(self, request)
    )]
    async fn refresh(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<AuthenticationResponse>, Status> {
        // Break up the Tonic Request into its three parts: 1. Metadata; 2. Extensions; 3. Message;
        let (request_metadata, _request_extensions, _request_message) =
            request.into_parts();

        // Get the Token Secret from config and wrap it in a Secret to help limit leaks
        let token_secret = &self.config_ref().application.token_secret;

        // Set the JWT issuer as the ip address of the server
        // TODO: Build a issuer in the config file
        let issuer = &self.config.application.ip_address;

        // Get the refresh token from the request header (metadata)
        let refresh_token: domain::RefreshToken = domain::RefreshToken::from_header(
            token_secret,
            issuer,
            &request_metadata,
        )?;

        // Using the Token Secret decode the token into a Token Claim
        // This also validates the token expiration, not before and Issuer
        let refresh_token_claim = domain::TokenClaim::parse(
            &refresh_token.to_string(),
            token_secret,
            issuer,
        )
        .map_err(|_| {
            tracing::error!("Refresh Token is invalid!");
            BackendError::AuthenticationError("Authentication Failed!".to_string())
        })?;

        //-- 3. Check Session status in database
        let session = database::Sessions::from_token(
            &refresh_token.to_string(),
            self.database_ref(),
        )
        .await
        .map_err(|_| {
            tracing::error!("Refresh token not in sessions database");
            BackendError::AuthenticationError("Authentication Failed!".to_string())
        })?;;

        // Check if the session is active
        if session.is_active == false {
            tracing::error!("Session is not active");
            BackendError::AuthenticationError("Authentication Failed!".to_string());
        }
        tracing::info!("Session is active.");

        // Get user id from the refresh token claim
        let user_id = Uuid::try_parse(&refresh_token_claim.sub).map_err(|_| {
            tracing::error!("Unable to parse Uuid");
            BackendError::AuthenticationError("Authentication Failed!".to_string())
        })?;

        // Check user id in the token claim is in the database
        let user =
            database::Users::from_user_id(&user_id, self.database_ref()).await?;

        //-- 5. Generate new Access Token
        // Build an Access Token
        // Build access token duration seconds from config minutes
        let at_duration = time::Duration::new(
            (&self.config.application.access_token_duration_minutes * 60), // TODO: this could be done in the config file
            0, // Milliseconds
        );
        let access_token =
            domain::AccessToken::new(&token_secret, &issuer, &at_duration, &user)?;

        tracing::debug!("Using Access Token: {}", access_token);

        // Cast database user into a Tonic UserResponse
        let user_response_message: UserResponse = user.into();

        //-- 5. Send new Access Token and Refresh Token
        // Build Authenticate Response with the token
        let response_message = AuthenticationResponse {
            access_token: access_token.to_string(),
            user: Some(user_response_message),
        };

        // Create a new mutable Tonic response. It is mutable because we need to add the set-cookie header
        let mut response = Response::new(response_message);

        let domain = format!(
            "http://{}:{}",
            self.config.application.ip_address, self.config.application.port
        );

        // Create a new http header map
        let mut http_header = HeaderMap::new();

        // Add request refresh cookie to the http header map
        http_header.insert(SET_COOKIE, refresh_token.to_string().parse().unwrap());

        // Add the http header to the rpc response
        *response.metadata_mut() = MetadataMap::from_headers(http_header);

        tracing::debug!("The response is: {:?}", response);

        // Send Response
        Ok(response)
    }

    #[tracing::instrument(name = "Update Password Request: ", skip(self, request))]
    async fn update_password(
        &self,
        request: Request<UpdatePasswordRequest>,
    ) -> Result<Response<UpdatePasswordResponse>, Status> {
        //-- 0. Break the request up into its parts
        let (request_metadata, _extensions, request_message) = request.into_parts();

        //-- 1. Get access token and verify
        // Get Access Token from the request
        let access_token = request_metadata
            .get("access_token")
            .ok_or(BackendError::AuthenticationError(
                "Authentication Failed! 1".to_string(),
            ))?
            .to_str()
            .map_err(|_| {
                tracing::error!("Unable to parse access token from header!");
                BackendError::AuthenticationError(
                    "Authentication Failed! 2".to_string(),
                )
            })?;
        tracing::debug!("Using Access Token: {}", access_token);

        // Get the Token Secret from config and wrap it in a Secret to help limit leaks
        let token_secret = &self.config_ref().application.token_secret;
        let token_secret = token_secret.to_owned();

        // Set the JWT issuer as the ip address of the server
        let issuer = &self.config.application.ip_address;

        // Using the Token Secret decode the Access Token into a Token Claim. This also
        // validates the token expiration, not before and Issuer.
        let access_token_claim =
            domain::TokenClaim::parse(&access_token, &token_secret, &issuer)
                .map_err(|_| {
                    tracing::error!("Access Token is invalid!");
                    return BackendError::AuthenticationError(
                        "Authentication Failed!".to_string(),
                    );
                })?;
        // tracing::debug!("Decoded Access Token Claim: {}", access_token_claim);

        //-- 2. Get user from database and check status
        // We can only change our own password so use the user_id in the access token
        // Parse token claim user_id string into a UUID
        let user_id: Uuid = access_token_claim.sub.parse().map_err(|_| {
            tracing::error!("Unable to parse user id to UUID!");
            return BackendError::AuthenticationError(
                "Authentication Failed!".to_string(),
            );
        })?;

        // Get the user from the database using the token claim user_id, so we
        // can verify status and password hash
        let mut user = database::Users::from_user_id(&user_id, self.database_ref())
            .await
            .map_err(|_| {
                tracing::error!("User id not found in database: {}", user_id);
                return BackendError::AuthenticationError(
                    "Authentication Failed!".to_string(),
                );
            })?;

        // Check user is active
        if user.is_active == false {
            tracing::error!("User is not active: {}", user_id);
            return Err(Status::unauthenticated("Authentication Failed!"));
        }
        tracing::debug!("User is active in the database: {}", user.id);

        // Check user email is verified
        if user.is_verified == false {
            tracing::error!("User email is not verified: {}", user_id);
            return Err(Status::unauthenticated("Authentication Failed!"));
        }
        tracing::debug!("User email is verified in the database: {}", user.id);

        //-- 4. Verify existing/original password
        let original_password = Secret::new(request_message.password_original);
        if user.password_hash.verify_password(&original_password)? == false {
            tracing::error!("Original password is incorrect");
            return Err(Status::unauthenticated("Authentication Failed!"));
        }
        tracing::debug!("Users original password is verified: {}", user.id);

        //-- 5. Update the users password in the database
        let new_password = Secret::new(request_message.password_new);
        let new_password_hash = domain::PasswordHash::parse(new_password)?;
        user.password_hash = new_password_hash;
        user.update(&self.database_ref());
        tracing::debug!("Users password updated in the database: {}", user.id);

        let access_token_duration_minutes =
            self.config.application.access_token_duration_minutes;

        let at_duration = time::Duration::new(
            (&self.config.application.access_token_duration_minutes * 60), // Seconds
            0, // Milliseconds
        );

        // Build an new Access Token
        let access_token =
            domain::AccessToken::new(&token_secret, &issuer, &at_duration, &user)?;
        tracing::debug!("Using Access Token: {}", access_token);

        let rt_duration = time::Duration::new(
            (&self.config.application.refresh_token_duration_minutes * 60), // Seconds
            0, // Milliseconds
        );

        // Build a new session instance
        let session =
            database::Sessions::new(&token_secret, &issuer, &rt_duration, &user)?;

        // Revoke sessions associated with the user before adding new one to the database
        // TODO: When do we clean up (delete) the database
        let _rows_affected = session.revoke_associated(self.database_ref()).await?;

        // Add new Session to the database
        let session = session.insert(self.database_ref()).await?;
        tracing::debug!("Using Refresh Token: {}", session.refresh_token);

        // Build GRPC response message
        let response_message = UpdatePasswordResponse {
            success: true,
            message: "Password updated successfully".to_string(),
        };

        // Send Response
        Ok(Response::new(response_message))
    }

    #[tracing::instrument(name = "Reset Password Request: ", skip(self, request))]
    async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordResponse>, Status> {
        //-- 0. Break the request up into its parts
        let (metadata, _extensions, request_message) = request.into_parts();

        unimplemented!()
    }

    #[tracing::instrument(name = "Register User Request: ", skip(self, request))]
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<AuthenticationResponse>, Status> {
        //-- 0. Break the request up into its parts
        let (metadata, _extensions, request_message) = request.into_parts();

        unimplemented!()
    }

    /// Revoke all Sessions in the database
    #[tracing::instrument(name = "Log Out User Request: ", skip(self, request))]
    async fn logout(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<LogoutResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (request_metadata, _request_extensions, _request_message) =
            request.into_parts();

        // Get the Token Secret from config and wrap it in a Secret to help limit leaks
        let token_secret = &self.config_ref().application.token_secret;

        // Set the JWT issuer as the ip address of the server
        // TODO: Build a issuer in the config file
        let issuer = &self.config.application.ip_address;

        // Get the refresh token from the request header (metadata)
        let refresh_token: domain::RefreshToken = domain::RefreshToken::from_header(
            token_secret,
            issuer,
            &request_metadata,
        )?;

        // Using the Token Secret decode the token into a Token Claim
        // This also validates the token expiration, not before and Issuer
        let refresh_token_claim = domain::TokenClaim::parse(
            &refresh_token.to_string(),
            token_secret,
            issuer,
        )
        .map_err(|_| {
            tracing::error!("Refresh Token is invalid!");
            BackendError::AuthenticationError("Authentication Failed!".to_string())
        })?;

        //-- 3. Get Session from database
        let session = database::Sessions::from_token(
            &refresh_token.to_string(),
            self.database_ref(),
        )
        .await?;

        // Revoke all Sessions associated with user_id
        let rows_affected =
            session.revoke_associated(self.database_ref()).await? as i64;

        // Build Tonic response message
        let response_message = LogoutResponse {
            success: true,
            message: "You are logged out".to_string(),
        };

        // Send Response
        Ok(Response::new(response_message))
    }
}
