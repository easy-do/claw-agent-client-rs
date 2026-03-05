use crate::platform::types::{BrowserType, FileInfo, SoftwarePackage};
use crate::error::AgentResult;
use std::process::Command;

pub fn is_elevated_impl() -> bool {
    #[cfg(windows)]
    {
        Command::new("net")
            .args(["session"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    #[cfg(not(windows))]
    {
        false
    }
}

pub fn get_default_browser_impl() -> Option<BrowserType> {
    let output = Command::new("powershell")
        .args(&["-Command", r#"Get-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\Shell\Associations\UrlAssociations\http\UserChoice' -Name 'ProgId' | Select-Object -ExpandProperty ProgId"#])
        .output()
        .ok()?;
    
    let result = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
    
    if result.contains("chrome") {
        Some(BrowserType::Chrome)
    } else if result.contains("firefox") {
        Some(BrowserType::Firefox)
    } else if result.contains("edge") || result.contains("microsoft") {
        Some(BrowserType::Edge)
    } else if result.contains("brave") {
        Some(BrowserType::Brave)
    } else {
        Some(BrowserType::Chrome)
    }
}

pub fn get_browser_path(browser: BrowserType) -> AgentResult<String> {
    let browser_key = match browser {
        BrowserType::Chrome => r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\chrome.exe",
        BrowserType::Firefox => r"SOFTWARE\Mozilla\Mozilla Firefox",
        BrowserType::Edge => r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\msedge.exe",
        BrowserType::Brave => r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\brave.exe",
        BrowserType::Safari => r"SOFTWARE\Apple Computer\Safari",
    };
    
    let path = format!("HKLM\\{}", browser_key);
    
    let output = Command::new("reg")
        .args(&["query", &path])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Browser not found: {:?}", browser).into());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let exe_path = stdout.lines()
        .find(|line| line.contains(".exe"))
        .map(|line| line.split_whitespace().last().unwrap_or(""))
        .unwrap_or("");
    
    Ok(exe_path.to_string())
}

pub fn get_browser_process_name(browser: BrowserType) -> &'static str {
    match browser {
        BrowserType::Chrome => "chrome.exe",
        BrowserType::Firefox => "firefox.exe",
        BrowserType::Edge => "msedge.exe",
        BrowserType::Brave => "brave.exe",
        BrowserType::Safari => "safari.exe",
    }
}

pub async fn list_installed_software() -> AgentResult<Vec<SoftwarePackage>> {
    let mut packages = Vec::new();
    
    let reg_paths = [
        r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    ];
    
    for reg_path in &reg_paths {
        let output = Command::new("reg")
            .args(&["query", reg_path])
            .output()?;
        
        if !output.status.success() {
            continue;
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            if line.starts_with("HKEY_") {
                let subkey = line.trim();
                
                let name_output = Command::new("reg")
                    .args(&["query", subkey, "/v", "DisplayName"])
                    .output()?;
                
                let name = String::from_utf8_lossy(&name_output.stdout)
                    .lines()
                    .find(|l| l.contains("DisplayName"))
                    .map(|l| l.split_whitespace().skip(2).collect::<Vec<_>>().join(" "));
                
                if let Some(name) = name {
                    let version_output = Command::new("reg")
                        .args(&["query", subkey, "/v", "DisplayVersion"])
                        .output()?;
                    
                    let version = String::from_utf8_lossy(&version_output.stdout)
                        .lines()
                        .find(|l| l.contains("DisplayVersion"))
                        .map(|l| l.split_whitespace().skip(2).collect::<Vec<_>>().join(" "))
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    packages.push(SoftwarePackage {
                        name,
                        version,
                        publisher: None,
                        install_path: Some(subkey.to_string()),
                        install_date: None,
                    });
                }
            }
        }
    }
    
    Ok(packages)
}

pub async fn search_winget_packages(query: &str) -> AgentResult<Vec<SoftwarePackage>> {
    let output = Command::new("winget")
        .args(&["search", query, "--accept-source-agreements"])
        .output()?;
    
    if !output.status.success() {
        return Ok(vec![]);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();
    
    for line in stdout.lines().skip(2) {
        if line.len() > 10 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                packages.push(SoftwarePackage {
                    name: parts[0].to_string(),
                    version: parts.get(1).unwrap_or(&"").to_string(),
                    publisher: None,
                    install_path: None,
                    install_date: None,
                });
            }
        }
    }
    
    Ok(packages)
}

pub async fn check_winget_updates() -> AgentResult<Vec<SoftwarePackage>> {
    let output = Command::new("winget")
        .args(&["list", "--upgrade-available", "--accept-source-agreements"])
        .output()?;
    
    if !output.status.success() {
        return Ok(vec![]);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();
    
    for line in stdout.lines().skip(2) {
        if line.len() > 10 && !line.contains("---") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                packages.push(SoftwarePackage {
                    name: parts[0].to_string(),
                    version: parts.get(1).unwrap_or(&"").to_string(),
                    publisher: None,
                    install_path: None,
                    install_date: None,
                });
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
            permissions: Some(if metadata.permissions().readonly() { "444" } else { "644" }.to_string()),
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

pub fn get_user_dir_impl(dir_type: crate::platform::types::UserDirType) -> String {
    use crate::platform::types::UserDirType;
    
    match dir_type {
        UserDirType::Home => dirs::home_dir().unwrap_or_default().to_string_lossy().to_string(),
        UserDirType::Desktop => dirs::desktop_dir().unwrap_or_default().to_string_lossy().to_string(),
        UserDirType::Documents => dirs::document_dir().unwrap_or_default().to_string_lossy().to_string(),
        UserDirType::Downloads => dirs::download_dir().unwrap_or_default().to_string_lossy().to_string(),
        UserDirType::Pictures => dirs::picture_dir().unwrap_or_default().to_string_lossy().to_string(),
        UserDirType::Music => dirs::audio_dir().unwrap_or_default().to_string_lossy().to_string(),
        UserDirType::Videos => dirs::video_dir().unwrap_or_default().to_string_lossy().to_string(),
        UserDirType::Temp => std::env::temp_dir().to_string_lossy().to_string(),
        UserDirType::Cache => dirs::cache_dir().unwrap_or_default().to_string_lossy().to_string(),
        UserDirType::Config => dirs::config_dir().unwrap_or_default().to_string_lossy().to_string(),
    }
}
