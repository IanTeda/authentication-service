//-- ./src/domain/mod.rs

//! A collection of new data type domains
//! ---

mod email_address;
mod user_name;

pub use email_address::EmailAddress;
pub use user_name::UserName;