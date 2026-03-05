use crate::error::AgentResult;

pub async fn run_applescript(script: &str) -> AgentResult<String> {
    use std::process::Command;
    
    let output = Command::new("osascript")
        .args(&["-e", script])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("AppleScript failed: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn run_applescript_file(path: &str) -> AgentResult<String> {
    use std::process::Command;
    
    let output = Command::new("osascript")
        .arg(path)
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("AppleScript file failed: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
