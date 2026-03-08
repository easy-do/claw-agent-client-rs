pub mod system;
pub mod env;
pub mod applescript;
pub mod launchd;
pub mod security;

pub use system::*;
pub use env::*;
pub use applescript::*;
pub use launchd::*;
pub use security::*;

use crate::platform::traits::*;
use crate::platform::types::*;
use crate::platform::common::*;
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
}

#[async_trait]
impl Platform for MacOSPlatform {
    fn name(&self) -> &'static str {
        "macos"
    }
    
    async fn get_system_info(&self) -> AgentResult<SystemInfo> {
        let hostname = get_hostname()?;
        let username = get_username()?;
        
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
    
    fn get_program_files_dir(&self) -> String {
        "/Applications".to_string()
    }
}
