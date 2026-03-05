use crate::platform::types::{BrowserType, FileInfo, SoftwarePackage, UserDirType};
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

pub async fn list_dir_impl(path: &str) -> AgentResult<Vec<FileInfo>> {
    let entries = std::fs::read_dir(path)?;
    let mut files = Vec::new();
    
    for entry in entries.flatten() {
        let metadata = entry.metadata()?;
        let name = entry.file_name().to_string_lossy().to_string();
        let path_str = entry.path().to_string_lossy().to_string();
        
        files.push(FileInfo {
            path: path_str,
            name,
            size_bytes: metadata.len(),
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
            modified: metadata.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
            created: metadata.created().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
            permissions: Some(if metadata.permissions().readonly() { "readonly".to_string() } else { "writable".to_string() }),
        });
    }
    
    Ok(files)
}

pub async fn download_file_impl(url: &str, dest: &str) -> AgentResult<String> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    
    let bytes = response.bytes().await?;
    std::fs::write(dest, &bytes)?;
    
    Ok(dest.to_string())
}

pub fn get_user_dir_impl(dir_type: UserDirType) -> String {
    let home = dirs::home_dir().unwrap_or_default();
    
    match dir_type {
        UserDirType::Home => home.to_string_lossy().to_string(),
        UserDirType::Desktop => home.join("Desktop").to_string_lossy().to_string(),
        UserDirType::Documents => home.join("Documents").to_string_lossy().to_string(),
        UserDirType::Downloads => home.join("Downloads").to_string_lossy().to_string(),
        UserDirType::Pictures => home.join("Pictures").to_string_lossy().to_string(),
        UserDirType::Music => home.join("Music").to_string_lossy().to_string(),
        UserDirType::Videos => home.join("Videos").to_string_lossy().to_string(),
        UserDirType::Temp => std::env::temp_dir().to_string_lossy().to_string(),
        UserDirType::Cache => home.join(".cache").to_string_lossy().to_string(),
        UserDirType::Config => home.join(".config").to_string_lossy().to_string(),
    }
}
