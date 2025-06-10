//-- ./src/domain/mod.rs

#![allow(unused)] // For beginning only.

//! # Domain Types Module
//!
//! This module defines and re-exports new type wrappers and domain-specific types
//! used throughout the authentication service. These types provide additional
//! type safety, validation, and encapsulation for core concepts such as tokens,
//! email addresses, user roles, and more.
//!
//! ## Domains included:
//! - AccessToken
//! - EmailAddress
//! - TokenClaim (JWT)
//! - PasswordHash
//! - RefreshToken
//! - RowID
//! - UserName
//! - UserRole
//!
//! Use these types in place of primitive types to enforce invariants and improve code clarity.

mod access_token;
mod email_address;
mod jwt_token;
mod password_hash;
mod refresh_token;
mod row_id;
mod user_name;
mod user_role;
mod tokens;

// Re-export domain structs
pub use access_token::AccessToken;
pub use email_address::EmailAddress;
pub use jwt_token::TokenClaim;
pub use password_hash::PasswordHash;
pub use refresh_token::RefreshToken;
pub use row_id::RowID;
pub use user_name::UserName;
pub use user_role::UserRole;
