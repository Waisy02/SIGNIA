use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

use crate::config::TelemetryConfig;

pub fn init(cfg: &TelemetryConfig) -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if cfg.json {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer())
            .init();
    }
    Ok(())
}
