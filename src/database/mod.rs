//-- ./src/database/mod.rs

//! Wrapper around database tables

#![allow(unused)] // For development only

use sqlx::{postgres::PgPoolOptions, PgPool};

// Reexport for cleaner code
pub use sessions::Sessions;
pub use users::Users;
pub use logins::Logins;

use crate::{configuration::DatabaseConfiguration, prelude::*};

mod logins;
mod sessions;
mod users;

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
