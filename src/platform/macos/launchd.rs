use crate::error::AgentResult;
use std::process::Command;

pub async fn list_launchd_jobs() -> AgentResult<Vec<String>> {
    let output = Command::new("launchctl")
        .args(&["list"])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let jobs: Vec<String> = stdout.lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            parts.last().map(|s| s.to_string())
        })
        .collect();
    
    Ok(jobs)
}

pub async fn start_launchd_job(label: &str) -> AgentResult<()> {
    let output = Command::new("launchctl")
        .args(&["start", label])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to start job: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}

pub async fn stop_launchd_job(label: &str) -> AgentResult<()> {
    let output = Command::new("launchctl")
        .args(&["stop", label])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to stop job: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}

pub async fn load_launchd_plist(path: &str) -> AgentResult<()> {
    let output = Command::new("launchctl")
        .args(&["load", path])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to load plist: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}

pub async fn unload_launchd_plist(path: &str) -> AgentResult<()> {
    let output = Command::new("launchctl")
        .args(&["unload", path])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to unload plist: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}
