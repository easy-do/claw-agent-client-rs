use crate::platform::types::*;

pub enum Command {
    SystemInfo,
    ProcessList,
    SoftwareList,
    EnvList,
    FileList,
    ShellExecute,
}

impl Command {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "system.info" => Some(Command::SystemInfo),
            "process.list" => Some(Command::ProcessList),
            "software.list" => Some(Command::SoftwareList),
            "env.list" => Some(Command::EnvList),
            "file.list" => Some(Command::FileList),
            "shell.execute" => Some(Command::ShellExecute),
            _ => None,
        }
    }

    pub async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        match self {
            Command::SystemInfo => execute_system_info().await,
            Command::ProcessList => execute_process_list().await,
            Command::SoftwareList => execute_software_list().await,
            Command::EnvList => execute_env_list(params).await,
            Command::FileList => execute_file_list(params).await,
            Command::ShellExecute => execute_shell(params).await,
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
