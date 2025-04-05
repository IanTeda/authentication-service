// -- ./src/prelude.rs

//! Bring default crates into scope.
//!
//! These are the most common items used by the authentication/server code in
//! intended to be imported by all server code, for convenience.

// Re-export the crate Error.
// #[allow(unused_imports)]
pub use crate::error::AuthenticationError;

// Alias Result to be the crate Result.
// pub type Result<T> = core::result::Result<T, BackendError>;

// pub type TonicResult<T> = core::result::Result<T, tonic::Status>;
