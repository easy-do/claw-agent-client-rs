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
        let hostname = whoami::hostname();
        let username = whoami::username();
        
        let os_version = get_os_version().await?;
        let uptime = get_uptime().await?;
        let memory = get_memory_info().await?;
        let cpu = get_cpu_info().await?;
        
        Ok(SystemInfo {
            hostname,
            os_type: "macOS".to_string(),
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
    
    async fn reboot(&self) -> AgentResult<()> {
        Command::new("shutdown")
            .args(&["-r", "now"])
            .output()?;
        Ok(())
    }
    
    async fn shutdown(&self) -> AgentResult<()> {
        Command::new("shutdown")
            .args(&["-h", "now"])
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
        let app_name = match browser {
            BrowserType::Chrome => "Google Chrome",
            BrowserType::Firefox => "Firefox",
            BrowserType::Safari => "Safari",
            BrowserType::Edge => "Microsoft Edge",
            BrowserType::Brave => "Brave Browser",
        };
        
        Command::new("open")
            .args(&["-a", app_name, url])
            .output()?;
        
        Ok(())
    }
    
    async fn close_browser(&self, browser: BrowserType) -> AgentResult<()> {
        let app_name = match browser {
            BrowserType::Chrome => "Google Chrome",
            BrowserType::Firefox => "Firefox",
            BrowserType::Safari => "Safari",
            BrowserType::Edge => "Microsoft Edge",
            BrowserType::Brave => "Brave Browser",
        };
        
        Command::new("killall").arg(app_name).output()?;
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
        "/Applications".to_string()
    }
    
    fn is_elevated(&self) -> bool {
        is_elevated_impl()
    }
    
    async fn request_elevation(&self) -> AgentResult<bool> {
        Ok(false)
    }

    async fn shell_execute(&self, command: &str, timeout_secs: u64) -> AgentResult<serde_json::Value> {
        let output = std::process::Command::new("sh")
            .args(&["-c", command])
            .output()?;

        Ok(serde_json::json!({
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
            "exit_code": output.status.code().unwrap_or(-1),
            "platform": "macos"
        }))
    }
}
