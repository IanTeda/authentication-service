//-- ./src/database/logins/mod.rs

#![allow(unused)] // For development only

//! Database service for the Logins table

/// Reexport database model
pub use model::Logins;

mod model;
mod insert;
mod read;
mod update;
mod delete;