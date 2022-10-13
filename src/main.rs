use axum::{routing::get, Router, Server};
use std::{error::Error, net::SocketAddr};
use tracing::info;

async fn root() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/", get(root));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("Listening on {}", addr);

    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}
