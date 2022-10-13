use axum::{routing::get, Router, Server};
use std::{error::Error, net::SocketAddr};
use tokio::signal;
use tracing::{error, info};

async fn root() -> &'static str {
    "Hello, world!"
}

async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("Could not register ctrl+c handler!");

    info!("Shutting down!");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/", get(root));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    info!("Listening on {}", addr);

    let server = Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal());

    if let Err(err) = server.await {
        error!("Client error: {}", err)
    }

    Ok(())
}
