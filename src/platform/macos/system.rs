use crate::platform::types::CpuInfo;
use crate::error::AgentResult;
use crate::platform::types::MemoryInfo;
use crate::platform::common::get_hostname;

pub async fn get_os_version() -> AgentResult<String> {
    let output = std::process::Command::new("sw_vers")
        .args(&["-productVersion"])
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn get_uptime() -> AgentResult<u64> {
    let output = std::process::Command::new("uptime")
        .args(&["-s"])
        .output()?;
    
    let uptime_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let boot_time = chrono::DateTime::parse_from_rfc3339(&format!("{}Z", uptime_str))
        .map(|dt| dt.timestamp() as u64)
        .unwrap_or(0);
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Ok(now - boot_time)
}
