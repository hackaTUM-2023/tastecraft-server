use clap::Parser;
use sqlx::{PgPool, Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use crate::config::Config;

pub async fn init_db() -> Result<Pool<Postgres>, sqlx::Error> {
    let config = Config::parse();
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url).await?;

    Ok(pool)
}

async fn test_connection(db: &PgPool) -> Result<(), sqlx::Error> {
    let results = sqlx::query!("SELECT 1+1 as sum").fetch_all(db).await?;

    Ok(())
}