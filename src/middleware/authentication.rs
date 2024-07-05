//-- ./src/middleware/authentication.rs

#![allow(unused)] // For development only

//! Authenticate a request
//! 
//! ---

use tonic::{metadata::MetadataValue, Request, Status};

pub fn check_authentication(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "Bearer some-auth-token".parse().unwrap();

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}