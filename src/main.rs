mod db;
mod config;
mod services;
mod models;

use std::error::Error;
use clap::Parser;
use crate::config::Config;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = api::init_routing();

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

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

}
