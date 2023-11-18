use axum::Router;
use axum::routing::get;


#[derive(Clone)]
pub struct ApiContext {
}

pub fn init_routing() -> Router {
    Router::new()
        .route("/version", get(get_version))
}

async fn get_version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    version.to_string()
}