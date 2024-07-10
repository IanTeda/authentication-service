//-- ./src/database/users/mod.rs

//! Wrapper around users database tables

// #![allow(unused)] // For development only

mod delete;
mod insert;
mod model;
mod read;
mod update;

pub use model::UserModel;
