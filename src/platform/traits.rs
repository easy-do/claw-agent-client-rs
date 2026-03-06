use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::AgentResult;
use crate::platform::types::*;

#[async_trait]
pub trait Platform: Send + Sync {
    fn name(&self) -> &'static str;
    
    fn version(&self) -> String {
        std::env::consts::OS.to_string()
    }
    
    async fn get_system_info(&self) -> AgentResult<SystemInfo>;
    
    async fn get_hostname(&self) -> AgentResult<String> {
        Ok(whoami::hostname())
    }
    
    async fn get_username(&self) -> AgentResult<String> {
        Ok(whoami::username())
    }
    
    async fn start_process(
        &self,
        program: &str,
        args: &[String],
        elevated: bool,
        working_dir: Option<&str>,
    ) -> AgentResult<u32>;
    
    async fn stop_process(&self, pid: u32, force: bool) -> AgentResult<()>;
    
    async fn list_processes(&self) -> AgentResult<Vec<ProcessInfo>>;
    
    async fn find_process(&self, name: &str) -> AgentResult<Option<ProcessInfo>>;
    
    async fn get_process_info(&self, pid: u32) -> AgentResult<Option<ProcessInfo>>;
    
    async fn get_env_var(&self, name: &str, scope: EnvScope) -> AgentResult<Option<String>>;
    
    async fn set_env_var(&self, name: &str, value: &str, scope: EnvScope) -> AgentResult<()>;
    
    async fn delete_env_var(&self, name: &str, scope: EnvScope) -> AgentResult<()>;
    
    async fn list_env_vars(&self, scope: EnvScope) -> AgentResult<HashMap<String, String>>;
    
    async fn get_config(&self, path: &str) -> AgentResult<serde_json::Value>;
    
    async fn set_config(&self, path: &str, value: serde_json::Value) -> AgentResult<()>;
    
    async fn toggle_feature(&self, feature: &str, enabled: bool) -> AgentResult<()>;
    
    async fn reboot(&self) -> AgentResult<()>;
    
    async fn shutdown(&self) -> AgentResult<()>;
    
    async fn list_software(&self) -> AgentResult<Vec<SoftwarePackage>>;
    
    async fn search_software(&self, query: &str) -> AgentResult<Vec<SoftwarePackage>>;
    
    async fn install_software(&self, package: &str, silent: bool) -> AgentResult<()>;
    
    async fn uninstall_software(&self, package: &str) -> AgentResult<()>;
    
    async fn update_software(&self, package: Option<&str>) -> AgentResult<()>;
    
    async fn check_updates(&self) -> AgentResult<Vec<SoftwarePackage>>;
    
    fn get_default_browser(&self) -> BrowserType;
    
    async fn launch_browser(&self, browser: BrowserType, url: &str) -> AgentResult<()>;
    
    async fn close_browser(&self, browser: BrowserType) -> AgentResult<()>;
    
    async fn read_file(&self, path: &str) -> AgentResult<String>;
    
    async fn read_file_with_options(&self, path: &str, options: FileReadOptions) -> AgentResult<FileReadResult>;
    
    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()>;
    
    async fn write_file_with_options(&self, path: &str, content: &str, options: FileWriteOptions) -> AgentResult<FileWriteResult>;
    
    async fn delete_file(&self, path: &str) -> AgentResult<()>;
    
    async fn list_dir(&self, path: &str) -> AgentResult<Vec<FileInfo>>;
    
    async fn create_dir(&self, path: &str, recursive: bool) -> AgentResult<()>;
    
    async fn copy_file(&self, src: &str, dst: &str) -> AgentResult<()>;
    
    async fn move_file(&self, src: &str, dst: &str) -> AgentResult<()>;
    
    async fn download_file(&self, url: &str, dest: &str) -> AgentResult<String>;
    
    fn get_user_dir(&self, dir_type: UserDirType) -> String;
    
    fn get_program_files_dir(&self) -> String;
    
    fn is_elevated(&self) -> bool;
    
    async fn request_elevation(&self) -> AgentResult<bool>;

    async fn shell_execute(&self, command: &str, timeout_secs: u64) -> AgentResult<serde_json::Value>;
}

pub fn get_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "windows")]
    {
        Box::new(crate::platform::windows::WindowsPlatform::new())
    }

    #[cfg(target_os = "macos")]
    {
        Box::new(crate::platform::macos::MacOSPlatform::new())
    }

    #[cfg(target_os = "linux")]
    {
        Box::new(crate::platform::linux::LinuxPlatform::new())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        panic!("Unsupported operating system: {}", std::env::consts::OS);
    }
}

pub fn platform_name() -> &'static str {
    #[cfg(target_os = "windows")]
    { "windows" }
    
    #[cfg(target_os = "macos")]
    { "macos" }
    
    #[cfg(target_os = "linux")]
    { "linux" }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    { "unknown" }
}
