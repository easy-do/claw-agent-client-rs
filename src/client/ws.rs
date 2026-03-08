use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use crate::config::AgentConfig;
use crate::client::CommandContext;

fn log_command_to_file(action: &str, params: &serde_json::Value) {
    let log_path = std::path::Path::new("log/cmd.log");
    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let params_str = params.to_string();
    
    let log_line = format!("[{}] {} | {}\n", timestamp, action, params_str);
    
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let _ = file.write_all(log_line.as_bytes());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthRequest {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(rename = "agent_id")]
    agent_id: String,
    token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthResponse {
    #[serde(rename = "type")]
    msg_type: String,
    success: Option<bool>,
    session_id: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandRequest {
    #[serde(rename = "command_id")]
    command_id: String,
    action: String,
    params: serde_json::Value,
}

enum InternalMessage {
    Pong(Vec<u8>),
}

enum OutgoingMessage {
    Command(CommandRequest),
    PingPong(InternalMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandResponse {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(rename = "command_id")]
    command_id: String,
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WelcomeMessage {
    #[serde(rename = "type")]
    msg_type: String,
    version: Option<String>,
}

pub async fn connect(url: &str, config: Arc<AgentConfig>) -> Result<(), anyhow::Error> {
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    tracing::info!("Connected to server, waiting for welcome message...");

    if let Some(msg) = read.next().await {
        let msg = msg?;
        let text = msg.to_text()?;
        
        let welcome: WelcomeMessage = serde_json::from_str(text)
            .map_err(|e| anyhow::anyhow!("Failed to parse welcome: {}", e))?;
        
        tracing::info!("Received welcome message, version: {:?}", welcome.version);
    }

    let auth = AuthRequest {
        msg_type: "auth".to_string(),
        agent_id: config.agent_id.clone(),
        token: config.auth.token.clone()
            .ok_or_else(|| anyhow::anyhow!("token is required"))?,
    };

    let auth_json = serde_json::to_string(&auth)?;
    write.send(Message::Text(auth_json)).await?;

    if let Some(msg) = read.next().await {
        let msg = msg?;
        let text = msg.to_text()?;
        
        let auth_resp: AuthResponse = serde_json::from_str(text)
            .map_err(|e| anyhow::anyhow!("Failed to parse auth response: {}", e))?;
        
        if auth_resp.success != Some(true) {
            return Err(anyhow::anyhow!("Authentication failed: {:?}", auth_resp.message));
        }
        
        tracing::info!("Authenticated successfully, session: {:?}", auth_resp.session_id);
    }

    let (cmd_tx, mut cmd_rx) = mpsc::channel::<OutgoingMessage>(100);
    let write_arc = Arc::new(Mutex::new(write));
    
    let cmd_ctx = CommandContext::new(config.clone());
    
    let write_for_sender = write_arc.clone();
    tokio::spawn(async move {
        while let Some(msg) = cmd_rx.recv().await {
            let mut write = write_for_sender.lock().await;
            
            match msg {
                OutgoingMessage::Command(cmd) => {
                    let result = execute_command(&cmd_ctx, cmd.action.as_str(), cmd.params).await;
                    
                    let response = match result {
                        Ok(data) => CommandResponse {
                            msg_type: "command_response".to_string(),
                            command_id: cmd.command_id,
                            success: true,
                            data: Some(data),
                            error: None,
                        },
                        Err(e) => CommandResponse {
                            msg_type: "command_response".to_string(),
                            command_id: cmd.command_id,
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                        },
                    };
                    
                    if let Ok(json) = serde_json::to_string(&response) {
                        let _ = write.send(Message::Text(json)).await;
                    }
                }
                OutgoingMessage::PingPong(InternalMessage::Pong(data)) => {
                    let _ = write.send(Message::Pong(data)).await;
                }
            }
        }
    });

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        tracing::info!("Received Text");
                        if let Ok(cmd) = serde_json::from_str::<CommandRequest>(&text) {
                            tracing::info!("Received command: {} {}", cmd.command_id, cmd.action);
                            log_command_to_file(&cmd.action, &cmd.params);
                            let _ = cmd_tx.send(OutgoingMessage::Command(cmd)).await;

                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = cmd_tx.send(OutgoingMessage::PingPong(InternalMessage::Pong(data))).await;
                    }
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!("Server closed connection");
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        tracing::warn!("Received None");
                    }
                    _ => {
                        tracing::warn!("Unexpected message");
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!("Connection lost"))
}

async fn execute_command(ctx: &CommandContext, action: &str, params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    ctx.execute(action, params).await
}
