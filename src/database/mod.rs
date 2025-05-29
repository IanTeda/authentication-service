//-- ./src/database/mod.rs

//! Database module for the authentication service.
//!
//! This module provides initialisation and access to the application's database tables,
//! including connection pool setup, migrations, and re-exports of user and session models.
//!
//! # Contents
//! - Connection pool initialisation and migration runner
//! - Import user and session database models and logic
//! - Re-exports modules for convenient access in other parts of the application

// #![allow(unused)] // For development only

use sqlx::{postgres::PgPoolOptions, PgPool};
use crate::{configuration::DatabaseConfiguration, prelude::*};

// Module imports
mod sessions;
mod users;

// Reexport modules for cleaner code
pub use sessions::Sessions;
pub use users::Users;

/// Initialize the PostgreSQL connection pool and run database migrations.
///
/// # Parameters
/// * `database_configuration` - Reference to the application's database configuration.
///
/// # Returns
/// * `Ok(PgPool)` - The initialize PostgreSQL connection pool if successful.
/// * `Err(AuthenticationError)` - If the connection or migration fails.
///
/// # Behaviors
/// - Builds a lazy connection pool using the provided configuration.
/// - Runs all pending SQLx migrations from the `./migrations` directory before returning the pool.
/// - Returns an error if the connection or migration fails.
pub async fn init_pool(
    database_configuration: &DatabaseConfiguration,
) -> Result<PgPool, AuthenticationError> {
    // Build connection pool
    let database =
        PgPoolOptions::new().connect_lazy_with(database_configuration.connection());

    // Migrate database
    sqlx::migrate!("./migrations").run(&database).await?;

    // Return database
    Ok(database)
}
