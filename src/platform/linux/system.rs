use crate::platform::types::{CpuInfo, MemoryInfo};
use crate::error::AgentResult;
use sysinfo::System;

pub async fn get_os_version() -> AgentResult<String> {
    let output = std::process::Command::new("cat")
        .arg("/etc/os-release")
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    for line in stdout.lines() {
        if line.starts_with("PRETTY_NAME=") {
            return Ok(line.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string());
        }
    }
    
    Ok("Linux".to_string())
}

pub async fn get_uptime() -> AgentResult<u64> {
    let output = std::process::Command::new("cat")
        .arg("/proc/uptime")
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(uptime_str) = stdout.split_whitespace().next() {
        if let Ok(uptime) = uptime_str.parse::<f64>() {
            return Ok(uptime as u64);
        }
    }
    
    Ok(0)
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
