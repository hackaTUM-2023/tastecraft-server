mod db;
mod config;
mod services;
mod models;

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

    let recipe2 = services::recipes::create_motified_recipe(&db, 2, vec!["chicken"]).await?;
    println!("{:?}", recipe2);
    let recipe3 = services::recipes::create_motified_recipe(&db, 3, vec!["beef"]).await?;
    println!("{:?}", recipe3);

    let recipes = services::recipes::get_original_recipes(&db, "title").await?;
    println!("{:?}", recipes);

    Ok(())
}
