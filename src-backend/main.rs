mod api;
mod config;
mod error;
mod middleware;
mod model;
mod task;
mod utils;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use axum_response_cache::CacheLayer;
use bollard::Docker;
use config::Config;
use mimalloc::MiMalloc;
use sea_orm::Database;
use tokio::{net::TcpListener, signal};
use tower_http::{compression::CompressionLayer, services::ServeDir, trace::TraceLayer};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    // Register tracer
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,sqlx=warn,bollard=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let cfg = Config::get_instance();
    debug!("configs: {:?}", cfg);

    // Initialize database
    let db = Database::connect(&cfg.db_url).await?;
    db.ping().await?;

    utils::init_table(&db, model::heartbeat::Entity).await?;
    utils::init_table(&db, model::status::Entity).await?;

    // Initialize docker client
    let docker = Docker::connect_with_local_defaults()?;
    docker.ping().await?;

    // Initialize scheduled tasks
    tokio::spawn(task::clean_database(&db));
    tokio::spawn(task::collect_status(&db, &docker));

    // Initialize routers
    let api_router = Router::new()
        .route("/heartbeat/{token}", post(api::heartbeat))
        .route(
            "/status",
            get(api::status).layer(CacheLayer::with_lifespan(300).add_response_headers()),
        )
        .with_state(db);
    let serve_dir = ServeDir::new("www");
    let app = Router::new()
        .nest("/api", api_router)
        .fallback_service(serve_dir)
        .layer(axum::middleware::from_fn(middleware::auth))
        .layer(TraceLayer::new_for_http())
        .layer(
            CompressionLayer::new()
                .gzip(true)
                .deflate(true)
                .br(true)
                .zstd(true),
        );

    // Start listening
    let listener = TcpListener::bind((cfg.bind_addr.as_str(), cfg.bind_port)).await?;
    info!("start listening on `{}`", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Success
    Ok(())
}

async fn shutdown_signal() {
    // Ctrl C handler
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // Terminate hanler
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wait signals
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
