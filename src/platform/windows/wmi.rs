use crate::error::AgentResult;

pub async fn query_wmi(_query: &str) -> AgentResult<serde_json::Value> {
    let output = std::process::Command::new("powershell")
        .args(&["-Command", &format!("Get-CimInstance -Query '{}' | ConvertTo-Json", _query)])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("WMI query failed").into());
    }
    
    let json_str = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&json_str)
        .unwrap_or(serde_json::Value::String(json_str.to_string()));
    
    Ok(value)
}

pub async fn get_installed_software_list() -> AgentResult<Vec<serde_json::Value>> {
    let query = r#"SELECT DisplayName, DisplayVersion, Publisher, InstallDate FROM Win32_Product WHERE DisplayName IS NOT NULL"#;
    
    let output = std::process::Command::new("powershell")
        .args(&["-Command", &format!("Get-CimInstance -Query '{}' | ConvertTo-Json -Depth 3", query)])
        .output()?;
    
    if !output.status.success() {
        return Ok(vec![]);
    }
    
    let json_str = String::from_utf8_lossy(&output.stdout);
    
    if json_str.trim().is_empty() || json_str.trim() == "null" {
        return Ok(vec![]);
    }
    
    let value: serde_json::Value = serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null);
    
    match value {
        serde_json::Value::Array(arr) => Ok(arr),
        _ => Ok(vec![value]),
    }
}
