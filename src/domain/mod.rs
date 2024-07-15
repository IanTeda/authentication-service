//-- ./src/domain/mod.rs

//! A collection of new data type domains
//! ---

#![allow(unused)] // For beginning only.

mod access_token;
mod email_address;
mod password_hash;
mod refresh_token;
mod token_claim;
mod user_name;
mod user_role;

// Re-export domain structs
pub use access_token::AccessToken;
pub use email_address::EmailAddress;
pub use password_hash::PasswordHash;
pub use refresh_token::RefreshToken;
pub use token_claim::{TokenClaim, TOKEN_ISSUER};
pub use user_name::UserName;
pub use user_role::UserRole;
