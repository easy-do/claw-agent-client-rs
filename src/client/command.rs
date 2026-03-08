use crate::config::{AgentConfig, Capabilities};
use crate::platform::types::*;
use std::sync::Arc;

pub struct CommandContext {
    pub config: Arc<AgentConfig>,
}

impl CommandContext {
    pub fn new(config: Arc<AgentConfig>) -> Self {
        Self { config }
    }

    pub fn get_capabilities(&self) -> &Capabilities {
        &self.config.capabilities
    }

    pub fn is_command_enabled(&self, command_id: &str) -> bool {
        self.config.capabilities.is_enabled(command_id)
    }

    pub fn execute_capabilities(&self) -> serde_json::Value {
        self.config.capabilities.to_capabilities_list()
    }

    pub async fn execute(&self, command: &str, params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        if !self.is_command_enabled(command) {
            return Err(anyhow::anyhow!(
                "Command '{}' is disabled in configuration",
                command
            ));
        }

        match command {
            "capabilities" => Ok(self.execute_capabilities()),
            "system.info" => execute_system_info().await,
            "process.list" => execute_process_list().await,
            "software.list" => execute_software_list().await,
            "env.list" => execute_env_list(params).await,
            "file.list" => execute_file_list(params).await,
            "shell.execute" => execute_shell(params).await,
            "software.search" => execute_search_software(params).await,
            "software.install" => execute_install_software(params).await,
            "software.uninstall" => execute_uninstall_software(params).await,
            "env.get" => execute_get_env_var(params).await,
            "env.set" => execute_set_env_var(params).await,
            "env.delete" => execute_delete_env_var(params).await,
            "file.read" => execute_read_file(params).await,
            "file.write" => execute_write_file(params).await,
            "file.delete" => execute_delete_file(params).await,
            "file.create_dir" => execute_create_dir(params).await,
            "file.copy" => execute_copy_file(params).await,
            "file.move" => execute_move_file(params).await,
            "file.download" => execute_download_file(params).await,
            "config.get" => execute_get_config(params).await,
            "config.set" => execute_set_config(params).await,
            "system.reboot" => execute_reboot().await,
            "system.shutdown" => execute_shutdown().await,
            "process.stop" => execute_stop_process(params).await,
            _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
        }
    }
}

async fn execute_system_info() -> Result<serde_json::Value, anyhow::Error> {
    let platform = crate::platform::get_platform();
    let info = platform.get_system_info().await?;
    Ok(serde_json::json!({
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
    }))
}

async fn execute_process_list() -> Result<serde_json::Value, anyhow::Error> {
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
    Ok(serde_json::to_value(list)?)
}

async fn execute_software_list() -> Result<serde_json::Value, anyhow::Error> {
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
    Ok(serde_json::to_value(list)?)
}

async fn execute_env_list(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let scope = params.get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("user");
    let scope = match scope {
        "system" => EnvScope::System,
        "session" => EnvScope::Session,
        _ => EnvScope::User,
    };

    let platform = crate::platform::get_platform();
    let vars = platform.list_env_vars(scope).await?;
    let list: Vec<serde_json::Value> = vars.into_iter().map(|(k, v)| {
        serde_json::json!({
            "name": k,
            "value": v
        })
    }).collect();
    Ok(serde_json::to_value(list)?)
}

async fn execute_file_list(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
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
    Ok(serde_json::to_value(list)?)
}

async fn execute_shell(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let command = params.get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing command parameter"))?;

    let timeout_secs = params.get("timeout")
        .and_then(|v| v.as_u64())
        .unwrap_or(30);

    let platform = crate::platform::get_platform();
    platform.shell_execute(command, timeout_secs)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}

async fn execute_search_software(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let query = params.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;

    let platform = crate::platform::get_platform();
    let software = platform.search_software(query).await?;
    let list: Vec<serde_json::Value> = software.into_iter().map(|s| {
        serde_json::json!({
            "name": s.name,
            "version": s.version,
            "publisher": s.publisher,
            "install_path": s.install_path
        })
    }).collect();
    Ok(serde_json::to_value(list)?)
}

async fn execute_install_software(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let package = params.get("package")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing package parameter"))?;

    let silent = params.get("silent")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let platform = crate::platform::get_platform();
    platform.install_software(package, silent).await?;
    Ok(serde_json::json!({ "success": true, "message": format!("Installed {}", package) }))
}

async fn execute_uninstall_software(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let package = params.get("package")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing package parameter"))?;

    let platform = crate::platform::get_platform();
    platform.uninstall_software(package).await?;
    Ok(serde_json::json!({ "success": true, "message": format!("Uninstalled {}", package) }))
}

async fn execute_get_env_var(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing name parameter"))?;

    let scope = params.get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("user");
    let scope = match scope {
        "system" => EnvScope::System,
        "session" => EnvScope::Session,
        _ => EnvScope::User,
    };

    let platform = crate::platform::get_platform();
    let value = platform.get_env_var(name, scope).await?;
    Ok(serde_json::json!({ "name": name, "value": value }))
}

async fn execute_set_env_var(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing name parameter"))?;

    let value = params.get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing value parameter"))?;

    let scope = params.get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("user");
    let scope = match scope {
        "system" => EnvScope::System,
        "session" => EnvScope::Session,
        _ => EnvScope::User,
    };

    let platform = crate::platform::get_platform();
    platform.set_env_var(name, value, scope).await?;
    Ok(serde_json::json!({ "success": true, "message": format!("Set {}={}", name, value) }))
}

async fn execute_delete_env_var(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing name parameter"))?;

    let scope = params.get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("user");
    let scope = match scope {
        "system" => EnvScope::System,
        "session" => EnvScope::Session,
        _ => EnvScope::User,
    };

    let platform = crate::platform::get_platform();
    platform.delete_env_var(name, scope).await?;
    Ok(serde_json::json!({ "success": true, "message": format!("Deleted {}", name) }))
}

async fn execute_read_file(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let path = params.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;

    let platform = crate::platform::get_platform();
    let content = platform.read_file(path).await?;
    Ok(serde_json::json!({ "path": path, "content": content }))
}

async fn execute_write_file(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let path = params.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;

    let content = params.get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing content parameter"))?;

    let platform = crate::platform::get_platform();
    platform.write_file(path, content).await?;
    Ok(serde_json::json!({ "success": true, "message": format!("Wrote to {}", path) }))
}

async fn execute_delete_file(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let path = params.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;

    let platform = crate::platform::get_platform();
    platform.delete_file(path).await?;
    Ok(serde_json::json!({ "success": true, "message": format!("Deleted {}", path) }))
}

async fn execute_create_dir(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let path = params.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;

    let recursive = params.get("recursive")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let platform = crate::platform::get_platform();
    platform.create_dir(path, recursive).await?;
    Ok(serde_json::json!({ "success": true, "message": format!("Created directory {}", path) }))
}

async fn execute_copy_file(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let src = params.get("src")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing src parameter"))?;

    let dst = params.get("dst")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing dst parameter"))?;

    let platform = crate::platform::get_platform();
    platform.copy_file(src, dst).await?;
    Ok(serde_json::json!({ "success": true, "message": format!("Copied {} to {}", src, dst) }))
}

async fn execute_move_file(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let src = params.get("src")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing src parameter"))?;

    let dst = params.get("dst")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing dst parameter"))?;

    let platform = crate::platform::get_platform();
    platform.move_file(src, dst).await?;
    Ok(serde_json::json!({ "success": true, "message": format!("Moved {} to {}", src, dst) }))
}

async fn execute_download_file(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let url = params.get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing url parameter"))?;

    let dest = params.get("dest")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing dest parameter"))?;

    let platform = crate::platform::get_platform();
    let path = platform.download_file(url, dest).await?;
    Ok(serde_json::json!({ "success": true, "path": path }))
}

async fn execute_get_config(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let path = params.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;

    let platform = crate::platform::get_platform();
    let value = platform.get_config(path).await?;
    Ok(value)
}

async fn execute_set_config(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let path = params.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;

    let value = params.get("value")
        .ok_or_else(|| anyhow::anyhow!("Missing value parameter"))?;

    let platform = crate::platform::get_platform();
    platform.set_config(path, value.clone()).await?;
    Ok(serde_json::json!({ "success": true, "message": format!("Set config {}", path) }))
}

async fn execute_reboot() -> Result<serde_json::Value, anyhow::Error> {
    let platform = crate::platform::get_platform();
    platform.reboot().await?;
    Ok(serde_json::json!({ "success": true, "message": "Reboot initiated" }))
}

async fn execute_shutdown() -> Result<serde_json::Value, anyhow::Error> {
    let platform = crate::platform::get_platform();
    platform.shutdown().await?;
    Ok(serde_json::json!({ "success": true, "message": "Shutdown initiated" }))
}

async fn execute_stop_process(params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let pid = params.get("pid")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing pid parameter"))?;

    let force = params.get("force")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let platform = crate::platform::get_platform();
    platform.stop_process(pid as u32, force).await?;
    Ok(serde_json::json!({ "success": true, "message": format!("Stopped process {}", pid) }))
}
