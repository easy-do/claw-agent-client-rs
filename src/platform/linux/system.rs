use crate::error::AgentResult;

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

pub async fn get_hostname() -> AgentResult<String> {
    Ok(whoami::hostname())
}
