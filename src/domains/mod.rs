//-- ./src/domain/mod.rs

//! A collection of new data type domains
//! ---

mod email_address;
mod user_name;
mod password;

// Re-export domain structs
pub use email_address::EmailAddress;
pub use user_name::UserName;
pub use password::Password;