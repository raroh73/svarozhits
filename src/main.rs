use axum::{
    routing::{delete, get, patch, post},
    Router, Server,
};
use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, SqlitePool};
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

    let mut db_connect_options = SqliteConnectOptions::new()
        .filename("database.db")
        .create_if_missing(true);
    db_connect_options.log_statements(tracing::log::LevelFilter::Debug);
    let db_pool = SqlitePool::connect_with(db_connect_options).await?;
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    let svarozhits_port = env::var("SVAROZHITS_PORT").unwrap_or_else(|_| 8008.to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], svarozhits_port.parse::<u16>()?));

    info!("listening on {}", addr);

    Server::bind(&addr)
        .serve(app(&db_pool).into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    db_pool.close().await;

    Ok(())
}

fn app(db_pool: &SqlitePool) -> Router {
    Router::new()
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
        .with_state(db_pool.clone())
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
