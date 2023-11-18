

mod api;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = api::init_routing();

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
