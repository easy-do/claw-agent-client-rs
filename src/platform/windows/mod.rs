pub mod system;
pub mod env;
pub mod registry;
pub mod wmi;
pub mod security;

pub use system::*;
pub use env::*;
pub use registry::*;
pub use wmi::*;
pub use security::*;

use crate::platform::traits::*;
use crate::platform::types::*;
use crate::platform::common::{get_memory_info, get_cpu_info, build_system_info, get_hostname, get_username};
use crate::error::AgentResult;
use async_trait::async_trait;
use std::process::Command;
use std::collections::HashMap;
use crate::error::AgentError;

pub struct WindowsPlatform;

impl WindowsPlatform {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Platform for WindowsPlatform {
    fn name(&self) -> &'static str {
        "windows"
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
            "Windows".to_string(),
            os_version,
            uptime,
            memory,
            cpu,
        ))
    }
    
    async fn stop_process(&self, pid: u32, force: bool) -> AgentResult<()> {
        let flag = if force { "/F" } else { "" };
        let output = Command::new("taskkill")
            .args(&[flag, "/PID", &pid.to_string()])
            .output()?;
        
        if !output.status.success() {
            return Err(AgentError::Command(
                format!("Stop process failed: {}", String::from_utf8_lossy(&output.stderr))
            ).into());
        }
        Ok(())
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
        read_registry_value(path).await
    }
    
    async fn set_config(&self, path: &str, value: serde_json::Value) -> AgentResult<()> {
        write_registry_value(path, value).await
    }
    
    async fn list_software(&self) -> AgentResult<Vec<SoftwarePackage>> {
        list_installed_software().await
    }
    
    async fn search_software(&self, query: &str) -> AgentResult<Vec<SoftwarePackage>> {
        search_winget_packages(query).await
    }
    
    async fn install_software(&self, package: &str, silent: bool) -> AgentResult<()> {
        let mut args = vec!["install", "--id", package];
        if silent {
            args.extend(&[
                "--silent",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ]);
        }
        
        let output = Command::new("winget").args(&args).output()?;
        
        if !output.status.success() {
            return Err(AgentError::Command(
                format!("Install failed: {}", String::from_utf8_lossy(&output.stderr))
            ).into());
        }
        Ok(())
    }
    
    async fn uninstall_software(&self, package: &str) -> AgentResult<()> {
        let output = Command::new("winget")
            .args(&["uninstall", "--id", package, "--silent"])
            .output()?;
        
        if !output.status.success() {
            return Err(AgentError::Command(
                format!("Uninstall failed: {}", String::from_utf8_lossy(&output.stderr))
            ).into());
        }
        Ok(())
    }
    
    fn get_program_files_dir(&self) -> String {
        std::env::var("ProgramFiles")
            .unwrap_or_else(|_| r"C:\Program Files".to_string())
    }
}
