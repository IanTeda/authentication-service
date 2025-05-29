//-- ./src/database/sessions/mod.rs

//! Sessions database module for the authentication service.
//!
//! This module provides all session-related database operations, including creation, reading, updating, and deletion (CRUD).
//! It organises logic into submodules for each operation and re-exports the main `Sessions` model for convenient use elsewhere.
//!
//! # Contents
//! - Session deletion logic
//! - Session insertion/creation logic
//! - Session struct definition and model-level helpers
//! - Session read/query logic
//! - Session update and revoke logic

// #![allow(unused)] // For development only

pub use model::Sessions;

mod delete;
mod insert;
mod model;
mod read;
mod update;
