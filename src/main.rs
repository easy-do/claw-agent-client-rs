use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use cross_platform_agent_rs::{init_logging, load_config, run_server};

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use cross_platform_agent_rs::platform;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;
    
    let config = load_config()?;
    
    let token = config.auth.token.clone().unwrap();
    let agent_id = config.agent_id.clone();
    
    let config = Arc::new(config);
    
    tracing::info!("Starting Cross-Platform Agent...");

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    tracing::info!("Platform: {}", platform::platform_name());

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    tracing::info!("Platform: unknown");

    tracing::info!("Agent ID: {}", agent_id);
    tracing::info!("Token: {}", token);

    let server = run_server(config.clone());
    
    let shutdown = async {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        tracing::info!("Shutting down...");
    };
    
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        _ = shutdown => {}
    }
    
    tracing::info!("Agent stopped.");
    Ok(())
}
