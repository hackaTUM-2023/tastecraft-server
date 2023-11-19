use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use axum::{Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse};
use axum::{Router, extract::State};
use axum::routing::get;
use sqlx::{Postgres, Pool, PgPool};
use tower_http::services::ServeDir;
use serde::Deserialize;
use axum::extract::Query;

use crate::config::Config;
use crate::services;


#[derive(Clone)]
pub struct ApiContext {
    config: Arc<Config>,
    db: Pool<Postgres>,
}

pub fn api_router() -> Router<ApiContext> {
    let serve_dir = ServeDir::new("assets/");

    Router::new()
    .route("/version", get(get_version))
    .route("/recipes", get(get_recipes))
    .route("/recipes/:id", get(get_recipe_by_id))
    .nest_service("/assets", serve_dir)
}

pub async fn serve(config: Config, db: PgPool) -> anyhow::Result<()> {
    let app = api_router()
        .with_state(ApiContext {
            config: Arc::new(config),
            db,
        });

    axum::Server::bind(&"0.0.0.0:8080".parse()?)
        .serve(app.into_make_service())
        .await
        .context("error running HTTP server")
}

async fn get_version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    version.to_string()
}

async fn get_recipes(axum::extract::Query(params):
axum::extract::Query<HashMap<String, String>>, State(state): State<ApiContext>) -> impl IntoResponse {
    println!("Get items with query params: {:?}", params);
    let search_text = params.get("searchText").map(|s| s.as_str());
    let res = services::recipes::get_original_recipes(&state.db, search_text).await;
    if let Ok(recipes) = res {
        return (StatusCode::OK, Json(recipes)).into_response()
    }
   (StatusCode::INTERNAL_SERVER_ERROR, res.unwrap_err().to_string()).into_response()
}

#[derive(Deserialize, Debug)]
pub struct PrefParam {
    pub preferences: Vec<String>
}

async fn get_recipe_by_id(axum::extract::Path(id):
axum::extract::Path<i32>, Query(params): Query<PrefParam>, State(state): State<ApiContext>) -> impl IntoResponse {
    let res = services::recipes::get_recipe_by_id(&state.db, id, params.preferences.as_slice()).await;
    if let Ok(recipe) = res {
        return (StatusCode::OK, Json(recipe)).into_response()
    }
   (StatusCode::INTERNAL_SERVER_ERROR, res.unwrap_err().to_string()).into_response()
}