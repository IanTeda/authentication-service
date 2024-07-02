//-- ./src/database/users/mod.rs

//! Wrapper around users database tables
 
#![allow(unused)] // For development only


mod model;
mod create;
mod read;
mod update;
mod delete;

pub use model::UserModel;
pub use create::insert_user;
pub use read::{select_user_by_email, select_user_by_id, select_user_index};
pub use update::update_user_by_id;
pub use delete::delete_user_by_id;