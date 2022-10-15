use axum::{routing::get, Router, Server};
use std::{error::Error, net::SocketAddr};
use tokio::signal::unix::{signal, SignalKind};
use tracing::{info, trace};

async fn root() -> &'static str {
    "Hello, world!"
}

async fn shutdown_signal() {
    let mut sigint = signal(SignalKind::interrupt()).expect("could not create SIGINT stream");
    let mut sigterm = signal(SignalKind::terminate()).expect("could not create SIGTERM stream");

    tokio::select! {
        _ = sigint.recv() => trace!("received SIGINT"),
        _ = sigterm.recv() => trace!("received SIGTERM"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/", get(root));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    info!("listening on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
