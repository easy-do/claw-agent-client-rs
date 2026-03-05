pub mod client;

pub use client::*;

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use crate::config::AgentConfig;

pub async fn run_server(config: Arc<AgentConfig>) -> Result<(), anyhow::Error> {
    let server_url = config.server_url.clone()
        .ok_or_else(|| anyhow::anyhow!("server_url is required"))?;
    
    let ws_url = format!("{}/agent/ws", server_url);
    
    loop {
        tracing::info!("Connecting to OpenClaw server: {}", ws_url);
        
        match client::connect(&ws_url, config.clone()).await {
            Ok(_) => {
                tracing::info!("Connection closed normally");
                break;
            }
            Err(e) => {
                tracing::error!("Connection error: {}", e);
                tracing::info!("Reconnecting in 2 seconds...");
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
    
    Ok(())
}
