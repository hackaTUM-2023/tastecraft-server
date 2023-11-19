mod db;
mod config;
mod services;
mod models;
mod api;

use std::error::Error;
use clap::Parser;
use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // build our application with a single route

    dotenv::dotenv().ok();

    env_logger::init();
    let config = Config::parse();

    let db = db::init_db().await?;

    sqlx::migrate!().run(&db).await?;
    println!("Migrated database");

    println!("Starting server on port 8080");
    api::serve(config, db).await?;

    Ok(())
}
