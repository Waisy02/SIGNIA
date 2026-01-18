use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub listen_addr: String,
    pub log_level: String,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub cors: CorsConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    pub store_root: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8080".to_string(),
            log_level: "info".to_string(),
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            cors: CorsConfig::default(),
            telemetry: TelemetryConfig::default(),
            store_root: ".signia".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// "disabled" | "optional" | "required"
    #[serde(default = "AuthConfig::default_mode")]
    pub mode: String,
    #[serde(default)]
    pub bearer_tokens: Vec<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self { mode: Self::default_mode(), bearer_tokens: vec![] }
    }
}

impl AuthConfig {
    fn default_mode() -> String {
        "optional".to_string()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "RateLimitConfig::default_rpm")]
    pub rpm: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self { enabled: true, rpm: Self::default_rpm() }
    }
}

impl RateLimitConfig {
    fn default_rpm() -> u32 {
        600
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    #[serde(default)]
    pub allow_any_origin: bool,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self { allow_any_origin: true, allowed_origins: vec![] }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TelemetryConfig {
    #[serde(default = "TelemetryConfig::default_format")]
    pub format: String,
    #[serde(default)]
    pub json: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self { format: Self::default_format(), json: false }
    }
}

impl TelemetryConfig {
    fn default_format() -> String {
        "pretty".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Args {
    pub config: Option<String>,
}

impl Args {
    pub fn parse() -> Self {
        let mut config: Option<String> = None;
        let mut it = std::env::args().skip(1);
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "--config" => {
                    if let Some(v) = it.next() {
                        config = Some(v);
                    }
                }
                _ => {}
            }
        }
        Self { config }
    }
}

pub fn load_config(path: Option<&str>) -> Result<AppConfig> {
    match path {
        None => Ok(AppConfig::default()),
        Some(p) => {
            let raw = fs::read_to_string(Path::new(p))?;
            let mut cfg: AppConfig = serde_json::from_str(&raw)
                .map_err(|e| anyhow!("invalid config json: {e}"))?;
            if cfg.listen_addr.trim().is_empty() {
                cfg.listen_addr = AppConfig::default().listen_addr;
            }
            if cfg.log_level.trim().is_empty() {
                cfg.log_level = AppConfig::default().log_level;
            }
            Ok(cfg)
        }
    }
}
