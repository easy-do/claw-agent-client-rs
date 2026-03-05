use crate::error::AgentResult;
use std::process::Command;

pub async fn read_registry_value(path: &str) -> AgentResult<serde_json::Value> {
    let output = Command::new("reg")
        .args(&["query", path])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Registry query failed: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = stdout.lines()
        .find(|line| !line.is_empty() && !line.starts_with("HKEY_"))
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                parts[2..].join(" ")
            } else {
                line.to_string()
            }
        })
        .unwrap_or_default();
    
    Ok(serde_json::Value::String(value))
}

pub async fn write_registry_value(path: &str, value: serde_json::Value) -> AgentResult<()> {
    let str_value = match &value {
        serde_json::Value::String(s) => s.clone(),
        _ => value.to_string(),
    };
    
    let parts: Vec<&str> = path.split('\\').collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid registry path").into());
    }
    
    let key = parts.last().unwrap();
    let parent_path = parts[..parts.len()-1].join("\\");
    
    let output = Command::new("reg")
        .args(&["add", &parent_path, "/v", key, "/t", "REG_SZ", "/d", &str_value, "/f"])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Registry write failed: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}
