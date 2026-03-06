pub mod system;
pub mod process;
pub mod env;
pub mod registry;
pub mod wmi;
pub mod security;

pub use system::*;
pub use process::*;
pub use env::*;
pub use registry::*;
pub use wmi::*;
pub use security::*;

use crate::platform::traits::*;
use crate::platform::types::*;
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
        
        Ok(SystemInfo {
            hostname,
            os_type: "Windows".to_string(),
            os_version,
            arch: std::env::consts::ARCH.to_string(),
            username,
            uptime_secs: uptime,
            total_memory_gb: memory.total_gb,
            available_memory_gb: memory.available_gb,
            cpu_count: cpu.count,
            cpu_usage_percent: cpu.usage_percent,
        })
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
    
    async fn list_processes(&self) -> AgentResult<Vec<ProcessInfo>> {
        list_processes_impl().await
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
    
    async fn reboot(&self) -> AgentResult<()> {
        Command::new("shutdown")
            .args(&["/r", "/t", "0"])
            .output()?;
        Ok(())
    }
    
    async fn shutdown(&self) -> AgentResult<()> {
        Command::new("shutdown")
            .args(&["/s", "/t", "0"])
            .output()?;
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
    
    async fn read_file(&self, path: &str) -> AgentResult<String> {
        let options = FileReadOptions::default();
        let result = self.read_file_with_options(path, options).await?;
        Ok(result.content)
    }
    
    async fn read_file_with_options(&self, path: &str, options: FileReadOptions) -> AgentResult<FileReadResult> {
        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();
        
        let max_size = options.max_size.unwrap_or(DEFAULT_MAX_FILE_SIZE);
        
        if file_size > max_size {
            return Err(AgentError::Io(format!(
                "File size {} bytes exceeds maximum allowed size {} bytes",
                file_size, max_size
            )).into());
        }
        
        let bytes = std::fs::read(path)?;
        let size_bytes = bytes.len() as u64;
        
        let is_text = bytes.iter().take(8000).all(|&b| {
            b < 128 || b == b'\n' || b == b'\r' || b == b'\t'
        });
        
        if is_text {
            let content = String::from_utf8_lossy(&bytes).to_string();
            Ok(FileReadResult {
                content,
                is_base64: false,
                size_bytes,
                truncated: false,
            })
        } else {
            use base64::Engine;
            let content = base64::engine::general_purpose::STANDARD.encode(&bytes);
            Ok(FileReadResult {
                content,
                is_base64: true,
                size_bytes,
                truncated: false,
            })
        }
    }
    
    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()> {
        let options = FileWriteOptions::default();
        self.write_file_with_options(path, content, options).await?;
        Ok(())
    }
    
    async fn write_file_with_options(&self, path: &str, content: &str, options: FileWriteOptions) -> AgentResult<FileWriteResult> {
        if options.append {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            file.write_all(content.as_bytes())?;
        } else {
            std::fs::write(path, content)?;
        }
        
        let bytes_written = content.len() as u64;
        Ok(FileWriteResult { bytes_written })
    }
    
    async fn delete_file(&self, path: &str) -> AgentResult<()> {
        std::fs::remove_file(path)?;
        Ok(())
    }
    
    async fn list_dir(&self, path: &str) -> AgentResult<Vec<FileInfo>> {
        list_dir_impl(path).await
    }
    
    async fn create_dir(&self, path: &str, recursive: bool) -> AgentResult<()> {
        if recursive {
            std::fs::create_dir_all(path)?;
        } else {
            std::fs::create_dir(path)?;
        }
        Ok(())
    }
    
    async fn copy_file(&self, src: &str, dst: &str) -> AgentResult<()> {
        std::fs::copy(src, dst)?;
        Ok(())
    }
    
    async fn move_file(&self, src: &str, dst: &str) -> AgentResult<()> {
        std::fs::rename(src, dst)?;
        Ok(())
    }
    
    async fn download_file(&self, url: &str, dest: &str) -> AgentResult<String> {
        download_file_impl(url, dest).await
    }
    
    fn get_user_dir(&self, dir_type: UserDirType) -> String {
        get_user_dir_impl(dir_type)
    }
    
    fn get_program_files_dir(&self) -> String {
        std::env::var("ProgramFiles")
            .unwrap_or_else(|_| r"C:\Program Files".to_string())
    }
    
    fn is_elevated(&self) -> bool {
        is_elevated_impl()
    }
    
    async fn request_elevation(&self) -> AgentResult<bool> {
        Ok(false)
    }

    async fn shell_execute(&self, command: &str, timeout_secs: u64) -> AgentResult<serde_json::Value> {
        let output = std::process::Command::new("cmd")
            .args(&["/c", command])
            .output()?;

        Ok(serde_json::json!({
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
            "exit_code": output.status.code().unwrap_or(-1),
            "platform": "windows"
        }))
    }
}
