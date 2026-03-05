use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use crate::config::AgentConfig;

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
    
    let write_for_sender = write_arc.clone();
    tokio::spawn(async move {
        while let Some(msg) = cmd_rx.recv().await {
            let mut write = write_for_sender.lock().await;
            
            match msg {
                OutgoingMessage::Command(cmd) => {
                    let result = execute_command(cmd.action.as_str(), cmd.params).await;
                    
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
                        // tracing::info!("Received Ping, queuing Pong response");
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

async fn execute_command(action: &str, params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let result = match action {
        "system.info" => {
            let platform = crate::platform::get_platform();
            let info = platform.get_system_info().await?;
            let json = serde_json::json!({
                "hostname": info.hostname,
                "os_type": info.os_type,
                "os_version": info.os_version,
                "arch": info.arch,
                "username": info.username,
                "uptime_secs": info.uptime_secs,
                "total_memory_gb": info.total_memory_gb,
                "available_memory_gb": info.available_memory_gb,
                "cpu_count": info.cpu_count,
                "cpu_usage_percent": info.cpu_usage_percent,
            });
            json
        }
        "process.list" => {
            let platform = crate::platform::get_platform();
            let processes = platform.list_processes().await?;
            let list: Vec<serde_json::Value> = processes.into_iter().map(|p| {
                serde_json::json!({
                    "pid": p.pid,
                    "name": p.name,
                    "cmd": p.cmd,
                    "cpu_percent": p.cpu_percent,
                    "memory_mb": p.memory_mb,
                    "status": p.status
                })
            }).collect();
            serde_json::to_value(list)?
        }
        "software.list" => {
            let platform = crate::platform::get_platform();
            let software = platform.list_software().await?;
            let list: Vec<serde_json::Value> = software.into_iter().map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "version": s.version,
                    "publisher": s.publisher,
                    "install_path": s.install_path
                })
            }).collect();
            serde_json::to_value(list)?
        }
        "env.list" => {
            let scope = params.get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("user");
            let scope = match scope {
                "system" => crate::platform::types::EnvScope::System,
                "session" => crate::platform::types::EnvScope::Session,
                _ => crate::platform::types::EnvScope::User,
            };
            let platform = crate::platform::get_platform();
            let vars = platform.list_env_vars(scope).await?;
            let list: Vec<serde_json::Value> = vars.into_iter().map(|(k, v)| {
                serde_json::json!({
                    "name": k,
                    "value": v
                })
            }).collect();
            serde_json::to_value(list)?
        }
        "file.list" => {
            let path = params.get("path")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            let platform = crate::platform::get_platform();
            let files = platform.list_dir(path).await?;
            let list: Vec<serde_json::Value> = files.into_iter().map(|f| {
                serde_json::json!({
                    "name": f.name,
                    "path": f.path,
                    "is_dir": f.is_dir,
                    "size_bytes": f.size_bytes,
                    "modified": f.modified
                })
            }).collect();
            serde_json::to_value(list)?
        }
        "shell.execute" => {
            let command = params.get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing command parameter"))?;

            let timeout_secs = params.get("timeout")
                .and_then(|v| v.as_u64())
                .unwrap_or(30);

            let platform = crate::platform::get_platform();
            let result = platform.shell_execute(command, timeout_secs).await?;
            result
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown action: {}", action));
        }
    };
    
    Ok(result)
}
