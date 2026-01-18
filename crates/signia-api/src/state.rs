use std::sync::Arc;

use anyhow::Result;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<AppConfig>,
    pub store: Arc<signia_store::Store>,
    pub plugins: Arc<signia_plugins::registry::PluginRegistry>,
}

impl AppState {
    pub fn new(cfg: AppConfig, store: signia_store::Store) -> Result<Self> {
        let mut reg = signia_plugins::registry::PluginRegistry::default();

        // Builtins
        signia_plugins::builtin::repo::register(&mut reg);
        signia_plugins::builtin::dataset::register(&mut reg);
        signia_plugins::builtin::workflow::register(&mut reg);
        signia_plugins::builtin::api::register(&mut reg);
        signia_plugins::builtin::spec::register(&mut reg);

        Ok(Self {
            cfg: Arc::new(cfg),
            store: Arc::new(store),
            plugins: Arc::new(reg),
        })
    }
}
