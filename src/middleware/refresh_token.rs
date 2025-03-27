//-- ./src/middleware/refresh_token.rs

// #![allow(unused)] // For beginning only.

use std::str::FromStr;

use secrecy::Secret;

use crate::{domain, prelude::*};

/// Check
#[derive(Clone)]
pub struct RefreshTokenInterceptor<'a>  {
    pub(crate) token_secret: Secret<String>,
    pub issuer: &'a str,
}

impl<'a>  tonic::service::Interceptor for RefreshTokenInterceptor<'a>  {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {

        // Unwrap the .get() option
        match request.metadata().get("refresh_token") {
            Some(refresh_token) => {
                // Convert Ascii to a string reference
                let refresh_token = refresh_token.to_str().map_err(|_| {
                    tracing::error!("Access Token is invalid!");
                    // Return error
                    BackendError::AuthenticationError(
                        "Authentication Failed!".to_string(),
                    )
                })?;

                // Using the Token Secret decode the Refresh Token into a Token Claim. This also
                // validates the token expiration, not before and Issuer.
                let refresh_token_claim =
                    domain::TokenClaim::parse(refresh_token, &self.token_secret, self.issuer)
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
                    refresh_token_claim.sub
                );

                // Parse Token Claim user role into domain type
                let requester_role =
                    domain::UserRole::from_str(&refresh_token_claim.jur)?;

                // If the User Role in the Token Claim is not Admin return early with Tonic Status error
                // All endpoints in the authentication microservice require admin so we check it here
                if requester_role != domain::UserRole::Admin {
                    tracing::error!(
                        "User request admin endpoint: {}",
                        &refresh_token_claim.sub
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
                request.extensions_mut().insert(refresh_token_claim);

                Ok(request)
            }
            None => {
                tracing::error!("Refresh Token not in request header");
                Err(tonic::Status::unauthenticated(
                    "Authentication Failed!",
                ))
            }
        }
    }
}
