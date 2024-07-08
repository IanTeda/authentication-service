//-- ./src/domain/mod.rs

//! A collection of new data type domains
//! ---
 
#![allow(unused)] // For beginning only.

mod email_address;
mod user_name;
mod password;
mod token_claim;
mod access_token;
mod refresh_token;

// Re-export domain structs
pub use email_address::EmailAddress;
pub use user_name::UserName;
pub use password::{Password, verify_password_hash};
pub use token_claim::{TokenClaim, TOKEN_ISSUER};
pub use access_token::AccessToken;
pub use refresh_token::RefreshToken;