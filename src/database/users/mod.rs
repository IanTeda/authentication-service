//-- ./src/database/users/mod.rs

//! Wrapper around users database tables

mod model;
mod insert;

pub use model::UserModel;
pub use insert::insert_user;