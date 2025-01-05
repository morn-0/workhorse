use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use crate::prelude::*;

pub mod entity;

pub async fn connect() -> HorseResult<DatabaseConnection> {
    let mut opt = ConnectOptions::new("sqlite://horsed.db3?mode=rwc");
    opt.connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .set_schema_search_path("my_schema"); // Setting default PostgreSQL schema

    let db = Database::connect(opt).await?;

    Ok(db)
}
