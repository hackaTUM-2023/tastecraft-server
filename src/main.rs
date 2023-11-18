mod db;
mod config;

use std::error::Error;
use clap::Parser;
use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    dotenv::dotenv().ok();

    env_logger::init();
    let _config = Config::parse();

    let db = db::init_db().await?;

    sqlx::migrate!().run(&db).await?;

    Ok(())
}
