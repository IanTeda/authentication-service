//-- ./src/middleware/access_token.rs

#![allow(unused)] // For beginning only.

use secrecy::SecretString;

use crate::{domain, middleware::access_token, prelude::*, utils};
use std::str::FromStr;

/// Check
#[derive(Clone)]
pub struct AccessTokenInterceptor {
    pub(crate) token_secret: SecretString,
    pub(crate) issuer: SecretString,
}

impl tonic::service::Interceptor for AccessTokenInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        // Get the metadata from the tonic request
        let metadata: &tonic::metadata::MetadataMap = request.metadata();

        // Using the cookie jar utility, extract the cookies form the metadata
        // into a Cookie Jar.
        let cookie_jar = utils::metadata::get_cookie_jar(metadata)?;

        // Initiate the access token string
        let mut access_token_string: String;

        // Get the access token cookie from the request metadata and return the
        // token string without the extra cookie metadata
        let access_token_cookie = match cookie_jar.get("access_token") {
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

        // Using the Token Secret decode the Access Token string into a Token Claim. 
        // This validates the token expiration, not before and Issuer.
        let access_token_claim = domain::TokenClaim::parse(
            &access_token_string,
            &self.token_secret,
            &self.issuer,
        )
        .map_err(|_| {
            tracing::error!("Access Token is invalid! Unable to parse token claim.");
            // Return error
            AuthenticationError::AuthenticationError(
                "Authentication Failed!".to_string(),
            )
        })?;
        tracing::debug!(
            "Access Token authenticated for user: {}",
            access_token_claim.sub
        );

        // Parse Token Claim user role into domain type
        // TODO: Implement authorisation function to confirm request authorisation
        let role = domain::UserRole::from_str(access_token_claim.jur.as_str())
            .map_err(|_| {
                tracing::error!("Access Token user role is invalid!");
                // Return error
                AuthenticationError::AuthenticationError(
                    "Authentication Failed! No valid auth token.".to_string(),
                )
            })?;

        Ok(request)
    }
}
