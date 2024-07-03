//-- ./src/database/mod.rs

//! Wrapper around database tables

use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{configuration::DatabaseSettings, prelude::*};

pub mod users;

pub async fn get_connection_pool(database: &DatabaseSettings) -> Result<PgPool, BackendError> {
	// Build connection pool
	let database = PgPoolOptions::new().connect_lazy_with(database.connection());

	sqlx::migrate!("./migrations").run(&database).await?;

	Ok(database)
}
