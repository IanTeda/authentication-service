//-- ./src/middleware/access_token.rs

// #![allow(unused)] // For beginning only.

use secrecy::Secret;

use crate::{domain, prelude::*};

/// Check
#[derive(Clone)]
pub struct AccessTokenInterceptor {
    pub(crate) token_secret: Secret<String>,
}

impl tonic::service::Interceptor for AccessTokenInterceptor {
    fn call(
        &mut self,
        request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        // Unwrap the .get() option
        match request.metadata().get("access_token") {
            Some(access_token) => {
                // Convert Ascii to a string reference
                let access_token = access_token.to_str().map_err(|_| {
                    tracing::error!("Access Token is invalid!");
                    // Return error
                    BackendError::AuthenticationError(
                        "Authentication Failed! No valid auth token.".to_string(),
                    )
                })?;

                // Using the Token Secret decode the Access Token into a Token Claim. This also
                // validates the token expiration, not before and Issuer.
                let access_token_claim =
                    domain::TokenClaim::from_token(access_token, &self.token_secret)
                        .map_err(|_| {
                            tracing::error!("Access Token is invalid!");
                            // Return error
                            BackendError::AuthenticationError(
                                "Authentication Failed! No valid auth token.".to_string(),
                            )
                        })?;

                tracing::info!("Access Token authenticated for user: {}", access_token_claim.sub);

                Ok(request)
            }
            None => {
                tracing::error!("Access Token not in request header");
                Err(tonic::Status::unauthenticated("Authentication Failed! No valid auth token."))
            },
        }
    }
}
