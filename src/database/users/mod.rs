//-- ./src/database/users/mod.rs

//! Wrapper around users database tables

// #![allow(unused)] // For development only

pub use model::Users;

mod delete;
mod insert;
mod model;
mod read;
mod update;
