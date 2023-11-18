use std::error::Error;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;

#[derive(Parser, Debug)]
pub struct Config {
    /// The connection URL for the Postgres database this application should use.
    #[clap(long, env)]
    pub database_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=migrations");

    dotenv::dotenv().ok();

    let config = Config::parse();

    let db = PgPoolOptions::new()
        .max_connections(1)
        .connect(&config.database_url)
        .await
        .unwrap();

    sqlx::migrate!().run(&db).await?;

    Ok(())
}