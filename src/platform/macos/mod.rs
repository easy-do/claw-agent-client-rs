pub mod system;
pub mod process;
pub mod env;
pub mod applescript;
pub mod launchd;
pub mod security;

pub use system::*;
pub use process::*;
pub use env::*;
pub use applescript::*;
pub use launchd::*;
pub use security::*;

use crate::platform::traits::*;
use crate::platform::types::*;
use crate::platform::common::{get_memory_info, get_cpu_info, build_system_info};
use crate::error::AgentResult;
use crate::error::AgentError;
use async_trait::async_trait;
use std::process::Command;
use std::collections::HashMap;

pub struct MacOSPlatform;

impl MacOSPlatform {
    pub fn new() -> Self {
        Self
    }
    
    fn browser_app_name(browser: BrowserType) -> &'static str {
        match browser {
            BrowserType::Chrome => "Google Chrome",
            BrowserType::Firefox => "Firefox",
            BrowserType::Safari => "Safari",
            BrowserType::Edge => "Microsoft Edge",
            BrowserType::Brave => "Brave Browser",
        }
    }
}

#[async_trait]
impl Platform for MacOSPlatform {
    fn name(&self) -> &'static str {
        "macos"
    }
    
    async fn get_system_info(&self) -> AgentResult<SystemInfo> {
        let hostname = whoami::hostname();
        let username = whoami::username();
        
        let os_version = get_os_version().await?;
        let uptime = get_uptime().await?;
        let memory = get_memory_info().await?;
        let cpu = get_cpu_info().await?;
        
        Ok(build_system_info(
            hostname,
            username,
            "macOS".to_string(),
            os_version,
            uptime,
            memory,
            cpu,
        ))
    }
    
    async fn start_process(
        &self,
        program: &str,
        args: &[String],
        elevated: bool,
        working_dir: Option<&str>,
    ) -> AgentResult<u32> {
        if elevated {
            let script = format!(
                r#"do shell script "{} {}" with administrator privileges"#,
                program,
                args.join(" ")
            );
            let output = Command::new("osascript")
                .args(&["-e", &script])
                .output()?;
            
            if !output.status.success() {
                return Err(AgentError::Command(
                    format!("Elevated start failed: {}", String::from_utf8_lossy(&output.stderr))
                ).into());
            }
            
            Ok(0)
        } else {
            let mut cmd = Command::new(program);
            cmd.args(args);
            if let Some(dir) = working_dir {
                cmd.current_dir(dir);
            }
            let pid = cmd.spawn()?.id();
            Ok(pid)
        }
    }
    
    async fn stop_process(&self, pid: u32, force: bool) -> AgentResult<()> {
        let signal = if force { "-9" } else { "-15" };
        Command::new("kill").arg(signal).arg(pid.to_string()).output()?;
        Ok(())
    }
    
    async fn list_processes(&self) -> AgentResult<Vec<ProcessInfo>> {
        list_processes_impl().await
    }

    async fn get_env_var(&self, name: &str, scope: EnvScope) -> AgentResult<Option<String>> {
        get_env_var_impl(name, scope).await
    }
    
    async fn set_env_var(&self, name: &str, value: &str, scope: EnvScope) -> AgentResult<()> {
        set_env_var_impl(name, value, scope).await
    }
    
    async fn delete_env_var(&self, name: &str, scope: EnvScope) -> AgentResult<()> {
        delete_env_var_impl(name, scope).await
    }
    
    async fn list_env_vars(&self, scope: EnvScope) -> AgentResult<HashMap<String, String>> {
        list_env_vars_impl(scope).await
    }
    
    async fn get_config(&self, path: &str) -> AgentResult<serde_json::Value> {
        let output = Command::new("plutil")
            .args(&["-convert", "json", "-o", "-", path])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Read plist failed").into());
        }
        
        let json = String::from_utf8_lossy(&output.stdout);
        Ok(serde_json::from_str(&json)?)
    }
    
    async fn set_config(&self, path: &str, value: serde_json::Value) -> AgentResult<()> {
        let temp_file = tempfile::NamedTempFile::new()?;
        std::fs::write(&temp_file, value.to_string())?;
        
        Command::new("plutil")
            .args(&["-convert", "xml1", "-o", path, temp_file.path().to_str().unwrap()])
            .output()?;
        
        Ok(())
    }
    
    async fn toggle_feature(&self, feature: &str, enabled: bool) -> AgentResult<()> {
        let (domain, key) = parse_feature_path(feature)?;
        let action = if enabled { "true" } else { "false" };
        
        Command::new("defaults")
            .args(&["write", &domain, &key, "-bool", action])
            .output()?;
        
        Ok(())
    }
    
    async fn list_software(&self) -> AgentResult<Vec<SoftwarePackage>> {
        let mut packages = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir("/Applications") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".app") {
                    let version = get_app_version(entry.path().to_str().unwrap())
                        .await
                        .unwrap_or_else(|_| "unknown".to_string());
                    
                    packages.push(SoftwarePackage {
                        name: name.trim_end_matches(".app").to_string(),
                        version,
                        publisher: None,
                        install_path: Some(entry.path().to_string_lossy().to_string()),
                        install_date: None,
                    });
                }
            }
        }
        
        if let Ok(output) = Command::new("brew").args(&["list", "--formula"]).output() {
            let formulae = String::from_utf8_lossy(&output.stdout);
            for formula in formulae.lines() {
                let version = get_brew_version(formula)
                    .await
                    .unwrap_or_else(|_| "unknown".to_string());
                
                packages.push(SoftwarePackage {
                    name: formula.to_string(),
                    version,
                    publisher: Some("Homebrew".to_string()),
                    install_path: None,
                    install_date: None,
                });
            }
        }
        
        if let Ok(output) = Command::new("brew").args(&["list", "--cask"]).output() {
            let casks = String::from_utf8_lossy(&output.stdout);
            for cask in casks.lines() {
                packages.push(SoftwarePackage {
                    name: cask.to_string(),
                    version: "unknown".to_string(),
                    publisher: Some("Homebrew Cask".to_string()),
                    install_path: None,
                    install_date: None,
                });
            }
        }
        
        Ok(packages)
    }
    
    async fn search_software(&self, query: &str) -> AgentResult<Vec<SoftwarePackage>> {
        search_brew_packages(query).await
    }
    
    async fn install_software(&self, package: &str, silent: bool) -> AgentResult<()> {
        let mut args = vec!["install"];
        if silent {
            args.push("--quiet");
        }
        args.push(package);
        
        let output = Command::new("brew").args(&args).output()?;
        
        if !output.status.success() {
            return Err(AgentError::Command(
                format!("Install failed: {}", String::from_utf8_lossy(&output.stderr))
            ).into());
        }
        Ok(())
    }
    
    async fn uninstall_software(&self, package: &str) -> AgentResult<()> {
        let output = Command::new("brew")
            .args(&["uninstall", package])
            .output()?;
        
        if !output.status.success() {
            return Err(AgentError::Command(
                format!("Uninstall failed: {}", String::from_utf8_lossy(&output.stderr))
            ).into());
        }
        Ok(())
    }
    
    async fn update_software(&self, package: Option<&str>) -> AgentResult<()> {
        Command::new("brew").arg("update").output()?;
        
        let args = match package {
            Some(pkg) => vec!["upgrade", pkg],
            None => vec!["upgrade"],
        };
        
        Command::new("brew").args(&args).output()?;
        Ok(())
    }
    
    async fn check_updates(&self) -> AgentResult<Vec<SoftwarePackage>> {
        check_brew_updates().await
    }
    
    fn get_default_browser(&self) -> BrowserType {
        BrowserType::Safari
    }
    
    async fn launch_browser(&self, browser: BrowserType, url: &str) -> AgentResult<()> {
        let app_name = Self::browser_app_name(browser);
        
        Command::new("open")
            .args(&["-a", app_name, url])
            .output()?;
        
        Ok(())
    }
    
    async fn close_browser(&self, browser: BrowserType) -> AgentResult<()> {
        let app_name = Self::browser_app_name(browser);
        
        Command::new("killall").arg(app_name).output()?;
        Ok(())
    }

    fn get_program_files_dir(&self) -> String {
        "/Applications".to_string()
    }
    
    fn is_elevated(&self) -> bool {
        is_elevated_impl()
    }
}
