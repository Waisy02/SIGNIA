use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Result;
use tracing::info;

mod app;
mod config;
mod dto;
mod error;
mod middleware;
mod routes;
mod state;
mod telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    let args = config::Args::parse();
    let cfg = config::load_config(args.config.as_deref())?;

    telemetry::init(&cfg.telemetry)?;

    let store_cfg = signia_store::StoreConfig::local_dev(PathBuf::from(&cfg.store_root))?;
    let store = signia_store::Store::open(store_cfg)?;

    let app_state = state::AppState::new(cfg.clone(), store)?;

    let router = app::build_router(app_state);

    let addr: SocketAddr = cfg.listen_addr.parse()?;
    info!(%addr, "starting signia-api");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
