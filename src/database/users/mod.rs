//-- ./src/database/users/mod.rs

//! Wrapper around users database tables

// #![allow(unused)] // For development only

mod create;
mod delete;
mod model;
mod read;
mod update;

pub use create::insert_user;
pub use delete::delete_user_by_id;
pub use model::UserModel;
pub use read::{ select_user_by_id, select_user_index, select_user_by_email };
pub use update::{ update_user_by_id, update_password_by_id };
