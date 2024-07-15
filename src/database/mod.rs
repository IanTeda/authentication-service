//-- ./src/database/mod.rs

//! Wrapper around database tables

// #![allow(unused)] // For development only

use sqlx::{postgres::PgPoolOptions, PgPool};

pub use refresh_tokens::RefreshTokens;
pub use users::Users;

use crate::{configuration::DatabaseConfiguration, prelude::*};

mod users;

mod refresh_tokens;

pub async fn init_pool(
    database_configuration: &DatabaseConfiguration,
) -> Result<PgPool, BackendError> {
    // Build connection pool
    let database =
        PgPoolOptions::new().connect_lazy_with(database_configuration.connection());

    // Migrate database
    sqlx::migrate!("./migrations").run(&database).await?;

    // Return database
    Ok(database)
}
