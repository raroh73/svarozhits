use axum::{
    routing::{delete, get, patch, post},
    Router, Server,
};
use sqlx::SqlitePool;
use std::{env, error::Error, net::SocketAddr};
use tokio::signal::unix::{signal, SignalKind};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::{debug, info};

pub mod errors;
pub mod models;
pub mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let svarozhits_port = env::var("SVAROZHITS_PORT").unwrap_or_else(|_| 8008.to_string());

    let db_pool = SqlitePool::connect("sqlite://database.db?mode=rwc").await?;
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    let app = Router::new()
        .route("/", get(routes::show_index))
        .route("/tasks", post(routes::add_task))
        .route("/tasks/:task_id", patch(routes::mark_task_as_done))
        .route("/tasks/:task_id", delete(routes::delete_task))
        .fallback(routes::fallback)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new()),
        )
        .with_state(db_pool.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], svarozhits_port.parse::<u16>()?));

    info!("listening on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    db_pool.close().await;

    Ok(())
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
