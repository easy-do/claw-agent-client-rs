pub mod loader;
pub mod types;
pub mod validator;

pub use types::*;
pub use validator::*;

pub fn load_config() -> Result<AgentConfig, anyhow::Error> {
    let config_path = std::path::PathBuf::from("config").join("agent.yml");
    
    tracing::info!("Loading config from: {:?}", config_path);
    
    if config_path.exists() {
        let config = AgentConfig::load_from_file(&config_path)?;
        if let Some(parent) = config_path.parent() {
            let example_path = parent.join("agent.example.yml");
            if !example_path.exists() {
                tracing::info!("Generating example config file: {:?}", example_path);
                let example_config = AgentConfig::default();
                example_config.save_to_file(&example_path)?;
            }
        }
        Ok(config)
    } else {
        tracing::warn!("Config file not found, generating default config");
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let default_config = AgentConfig::default();
        default_config.save_to_file(&config_path)?;
        
        let example_path = config_path.parent()
            .map(|p| p.join("agent.example.yml"))
            .unwrap_or_else(|| std::path::PathBuf::from("agent.example.yml"));
        
        tracing::info!("Generating example config file: {:?}", example_path);
        default_config.save_to_file(&example_path)?;
        
        Ok(default_config)
    }
}

pub fn init_logging() -> Result<(), anyhow::Error> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
    
    let log_level = "info";
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    Ok(())
}
