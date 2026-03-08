use crate::platform::types::{BrowserType, SoftwarePackage, UserDirType};
use crate::error::AgentResult;
use std::process::Command;

pub fn is_elevated_impl() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub async fn list_dpkg_packages() -> AgentResult<Vec<SoftwarePackage>> {
    let output = Command::new("dpkg")
        .args(&["--list"])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();
    
    for line in stdout.lines().skip(5) {
        if line.starts_with("ii") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                packages.push(SoftwarePackage {
                    name: parts[1].to_string(),
                    version: parts[2].to_string(),
                    publisher: None,
                    install_path: None,
                    install_date: None,
                });
            }
        }
    }
    
    Ok(packages)
}

pub async fn list_rpm_packages() -> AgentResult<Vec<SoftwarePackage>> {
    let output = Command::new("rpm")
        .args(&["-qa", "--queryformat", "%{NAME}|%{VERSION}\\n"])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();
    
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 2 {
            packages.push(SoftwarePackage {
                name: parts[0].to_string(),
                version: parts[1].to_string(),
                publisher: None,
                install_path: None,
                install_date: None,
            });
        }
    }
    
    Ok(packages)
}

pub async fn list_snap_packages() -> AgentResult<Vec<SoftwarePackage>> {
    let output = Command::new("snap")
        .args(&["list"])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();
    
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            packages.push(SoftwarePackage {
                name: parts[0].to_string(),
                version: parts[1].to_string(),
                publisher: Some("Snap".to_string()),
                install_path: None,
                install_date: None,
            });
        }
    }
    
    Ok(packages)
}

pub async fn list_flatpak_packages() -> AgentResult<Vec<SoftwarePackage>> {
    let output = Command::new("flatpak")
        .args(&["list", "--app"])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();
    
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            packages.push(SoftwarePackage {
                name: parts.last().unwrap_or(&"").to_string(),
                version: parts.get(1).unwrap_or(&"").to_string(),
                publisher: Some("Flatpak".to_string()),
                install_path: None,
                install_date: None,
            });
        }
    }
    
    Ok(packages)
}

pub async fn search_packages(query: &str) -> AgentResult<Vec<SoftwarePackage>> {
    let mut packages = Vec::new();
    
    if which::which("apt-cache").is_ok() {
        let output = Command::new("apt-cache")
            .args(&["search", query])
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(2, "- ").collect();
            if parts.len() >= 1 {
                packages.push(SoftwarePackage {
                    name: parts[0].trim().to_string(),
                    version: "unknown".to_string(),
                    publisher: Some("apt".to_string()),
                    install_path: None,
                    install_date: None,
                });
            }
        }
    }
    
    Ok(packages)
}

pub async fn check_updates_impl() -> AgentResult<Vec<SoftwarePackage>> {
    let mut packages = Vec::new();
    
    if which::which("apt-get").is_ok() {
        Command::new("apt-get")
            .args(&["update"])
            .output()?;
        
        let output = Command::new("apt-get")
            .args(&["-s", "upgrade"])
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            if line.contains("Inst ") {
                let parts: Vec<&str> = line.split("Inst ").collect();
                if parts.len() >= 2 {
                    let name_version: Vec<&str> = parts[1].split_whitespace().collect();
                    if !name_version.is_empty() {
                        packages.push(SoftwarePackage {
                            name: name_version[0].to_string(),
                            version: "unknown".to_string(),
                            publisher: Some("apt".to_string()),
                            install_path: None,
                            install_date: None,
                        });
                    }
                }
            }
        }
    }
    
    Ok(packages)
}
