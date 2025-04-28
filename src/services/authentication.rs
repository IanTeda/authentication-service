//-- ./src/rpc/auth.rs

// #![allow(unused)] // For development only

//! # Authentication Service
//!
//! This module contains the AuthenticationService struct and its implementations.
//!
//! The AuthenticationService struct contains a database pool and configuration.
//!
//! The implementation of the AuthenticationService struct contains the following
//! services:
//! - `authentication`: Authenticate a user using their email and password
//! - `refresh`: Get a new Access Token using the Refresh Token that has a longer life
//! - `update_password`: Update my password using the original password and new password
//! - `reset_password`: Reset my password using the original password and new password
//! - `register`: Register a new user
//! - `logout`: Revoke all Sessions for the user in the database
//!

use std::net::IpAddr;
use std::sync::Arc;
use std::time;

use http::header::{HeaderMap, SET_COOKIE};
use secrecy::SecretString;
use sqlx::{Pool, Postgres};
use tonic::metadata::MetadataMap;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::configuration::Configuration;
use crate::rpc::proto::authentication_service_server::AuthenticationService as Authentication;
use crate::rpc::proto::{
    Empty, LoginRequest, LoginResponse, LogoutResponse, RefreshResponse,
    RegisterRequest, RegisterResponse, ResetPasswordRequest, ResetPasswordResponse,
    UpdatePasswordRequest, UpdatePasswordResponse, UserResponse,
};
use crate::{database, domain};
use crate::{prelude::*, utils};

/// Authentication service containing a database pool
pub struct AuthenticationService {
    /// Database Arc reference
    database: Arc<Pool<Postgres>>,

    /// Configuration Arc reference
    config: Arc<Configuration>,
}

impl AuthenticationService {
    /// # New Authentication Service
    ///
    /// Initiate a new Authentication Service using a database pool and configuration
    ///
    /// ## Parameters
    ///
    /// - `database: Arc<Pool<Postgres>>` - Arc reference to the database pool
    /// - `Arc<Configuration>)`: Arc reference to the configuration
    ///
    pub fn new(database: Arc<Pool<Postgres>>, config: Arc<Configuration>) -> Self {
        Self { database, config }
    }

    /// # Authentication Database Pool Reference
    ///
    /// This function is a shorthand reference to the Authentication Service
    /// database pool.
    fn database_ref(&self) -> &Pool<Postgres> {
        &self.database
    }

    /// # Authentication Configuration Reference
    ///
    /// This function is a shorthand reference to the Authentication Service
    /// configuration.
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
    /// Domain types are used to sanitise the email and password before checking
    /// the database.
    /// Once the password is verified the user is check to if they are active and
    /// verified. Following this a access token is generated and a session instance
    /// is saved to the database.
    /// The access token and refresh token from the sessions instance is sent
    /// in response. With the refresh token being sent as a httponly cookie header
    #[tracing::instrument(name = "Authenticate Request: ", skip_all, fields(
        src_address=%request.remote_addr().unwrap(),
    ))]
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let socket_address = request.remote_addr().unwrap();

        // Break the request up into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (_request_metadata, _request_extensions, request_message) =
            request.into_parts();

        //-- 1. Verify the user email and password
        ////////////////////////////////////////////////////////////////////////

        // Parse the request email string into an EmailAddress
        let request_email = domain::EmailAddress::parse(&request_message.email)
            .map_err(|_| {
                tracing::error!(
                    "Error parsing authentication request email address: {}",
                    request_message.email
                );
                AuthenticationError::AuthenticationError(
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
                    AuthenticationError::AuthenticationError(
                        "Authentication Failed!".to_string(),
                    )
                })?;
        tracing::debug!("User retrieved from the database: {}", user.id);

        // Wrap request password in a Secret type to limit accidental exposure
        let password = SecretString::from(request_message.password);

        // Verify the password hash using the password secret.
        // This will return a boolean indicating if the password is valid.
        let is_password_valid =
            user.password_hash.verify_password(&password).map_err(|_| {
                tracing::error!("Password verification failed.");
                AuthenticationError::AuthenticationError(
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

        //-- 2. Generate new Access and Refresh Tokens
        ////////////////////////////////////////////////////////////////////////

        // Get the token secret from the config (it is wrapped in a Secret type
        // to help limit leaks)
        let token_secret = self.config.application.token_secret.clone();

        // Get the JWT issuer from the config (it is wrapped in a Secret type
        // to help limit leaks)
        let jwt_issuer = self.config.application.get_issuer();

        // Get the refresh token duration from the config
        let rt_duration: time::Duration = time::Duration::new(
            self.config.application.refresh_token_duration_minutes * 60,
            0,
        );

        // Get the refresh token duration from the config
        let at_duration: time::Duration = time::Duration::new(
            self.config.application.access_token_duration_minutes * 60,
            0,
        );

        // Build a new Refresh Token
        let refresh_token = domain::RefreshToken::new(
            &token_secret,
            &jwt_issuer,
            &rt_duration,
            &user,
        )?;

        // Build a new Access Token
        let access_token = domain::AccessToken::new(
            &token_secret,
            &jwt_issuer,
            &at_duration,
            &user,
        )?;

        //-- 3. Revoke all user associated sessions and add a new user session
        ////////////////////////////////////////////////////////////////////////

        // Revoke (make inactive) all sessions associated with the user id
        let _revoke_number =
            database::Sessions::revoke_user_id(&user.id, self.database_ref())
                .await?;

        // Get the ip address from the request socket
        let login_ip = socket_address.ip();

        // IpAddress is an enum with two types, so we need to handle both IP cases
        let login_ip = match login_ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                Some(i32::from_be_bytes(octets)) // Convert the octets to a big-endian i32
            }
            IpAddr::V6(_) => None,
        };

        // Create a new session instance
        let new_session =
            database::Sessions::new(&user, &login_ip, &rt_duration, &refresh_token)?;

        // Insert the session into the database
        let session = new_session.insert(self.database_ref()).await?;
        tracing::debug!("Session added to the database: {}", session.id);

        //-- 4. Send the Tonic response
        ////////////////////////////////////////////////////////////////////////

        // Cast user instance into a UserResponse instance
        // This is a conversion from the database user instance to the RPC user instance
        let user_response_message: UserResponse = user.into();

        // Build Authenticate response message
        let response_message = LoginResponse {
            access_token: access_token.to_string(),
            user: Some(user_response_message),
        };

        // Create a new mutable Tonic response. It is mutable because we need to add the set-cookie header
        let mut response = Response::new(response_message);

        // Set the domain for the cookie
        let domain = &self.config.application.get_domain();

        // Build the refresh cookie
        let refresh_cookie =
            session.refresh_token.build_cookie(domain, &rt_duration);

        // Create a new http header map
        let mut http_header = HeaderMap::new();

        // Add refresh cookie to the http header map
        // TODO: I think this can be done using Tonic without the need for a header map
        http_header.insert(SET_COOKIE, refresh_cookie.to_string().parse().unwrap());

        // Add the http header to the rpc response
        *response.metadata_mut() = MetadataMap::from_headers(http_header);

        tracing::info!("The response is: {:#?}", response);

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
    /// Following verification a new access token is generated and AuthenticationResponse
    /// is sent back with the same Refresh Token and User, but a different access
    /// token.
    #[tracing::instrument(
        name = "Refresh Access Token Request: ",
        skip(self, request)
    )]
    async fn refresh(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<RefreshResponse>, Status> {
        // Break up the Tonic Request into its three parts: 1. Metadata; 2. Extensions; 3. Message;
        let (request_metadata, _request_extensions, _request_message) =
            request.into_parts();

        //-- 1. Check the Refresh Token is Valid
        ////////////////////////////////////////////////////////////////////////
        
        tracing::debug!("Check the refresh token is valid");

        let cookie_jar = utils::metadata::get_cookie_jar(&request_metadata)?;
        tracing::debug!("Cookies jar collected: {:#?}", cookie_jar);

        // Initiate the access token string
        let refresh_token_string: String;

        // Get the access token cookie from the request metadata and return the
        // token string without the extra cookie metadata
        match cookie_jar.get("refresh_token") {
            Some(cookie) => {
                refresh_token_string = cookie.value_trimmed().to_string();
            }
            None => {
                tracing::error!(
                    "Refresh token cookie not found in the request header."
                );
                return Err(tonic::Status::unauthenticated(
                    "Authentication Failed!",
                ));
            }
        };
        tracing::debug!("Access token string: {}", refresh_token_string);

        // Get the Token Secret from config and wrap it in a Secret to help limit leaks
        let token_secret = &self.config_ref().application.token_secret;

        // Set the JWT issuer as the ip address of the server
        let issuer = &self.config.application.get_issuer();

        // Using the Token Secret decode the token into a Token Claim
        // This also validates the token expiration, not before and Issuer
        let refresh_token_claim = domain::TokenClaim::parse(
            &refresh_token_string,
            token_secret,
            issuer,
        )
        .map_err(|_| {
            tracing::error!("Refresh Token is invalid!");
            AuthenticationError::AuthenticationError(
                "Authentication Failed!".to_string(),
            )
        })?;

        //-- 2. Check the Session & User are valid
        ////////////////////////////////////////////////////////////////////////

        tracing::debug!("Check the Session and user are valid.");

        // Get the session from the database using the refresh token
        let session = database::Sessions::from_token(
            &refresh_token_string,
            self.database_ref(),
        )
        .await
        .map_err(|_| {
            tracing::error!("Refresh token not in sessions database");
            AuthenticationError::AuthenticationError(
                "Authentication Failed!".to_string(),
            )
        })?;

        // Check if the session is active
        if session.is_active == false {
            tracing::error!("Session is not active");
            AuthenticationError::AuthenticationError(
                "Authentication Failed!".to_string(),
            );
        }
        tracing::info!("Session is active.");

        // Get user id from the refresh token claim
        let user_id = Uuid::try_parse(&refresh_token_claim.sub).map_err(|_| {
            tracing::error!("Unable to parse Uuid");
            AuthenticationError::AuthenticationError(
                "Authentication Failed!".to_string(),
            )
        })?;

        // Check user id in the token claim is in the database
        let user =
            database::Users::from_user_id(&user_id, self.database_ref()).await?;

        //-- 3. Generate new (Refreshed) Access Token
        ////////////////////////////////////////////////////////////////////////

        tracing::debug!("Refresh the access token.");

        // Wrap the Token Secret string in a Secret type to limit accidental exposure
        let token_secret = self.config.application.token_secret.clone();

        // Set the JWT issuer as the ip address of the server
        let jwt_issuer = &self.config.application.get_issuer();

        // Get the refresh token duration from the config
        let at_duration: time::Duration = time::Duration::new(
            self.config.application.access_token_duration_minutes * 60,
            0,
        );

        // Build a new Access Token
        let access_token = domain::AccessToken::new(
            &token_secret,
            &jwt_issuer,
            &at_duration,
            &user,
        )?;
        tracing::debug!("Generated new Access Token: {}", access_token);

        //-- 4. Send the Tonic Refresh Response
        ////////////////////////////////////////////////////////////////////////
        
        tracing::debug!("Send the refresh response.");

        // Cast user instance into a UserResponse instance
        // This is a conversion from the database user instance to the RPC user instance
        let user_response_message: UserResponse = user.into();

        // Build Authenticate Response with the token
        let response_message = RefreshResponse {
            access_token: access_token.to_string(),
            user: Some(user_response_message),
        };

        // Create a new mutable Tonic response. It is mutable because we need to add the set-cookie header
        let response = Response::new(response_message);

        tracing::debug!("The response is: {:#?}", response);

        // Send Response
        Ok(response)
    }

    /// # Update My Password Service
    ///
    /// Update my password using the original password and new password
    ///
    /// This service takes a UpdatePasswordRequest with the original password and new password.
    /// The original password is verified against the password hash in the database.
    /// If the original password is valid, the new password is hashed and updated in the database.
    /// The function then sends a response message with a success boolean and message.
    #[tracing::instrument(name = "Update Password Request: ", skip(self, request))]
    async fn update_password(
        &self,
        request: Request<UpdatePasswordRequest>,
    ) -> Result<Response<UpdatePasswordResponse>, Status> {
        //-- 0. Break the request up into its parts
        let (request_metadata, _extensions, request_message) = request.into_parts();

        //-- 1. Check the Access Token is Valid
        ////////////////////////////////////////////////////////////////////////

        // print!("Request metadata: {:?}", request_metadata);

        // Using the cookie jar utility, extract the cookies form the metadata
        // into a Cookie Jar.
        let cookie_jar = utils::metadata::get_cookie_jar(&request_metadata)?;

        // Initiate the access token string
        let access_token_string: String;

        // Get the access token cookie from the request metadata and return the
        // token string without the extra cookie metadata
        match cookie_jar.get("access_token") {
            Some(cookie) => {
                access_token_string = cookie.value_trimmed().to_string();
            }
            None => {
                tracing::error!(
                    "Access token cookie not found in the request header."
                );
                return Err(tonic::Status::unauthenticated(
                    "Authentication Failed!",
                ));
            }
        };
        tracing::debug!("Access token string: {}", access_token_string);

        // Get the Token Secret from config and wrap it in a Secret to help limit leaks
        let token_secret = &self.config_ref().application.token_secret;
        let token_secret = token_secret.to_owned();

        // Set the JWT issuer as the ip address of the server
        let issuer = &self.config.application.get_issuer();

        // Using the Token Secret decode the Access Token string into a Token Claim.
        // This validates the token expiration, not before and Issuer.
        let access_token_claim =
            domain::TokenClaim::parse(&access_token_string, &token_secret, &issuer)
                .map_err(|_| {
                    tracing::error!(
                        "Access Token is invalid! Unable to parse token claim."
                    );
                    // Return error
                    AuthenticationError::AuthenticationError(
                        "Authentication Failed!".to_string(),
                    )
                })?;
        tracing::debug!("Access Token verified: {}", access_token_claim.jti);

        //-- 2. Get user from database and check status
        ////////////////////////////////////////////////////////////////////////

        // We can only change our own password so use the user_id in the access token
        // Parse token claim user_id string into a UUID
        let user_id: Uuid = access_token_claim.sub.parse().map_err(|_| {
            tracing::error!("Unable to parse user id to UUID!");
            return AuthenticationError::AuthenticationError(
                "Authentication Failed!".to_string(),
            );
        })?;

        // Get the user from the database using the token claim user_id, so we
        // can verify status and password hash
        let mut user = database::Users::from_user_id(&user_id, self.database_ref())
            .await
            .map_err(|_| {
                tracing::error!("User id not found in database: {}", user_id);
                return AuthenticationError::AuthenticationError(
                    "Authentication Failed!".to_string(),
                );
            })?;
        tracing::debug!("User retrieved from the database: {}", user.id);

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

        //-- Verify existing/original password
        let original_password =
            SecretString::from(request_message.password_original);
        if user.password_hash.verify_password(&original_password)? == false {
            tracing::error!("Original password is incorrect");
            return Err(Status::unauthenticated("Authentication Failed!"));
        }
        tracing::debug!("Users original password is verified: {}", user.id);

        //-- 3. Update the password has in the database
        /////////////////////////////////////////////////////////////////////////

        // Wrap the new password in a Secret type to limit accidental exposure
        let new_password = SecretString::from(request_message.password_new);

        // Parse the new password string into a PasswordHash
        let new_password_hash = domain::PasswordHash::parse(new_password)?;

        // Update the user instance with the new password hash
        user.password_hash = new_password_hash;

        // Update the user in the database
        let _user = user.update(&self.database_ref()).await?;
        tracing::debug!("Users password updated in the database: {}", user.id);

        //-- 4. Send the Tonic response
        ////////////////////////////////////////////////////////////////////////

        // Build GRPC response message
        let response_message = UpdatePasswordResponse {
            success: true,
            message: "Password updated successfully".to_string(),
        };

        // Send Response
        Ok(Response::new(response_message))
    }

    /// # Reset My Password Service
    ///
    #[tracing::instrument(name = "Reset Password Request: ", skip(self, request))]
    async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordResponse>, Status> {
        //-- 0. Break the request up into its parts
        let (_metadata, _extensions, _request_message) = request.into_parts();

        unimplemented!()
    }

    /// # Register a User Service
    #[tracing::instrument(name = "Register User Request: ", skip(self, request))]
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        //-- 0. Break the request up into its parts
        let (_metadata, _extensions, _request_message) = request.into_parts();

        unimplemented!()
    }

    /// # Logout Service
    ///
    /// Revoke all Sessions for the user in the database
    ///
    /// This function first validates the Refresh Token sent in the request header.
    /// After validation, all sessions associated with user are revoked (set inactive)
    /// in the database.
    /// The function then sends a response message with a success boolean and message.
    #[tracing::instrument(name = "Log Out User Request: ", skip(self, request))]
    async fn logout(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<LogoutResponse>, Status> {
        // Break up the request into its three parts: 1. Metadata, 2. Extensions & 3. Message
        let (request_metadata, _request_extensions, _request_message) =
            request.into_parts();

        //-- 1. Check the Refresh Token is Valid
        ////////////////////////////////////////////////////////////////////////

        // Get the Token Secret from config and wrap it in a Secret to help limit leaks
        let token_secret = &self.config_ref().application.token_secret;

        // Set the JWT issuer as the ip address of the server
        let issuer = &self.config.application.get_issuer();

        // Get the refresh token from the request header (metadata)
        let refresh_token: domain::RefreshToken =
            domain::RefreshToken::from_header(token_secret, &request_metadata)?;

        // Using the Token Secret decode the token into a Token Claim
        // This also validates the token expiration, not before and Issuer
        let refresh_token_claim = domain::TokenClaim::parse(
            &refresh_token.to_string(),
            token_secret,
            issuer,
        )
        .map_err(|_| {
            tracing::error!("Refresh Token is invalid!");
            AuthenticationError::AuthenticationError(
                "Authentication Failed!".to_string(),
            )
        })?;

        //-- 2. Check the Session & User are Valid
        ////////////////////////////////////////////////////////////////////////

        // Get the session from the database using the refresh token
        let session = database::Sessions::from_token(
            &refresh_token.to_string(),
            self.database_ref(),
        )
        .await
        .map_err(|_| {
            tracing::error!("Refresh token not in sessions database");
            AuthenticationError::AuthenticationError(
                "Authentication Failed!".to_string(),
            )
        })?;

        // Check if the session is active
        if session.is_active == false {
            tracing::error!("Session is not active");
            AuthenticationError::AuthenticationError(
                "Authentication Failed!".to_string(),
            );
        }
        tracing::info!("Session is active.");

        // Get user id from the refresh token claim
        let user_id = Uuid::try_parse(&refresh_token_claim.sub).map_err(|_| {
            tracing::error!("Unable to parse Uuid");
            AuthenticationError::AuthenticationError(
                "Authentication Failed!".to_string(),
            )
        })?;

        // Check user id in the token claim is in the database
        let _user =
            database::Users::from_user_id(&user_id, self.database_ref()).await?;

        //-- 3. Revoke associated session
        ////////////////////////////////////////////////////////////////////////

        // Revoke (make inactive) all sessions associated with the user id
        let rows_revoked = session.revoke_associated(&self.database.as_ref()).await? as i64;

        if rows_revoked == 0 {
            tracing::error!("No sessions revoked");
            return Err(Status::unauthenticated("Authentication Failed!"));
        }

        //-- 4. Send the Tonic response
        ////////////////////////////////////////////////////////////////////////

        // Build Tonic response message
        let response_message = LogoutResponse {
            success: true,
            message: "You are logged out".to_string(),
        };

        let mut response = Response::new(response_message);

        // Create a new http header map
        let mut http_header = HeaderMap::new();

        // Set th header key and value
        // This is the header key that tells the browser to clear the cookies
        // and delete them from the browser
        let header_key = "Clear-Site-Data";
        // The header value needs the quotes escaped
        let header_value = format!(r##""cookies""##);

        // Tell the browser to clear and delete the cookies
        http_header.insert(header_key, http::HeaderValue::try_from(header_value).unwrap());

        // Add the http header to the rpc response
        *response.metadata_mut() = MetadataMap::from_headers(http_header);

        tracing::info!("The response is: {:#?}", response);

        // Send Response
        Ok(response)
    }
}
