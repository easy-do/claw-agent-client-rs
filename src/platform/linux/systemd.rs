use crate::error::AgentResult;
use std::process::Command;

pub async fn list_services() -> AgentResult<Vec<String>> {
    let output = Command::new("systemctl")
        .args(&["list-units", "--type=service", "--all", "--no-pager"])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let services: Vec<String> = stdout.lines()
        .skip(1)
        .filter_map(|line| {
            let name = line.split_whitespace().next()?;
            if name.ends_with(".service") {
                Some(name.trim_end_matches(".service").to_string())
            } else {
                None
            }
        })
        .collect();
    
    Ok(services)
}

pub async fn start_service(name: &str) -> AgentResult<()> {
    let output = Command::new("systemctl")
        .args(&["start", name])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to start service: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}

pub async fn stop_service(name: &str) -> AgentResult<()> {
    let output = Command::new("systemctl")
        .args(&["stop", name])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to stop service: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}

pub async fn restart_service(name: &str) -> AgentResult<()> {
    let output = Command::new("systemctl")
        .args(&["restart", name])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to restart service: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}

pub async fn enable_service(name: &str) -> AgentResult<()> {
    let output = Command::new("systemctl")
        .args(&["enable", name])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to enable service: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}

pub async fn disable_service(name: &str) -> AgentResult<()> {
    let output = Command::new("systemctl")
        .args(&["disable", name])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to disable service: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}

pub async fn get_service_status(name: &str) -> AgentResult<String> {
    let output = Command::new("systemctl")
        .args(&["status", name])
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
