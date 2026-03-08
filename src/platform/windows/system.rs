use crate::error::AgentResult;

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

pub async fn get_hostname() -> AgentResult<String> {
    Ok(whoami::hostname())
}
