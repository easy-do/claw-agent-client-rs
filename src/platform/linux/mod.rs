pub mod system;
pub mod process;
pub mod env;
pub mod dbus;
pub mod systemd;
pub mod security;

pub use system::*;
pub use process::*;
pub use env::*;
pub use dbus::*;
pub use systemd::*;
pub use security::*;

use crate::platform::traits::*;
use crate::platform::types::*;
use crate::platform::common::{get_memory_info, get_cpu_info, build_system_info, get_hostname, get_username};
use crate::error::AgentResult;
use crate::error::AgentError;
use async_trait::async_trait;
use std::process::Command;
use std::collections::HashMap;

pub struct LinuxPlatform;

impl LinuxPlatform {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Platform for LinuxPlatform {
    fn name(&self) -> &'static str {
        "linux"
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
            "Linux".to_string(),
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
            let mut cmd = Command::new("sudo");
            cmd.arg(program).args(args);
            if let Some(dir) = working_dir {
                cmd.current_dir(dir);
            }
            let pid = cmd.spawn()?.id();
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
        let content = std::fs::read_to_string(path)?;
        
        if path.ends_with(".json") {
            Ok(serde_json::from_str(&content)?)
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
            Ok(serde_json::to_value(yaml)?)
        } else {
            Ok(serde_json::Value::String(content))
        }
    }
    
    async fn set_config(&self, path: &str, value: serde_json::Value) -> AgentResult<()> {
        std::fs::write(path, value.to_string())?;
        Ok(())
    }
    
    async fn toggle_feature(&self, feature: &str, enabled: bool) -> AgentResult<()> {
        let action = if enabled { "enable" } else { "disable" };
        Command::new("systemctl")
            .args(&[action, "--now", feature])
            .output()?;
        Ok(())
    }
    
    async fn list_software(&self) -> AgentResult<Vec<SoftwarePackage>> {
        let mut packages = Vec::new();
        
        if which::which("dpkg").is_ok() {
            let dpkg_packages = list_dpkg_packages().await?;
            packages.extend(dpkg_packages);
        }
        
        if which::which("rpm").is_ok() {
            let rpm_packages = list_rpm_packages().await?;
            packages.extend(rpm_packages);
        }
        
        if which::which("snap").is_ok() {
            let snap_packages = list_snap_packages().await?;
            packages.extend(snap_packages);
        }
        
        if which::which("flatpak").is_ok() {
            let flatpak_packages = list_flatpak_packages().await?;
            packages.extend(flatpak_packages);
        }
        
        Ok(packages)
    }
    
    async fn search_software(&self, query: &str) -> AgentResult<Vec<SoftwarePackage>> {
        search_packages(query).await
    }
    
    async fn install_software(&self, package: &str, silent: bool) -> AgentResult<()> {
        if which::which("apt-get").is_ok() {
            let args = if silent {
                vec!["install", "-y", "-qq", package]
            } else {
                vec!["install", "-y", package]
            };
            
            let output = Command::new("apt-get").args(&args).output()?;
            
            if output.status.success() {
                return Ok(());
            }
        }
        
        if which::which("dnf").is_ok() {
            let args = if silent {
                vec!["install", "-y", "-q", package]
            } else {
                vec!["install", "-y", package]
            };
            
            let output = Command::new("dnf").args(&args).output()?;
            
            if output.status.success() {
                return Ok(());
            }
        }
        
        if which::which("pacman").is_ok() {
            let args = if silent {
                vec!["-S", "--noconfirm", package]
            } else {
                vec!["-S", package]
            };
            
            let output = Command::new("pacman").args(&args).output()?;
            
            if output.status.success() {
                return Ok(());
            }
        }
        
        if which::which("zypper").is_ok() {
            let args = if silent {
                vec!["install", "-y", package]
            } else {
                vec!["install", package]
            };
            
            let output = Command::new("zypper").args(&args).output()?;
            
            if output.status.success() {
                return Ok(());
            }
        }
        
        return Err(AgentError::Platform("No package manager available or installation failed".to_string()));
    }
    
    async fn uninstall_software(&self, package: &str) -> AgentResult<()> {
        if which::which("apt-get").is_ok() {
            Command::new("apt-get")
                .args(&["remove", "-y", package])
                .output()?;
            return Ok(());
        }
        
        if which::which("dnf").is_ok() {
            Command::new("dnf")
                .args(&["remove", "-y", package])
                .output()?;
            return Ok(());
        }
        
        if which::which("pacman").is_ok() {
            Command::new("pacman")
                .args(&["-R", "--noconfirm", package])
                .output()?;
            return Ok(());
        }
        
        if which::which("zypper").is_ok() {
            Command::new("zypper")
                .args(&["remove", "-y", package])
                .output()?;
            return Ok(());
        }
        
        return Err(AgentError::Platform("No package manager available".to_string()));
    }
    
    async fn update_software(&self, package: Option<&str>) -> AgentResult<()> {
        if which::which("apt-get").is_ok() {
            Command::new("apt-get").arg("update").output()?;
            
            let args = match package {
                Some(pkg) => vec!["install", "--only-upgrade", "-y", pkg],
                None => vec!["upgrade", "-y"],
            };
            
            Command::new("apt-get").args(&args).output()?;
            return Ok(());
        }
        
        if which::which("dnf").is_ok() {
            let args = match package {
                Some(pkg) => vec!["upgrade", "-y", pkg],
                None => vec!["upgrade", "-y"],
            };
            
            Command::new("dnf").args(&args).output()?;
            return Ok(());
        }
        
        if which::which("pacman").is_ok() {
            Command::new("pacman").args(&["-Syu", "--noconfirm"]).output()?;
            return Ok(());
        }
        
        if which::which("zypper").is_ok() {
            let args = match package {
                Some(pkg) => vec!["update", pkg],
                None => vec!["update"],
            };
            
            Command::new("zypper").args(&args).output()?;
            return Ok(());
        }
        
        return Err(AgentError::Platform("No package manager available".to_string()));
    }
    
    async fn check_updates(&self) -> AgentResult<Vec<SoftwarePackage>> {
        check_updates_impl().await
    }
    
    fn get_default_browser(&self) -> BrowserType {
        let output = Command::new("xdg-settings")
            .args(&["get", "default-web-browser"])
            .output()
            .ok();
        
        if let Some(out) = output {
            let browser = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
            if browser.contains("chrome") {
                return BrowserType::Chrome;
            } else if browser.contains("firefox") {
                return BrowserType::Firefox;
            } else if browser.contains("edge") {
                return BrowserType::Edge;
            } else if browser.contains("brave") {
                return BrowserType::Brave;
            }
        }
        
        BrowserType::Firefox
    }
    
    async fn launch_browser(&self, browser: BrowserType, url: &str) -> AgentResult<()> {
        if which::which("xdg-open").is_ok() {
            Command::new("xdg-open").arg(url).output()?;
        } else {
            let browser_cmd = match browser {
                BrowserType::Chrome => "google-chrome",
                BrowserType::Firefox => "firefox",
                BrowserType::Safari => return Err(AgentError::NotSupported("Safari not supported on Linux".to_string()).into()),
                BrowserType::Edge => "microsoft-edge",
                BrowserType::Brave => "brave",
            };
            Command::new(browser_cmd).arg(url).output()?;
        }
        
        Ok(())
    }
    
    async fn close_browser(&self, browser: BrowserType) -> AgentResult<()> {
        let process_name = match browser {
            BrowserType::Chrome => "chrome",
            BrowserType::Firefox => "firefox",
            BrowserType::Safari => "safari",
            BrowserType::Edge => "msedge",
            BrowserType::Brave => "brave",
        };
        
        Command::new("pkill").arg(process_name).output()?;
        Ok(())
    }
    
    fn get_program_files_dir(&self) -> String {
        "/usr/bin".to_string()
    }
    
    fn is_elevated(&self) -> bool {
        is_elevated_impl()
    }
}
