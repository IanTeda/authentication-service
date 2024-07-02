//-- ./src/database/users/mod.rs

//! Wrapper around users database tables

mod model;
mod insert;
mod read;

pub use model::UserModel;
pub use insert::insert_user;