//-- ./src/database/users/mod.rs

//! Wrapper around users database tables

// #![allow(unused)] // For development only

pub use model::Users;

// User deletion logic and related database functions
mod delete;

// User insertion/creation logic and related database functions
mod insert;

// User struct definition and model-level helpers
mod model;

// User read/query logic and related database functions
mod read;

// User update logic and related database functions
mod update;
