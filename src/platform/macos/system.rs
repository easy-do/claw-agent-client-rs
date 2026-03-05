use crate::platform::types::{CpuInfo, MemoryInfo};
use crate::error::AgentResult;
use sysinfo::System;

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

pub async fn get_memory_info() -> AgentResult<MemoryInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let total = sys.total_memory();
    let available = sys.available_memory();
    let used = total - available;
    
    Ok(MemoryInfo {
        total_gb: total as f64 / 1_073_741_824.0,
        available_gb: available as f64 / 1_073_741_824.0,
        used_gb: used as f64 / 1_073_741_824.0,
        usage_percent: (used as f64 / total as f64 * 100.0) as f32,
    })
}

pub async fn get_cpu_info() -> AgentResult<CpuInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let cpus = sys.cpus();
    let count = cpus.len();
    let usage: f32 = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / count as f32;
    
    let brand = cpus.first().map(|c| c.brand().to_string()).unwrap_or_default();
    let frequency = cpus.first().map(|c| c.frequency()).unwrap_or(0);
    
    Ok(CpuInfo {
        count,
        usage_percent: usage,
        brand,
        frequency_mhz: frequency,
    })
}

pub async fn get_hostname() -> AgentResult<String> {
    Ok(whoami::hostname())
}
