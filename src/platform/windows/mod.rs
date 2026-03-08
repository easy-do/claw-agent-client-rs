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
use crate::platform::common::{get_memory_info, get_cpu_info, build_system_info};
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
        let hostname = whoami::hostname();
        let username = whoami::username();
        
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
    
    async fn start_process(
        &self,
        program: &str,
        args: &[String],
        elevated: bool,
        working_dir: Option<&str>,
    ) -> AgentResult<u32> {
        if elevated {
            let ps_args = format!(
                "Start-Process '{}' -ArgumentList '{}' -Verb RunAs -PassThru -WorkingDirectory '{}' | Select-Object -ExpandProperty Id",
                program,
                args.join(" "),
                working_dir.unwrap_or(".")
            );
            let output = Command::new("powershell")
                .args(&["-Command", &ps_args])
                .output()?;
            
            if !output.status.success() {
                return Err(AgentError::Command(
                    format!("Elevated start failed: {}", String::from_utf8_lossy(&output.stderr))
                ).into());
            }
            
            let pid = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<u32>()?;
            Ok(pid)
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
    
    async fn find_process(&self, name: &str) -> AgentResult<Option<ProcessInfo>> {
        let processes = self.list_processes().await?;
        Ok(processes.into_iter().find(|p| p.name.eq_ignore_ascii_case(name)))
    }
    
    async fn get_process_info(&self, pid: u32) -> AgentResult<Option<ProcessInfo>> {
        let processes = self.list_processes().await?;
        Ok(processes.into_iter().find(|p| p.pid == pid))
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
    
    async fn toggle_feature(&self, feature: &str, enabled: bool) -> AgentResult<()> {
        let action = if enabled { "Enable" } else { "Disable" };
        let ps_cmd = format!(
            "{}-WindowsOptionalFeature -Online -FeatureName {} -NoRestart",
            action, feature
        );
        let output = Command::new("powershell")
            .args(&["-Command", &ps_cmd])
            .output()?;
        
        if !output.status.success() {
            return Err(AgentError::Command(
                format!("Feature toggle failed: {}", String::from_utf8_lossy(&output.stderr))
            ).into());
        }
        Ok(())
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
    
    async fn update_software(&self, package: Option<&str>) -> AgentResult<()> {
        let args = match package {
            Some(pkg) => vec!["upgrade", "--id", pkg, "--silent"],
            None => vec!["upgrade", "--all", "--silent"],
        };
        
        Command::new("winget").args(&args).output()?;
        Ok(())
    }
    
    async fn check_updates(&self) -> AgentResult<Vec<SoftwarePackage>> {
        check_winget_updates().await
    }
    
    fn get_default_browser(&self) -> BrowserType {
        get_default_browser_impl().unwrap_or(BrowserType::Edge)
    }
    
    async fn launch_browser(&self, browser: BrowserType, url: &str) -> AgentResult<()> {
        let browser_path = get_browser_path(browser)?;
        Command::new(browser_path).arg(url).spawn()?;
        Ok(())
    }
    
    async fn close_browser(&self, browser: BrowserType) -> AgentResult<()> {
        let process_name = get_browser_process_name(browser);
        let processes = self.list_processes().await?;
        for proc in processes.iter().filter(|p| p.name.eq_ignore_ascii_case(process_name)) {
            self.stop_process(proc.pid, false).await?;
        }
        Ok(())
    }
    
    fn get_program_files_dir(&self) -> String {
        std::env::var("ProgramFiles")
            .unwrap_or_else(|_| r"C:\Program Files".to_string())
    }
    
    fn is_elevated(&self) -> bool {
        is_elevated_impl()
    }
}
