use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::AgentResult;
use crate::platform::common::{
    list_processes_impl,
    read_file_with_options_impl, read_file_impl,
    write_file_with_options_impl, write_file_impl,
    delete_file_impl, create_dir_impl, copy_file_impl, move_file_impl,
    list_dir_impl, download_file_impl, get_user_dir_impl,
    get_hostname, get_username
};
use crate::platform::types::*;

#[async_trait]
pub trait Platform: Send + Sync {
    fn name(&self) -> &'static str;
    
    fn version(&self) -> String {
        std::env::consts::OS.to_string()
    }
    
    async fn get_system_info(&self) -> AgentResult<SystemInfo>;
    
    async fn get_hostname(&self) -> AgentResult<String> {
        get_hostname()
    }
    
    async fn get_username(&self) -> AgentResult<String> {
        get_username()
    }
    
    async fn stop_process(&self, pid: u32, force: bool) -> AgentResult<()>;
    
    async fn list_processes(&self) -> AgentResult<Vec<ProcessInfo>>{
        list_processes_impl().await
    }
    
    async fn get_env_var(&self, name: &str, scope: EnvScope) -> AgentResult<Option<String>>;
    
    async fn set_env_var(&self, name: &str, value: &str, scope: EnvScope) -> AgentResult<()>;
    
    async fn delete_env_var(&self, name: &str, scope: EnvScope) -> AgentResult<()>;
    
    async fn list_env_vars(&self, scope: EnvScope) -> AgentResult<HashMap<String, String>>;
    
    async fn get_config(&self, path: &str) -> AgentResult<serde_json::Value>;
    
    async fn set_config(&self, path: &str, value: serde_json::Value) -> AgentResult<()>;
    
    async fn reboot(&self) -> AgentResult<()> {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("shutdown")
                .args(&["/r", "/t", "0"])
                .output()?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("shutdown")
                .args(&["-r", "now"])
                .output()?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("shutdown")
                .args(&["-r", "now"])
                .output()?;
        }
        Ok(())
    }
    
    async fn shutdown(&self) -> AgentResult<()> {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("shutdown")
                .args(&["/s", "/t", "0"])
                .output()?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("shutdown")
                .args(&["-h", "now"])
                .output()?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("shutdown")
                .args(&["-h", "now"])
                .output()?;
        }
        Ok(())
    }
    
    async fn list_software(&self) -> AgentResult<Vec<SoftwarePackage>>;
    
    async fn search_software(&self, query: &str) -> AgentResult<Vec<SoftwarePackage>>;
    
    async fn install_software(&self, package: &str, silent: bool) -> AgentResult<()>;
    
    async fn uninstall_software(&self, package: &str) -> AgentResult<()>;
    
    async fn read_file(&self, path: &str) -> AgentResult<String> {
        read_file_impl(path).await
    }
    
    async fn read_file_with_options(&self, path: &str, options: FileReadOptions) -> AgentResult<FileReadResult> {
        read_file_with_options_impl(path, options).await
    }
    
    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()> {
        write_file_impl(path, content).await
    }
    
    async fn write_file_with_options(&self, path: &str, content: &str, options: FileWriteOptions) -> AgentResult<FileWriteResult> {
        write_file_with_options_impl(path, content, options).await
    }
    
    async fn delete_file(&self, path: &str) -> AgentResult<()> {
        delete_file_impl(path).await
    }
    
    async fn list_dir(&self, path: &str) -> AgentResult<Vec<FileInfo>> {
        list_dir_impl(path).await
    }
    
    async fn create_dir(&self, path: &str, recursive: bool) -> AgentResult<()> {
        create_dir_impl(path, recursive).await
    }
    
    async fn copy_file(&self, src: &str, dst: &str) -> AgentResult<()> {
        copy_file_impl(src, dst).await
    }
    
    async fn move_file(&self, src: &str, dst: &str) -> AgentResult<()> {
        move_file_impl(src, dst).await
    }
    
    async fn download_file(&self, url: &str, dest: &str) -> AgentResult<String> {
        download_file_impl(url, dest).await
    }
    
    fn get_user_dir(&self, dir_type: UserDirType) -> String {
        get_user_dir_impl(dir_type)
    }
    
    fn get_program_files_dir(&self) -> String {
        #[cfg(target_os = "windows")]
        {
            std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".to_string())
        }
        #[cfg(target_os = "macos")]
        {
            "/Applications".to_string()
        }
        #[cfg(target_os = "linux")]
        {
            "/usr/bin".to_string()
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            String::new()
        }
    }
    
    async fn shell_execute(&self, command: &str, timeout_secs: u64) -> AgentResult<serde_json::Value> {
        use std::time::Duration;
        use std::process::Stdio;
        
        #[cfg(target_os = "windows")]
        let platform_name = "windows";
        #[cfg(target_os = "macos")]
        let platform_name = "macos";
        #[cfg(target_os = "linux")]
        let platform_name = "linux";
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let platform_name = "unknown";

        let output = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            async {
                #[cfg(target_os = "windows")]
                {
                    tokio::process::Command::new("cmd")
                        .args(&["/c", command])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .await
                }
                #[cfg(not(target_os = "windows"))]
                {
                    tokio::process::Command::new("sh")
                        .args(&["-c", command])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .await
                }
            }
        ).await.map_err(|_| anyhow::anyhow!("Command timed out after {} seconds", timeout_secs))??;

        Ok(serde_json::json!({
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
            "exit_code": output.status.code().unwrap_or(-1),
            "platform": platform_name
        }))
    }
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
