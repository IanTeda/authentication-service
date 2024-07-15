//-- ./src/middleware/authentication.rs

#![allow(unused)] // For development only

//! Authenticate a request
//!
//! ---

use tonic::{metadata::MetadataValue, Request, Status};

pub enum AuthError {
    InvalidToken,
    WrongCredentials,
    TokenCreation,
    MissingCredentials,
}

// impl IntoResponse for AuthError {
//     fn into_response(self) -> Response {
//         let (status, error_message) = match self {
//             AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
//             AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
//             AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
//             AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
//         };
//         let body = Json(json!({
//             "error": error_message,
//         }));
//         (status, body).into_response()
//     }
// }

pub fn check_authentication(request: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "Bearer some-auth-token".parse().unwrap();

    match request.metadata().get("authorization") {
        Some(t) if token == t => Ok(request),
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}
