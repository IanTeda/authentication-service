//-- ./src/middleware/access_token.rs

// #![allow(unused)] // For beginning only.

use std::str::FromStr;

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
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        // let remote_address = request::remote_addr();

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
                                "Authentication Failed! No valid auth token."
                                    .to_string(),
                            )
                        })?;

                tracing::info!(
                    "Access Token authenticated for user: {}",
                    access_token_claim.sub
                );

                // Parse Token Claim user role into domain type
                let requester_role =
                    domain::UserRole::from_str(&access_token_claim.jur)?;

                // If the User Role in the Token Claim is not Admin return early with Tonic Status error
                // All endpoints in the authentication microservice require admin so we check it here
                if requester_role != domain::UserRole::Admin {
                    tracing::error!(
                        "User request admin endpoint: {}",
                        &access_token_claim.sub
                    );
                    return Err(tonic::Status::unauthenticated(
                        "Admin access required!",
                    ));
                }

                // Add access token claim to request
                // let (request_metadata, request_extensions, request_message) = request.into_parts();

                // let access_token_claim = request_extensions.get::<domain::TokenClaim>().ok_or(
                //     BackendError::Static("Token Claim not found in request extension."),
                // )?;

                // Add Access token to the Tonic request extension for reference in services
                request.extensions_mut().insert(access_token_claim);

                Ok(request)
            }
            None => {
                tracing::error!("Access Token not in request header");
                Err(tonic::Status::unauthenticated(
                    "Authentication Failed! No valid auth token.",
                ))
            }
        }
    }
}
