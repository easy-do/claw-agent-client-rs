use crate::platform::types::{CpuInfo, MemoryInfo, OsVersion};
use crate::error::AgentResult;
use sysinfo::System;

pub async fn get_os_version() -> AgentResult<String> {
    let output = std::process::Command::new("powershell")
        .args(&["-Command", "(Get-CimInstance Win32_OperatingSystem).Caption"])
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn get_uptime() -> AgentResult<u64> {
    let output = std::process::Command::new("powershell")
        .args(&["-Command", "(Get-Date) - (Get-CimInstance Win32_OperatingSystem).LastBootUpTime | Select-Object -ExpandProperty TotalSeconds"])
        .output()?;
    
    let secs: u64 = String::from_utf8_lossy(&output.stdout).trim().parse().unwrap_or(0);
    Ok(secs)
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
