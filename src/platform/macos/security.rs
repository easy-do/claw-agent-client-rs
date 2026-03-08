use crate::platform::types::{BrowserType, SoftwarePackage, UserDirType};
use crate::error::AgentResult;
use std::process::Command;

pub fn is_elevated_impl() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub async fn get_app_version(path: &str) -> AgentResult<String> {
    let plist_path = format!("{}/Contents/Info.plist", path);
    let output = Command::new("plutil")
        .args(&["-p", &plist_path])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    for line in stdout.lines() {
        if line.contains("CFBundleShortVersionString") {
            if let Some(version) = line.split_whitespace().nth(1) {
                return Ok(version.trim_matches('"').to_string());
            }
        }
    }
    
    Ok("unknown".to_string())
}

pub async fn get_brew_version(package: &str) -> AgentResult<String> {
    let output = Command::new("brew")
        .args(&["info", package])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    for line in stdout.lines() {
        if line.contains(package) && line.contains(": ") {
            if let Some(version) = line.split(':').nth(1) {
                return Ok(version.trim().split_whitespace().next().unwrap_or("unknown").to_string());
            }
        }
    }
    
    Ok("unknown".to_string())
}

pub async fn search_brew_packages(query: &str) -> AgentResult<Vec<SoftwarePackage>> {
    let output = Command::new("brew")
        .args(&["search", query])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();
    
    for line in stdout.lines() {
        let name = line.trim().trim_matches(|c| c == '/' || c == ' ').to_string();
        if !name.is_empty() {
            packages.push(SoftwarePackage {
                name,
                version: "unknown".to_string(),
                publisher: Some("Homebrew".to_string()),
                install_path: None,
                install_date: None,
            });
        }
    }
    
    Ok(packages)
}

pub async fn check_brew_updates() -> AgentResult<Vec<SoftwarePackage>> {
    Command::new("brew").arg("update").output()?;
    
    let output = Command::new("brew")
        .args(&["outdated", "--json"])
        .output()?;
    
    if !output.status.success() {
        return Ok(vec![]);
    }
    
    let json_str = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null);
    
    let mut packages = Vec::new();
    
    if let Some(arr) = value.as_array() {
        for item in arr {
            if let (Some(name), Some(version)) = (
                item.get("name").and_then(|v| v.as_str()),
                item.get("current_version").and_then(|v| v.as_str()),
            ) {
                packages.push(SoftwarePackage {
                    name: name.to_string(),
                    version: version.to_string(),
                    publisher: Some("Homebrew".to_string()),
                    install_path: None,
                    install_date: None,
                });
            }
        }
    }
    
    Ok(packages)
}

pub fn parse_feature_path(feature: &str) -> AgentResult<(String, String)> {
    let parts: Vec<&str> = feature.split('.').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid feature path format").into());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
