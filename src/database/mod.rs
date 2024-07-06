//-- ./src/database/mod.rs

//! Wrapper around database tables

use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{configuration::DatabaseSettings, prelude::*};

pub mod users;

pub async fn init_pool(database_configuration: &DatabaseSettings) -> Result<PgPool, BackendError> {
	// Build connection pool
	let database = PgPoolOptions::new().connect_lazy_with(database_configuration.connection());

	sqlx::migrate!("./migrations").run(&database).await?;

	Ok(database)
}
