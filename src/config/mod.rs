pub mod loader;
pub mod types;
pub mod validator;

pub use loader::*;
pub use types::*;
pub use validator::*;

use crate::utils::constants::*;

pub fn load_config() -> Result<AgentConfig, anyhow::Error> {
    let config_path = std::env::var(AGENT_CONFIG_ENV)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::PathBuf::from("config")
                .join("agent.yml")
        });
    
    tracing::info!("Loading config from: {:?}", config_path);
    
    if config_path.exists() {
        AgentConfig::load_from_file(&config_path)
    } else {
        tracing::warn!("Config file not found, using defaults");
        Ok(AgentConfig::default())
    }
}

pub fn init_logging() -> Result<(), anyhow::Error> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
    
    let log_level = std::env::var(AGENT_LOG_LEVEL_ENV)
        .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    Ok(())
}
