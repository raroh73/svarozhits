use axum::{routing::get, Router, Server};
use std::{error::Error, net::SocketAddr};
use tokio::signal::unix::{signal, SignalKind};
use tower_http::trace::TraceLayer;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .route("/", get(root))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    info!("listening on {}", addr);

    Server::bind(&addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, world!"
}

async fn shutdown_signal() {
    let sigint = async {
        signal(SignalKind::interrupt())
            .expect("failed to install SIGINT handler")
            .recv()
            .await;
    };
    let sigterm = async {
        signal(SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = sigint => debug!("received SIGINT"),
        _ = sigterm => debug!("received SIGTERM"),
    }

    info!("shutting down");
}
