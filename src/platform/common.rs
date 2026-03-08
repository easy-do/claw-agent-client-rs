use crate::error::AgentResult;
use crate::platform::types::{CpuInfo, MemoryInfo, SystemInfo, BYTES_PER_GB, FileInfo, FileReadOptions, FileReadResult, FileWriteOptions, FileWriteResult, UserDirType};
use crate::ProcessInfo;
use sysinfo::System;
use std::io::Write;

pub fn get_hostname() -> AgentResult<String> {
    Ok(whoami::hostname())
}

pub fn get_username() -> AgentResult<String> {
    Ok(whoami::username())
}

pub fn build_system_info(
    hostname: String,
    username: String,
    os_type: String,
    os_version: String,
    uptime_secs: u64,
    memory: MemoryInfo,
    cpu: CpuInfo,
) -> SystemInfo {
    SystemInfo {
        hostname,
        os_type,
        os_version,
        arch: std::env::consts::ARCH.to_string(),
        username,
        uptime_secs,
        total_memory_gb: memory.total_gb,
        available_memory_gb: memory.available_gb,
        cpu_count: cpu.count,
        cpu_usage_percent: cpu.usage_percent,
    }
}

pub async fn get_memory_info() -> AgentResult<MemoryInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let total = sys.total_memory();
    let available = sys.available_memory();
    let used = total - available;
    
    Ok(MemoryInfo {
        total_gb: total as f64 / BYTES_PER_GB,
        available_gb: available as f64 / BYTES_PER_GB,
        used_gb: used as f64 / BYTES_PER_GB,
        usage_percent: (used as f64 / total as f64 * 100.0) as f32,
    })
}

pub async fn get_cpu_info() -> AgentResult<CpuInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let cpus = sys.cpus();
    let count = cpus.len();
    let usage: f32 = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / count as f32;
    
    let brand = cpus.first().map(|c| c.brand().to_string()).unwrap_or_default();
    let frequency = cpus.first().map(|c| c.frequency()).unwrap_or(0);
    
    Ok(CpuInfo {
        count,
        usage_percent: usage,
        brand,
        frequency_mhz: frequency,
    })
}

pub async fn list_processes_impl() -> AgentResult<Vec<ProcessInfo>> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let processes: Vec<ProcessInfo> = sys.processes()
        .iter()
        .map(|(pid, process)| {
            let cmd: Vec<String> = process.cmd().iter().map(|s| s.to_string()).collect();

            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cmd: cmd.join(" "),
                cpu_percent: process.cpu_usage(),
                memory_mb: process.memory() / 1024 / 1024,
                status: format!("{:?}", process.status()),
                start_time: Some(process.start_time()),
                user: process.user_id().map(|u| u.to_string()),
            }
        })
        .collect();

    Ok(processes)
}

pub async fn read_file_with_options_impl(path: &str, options: FileReadOptions) -> AgentResult<FileReadResult> {
    use crate::error::AgentError;
    use crate::platform::types::{DEFAULT_MAX_FILE_SIZE, TEXT_DETECTION_BYTES};
    
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
    
    let is_text = bytes.iter().take(TEXT_DETECTION_BYTES).all(|&b| {
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

pub async fn read_file_impl(path: &str) -> AgentResult<String> {
    let options = FileReadOptions::default();
    let result = read_file_with_options_impl(path, options).await?;
    Ok(result.content)
}

pub async fn write_file_with_options_impl(path: &str, content: &str, options: FileWriteOptions) -> AgentResult<FileWriteResult> {
    if options.append {
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

pub async fn write_file_impl(path: &str, content: &str) -> AgentResult<()> {
    let options = FileWriteOptions::default();
    write_file_with_options_impl(path, content, options).await?;
    Ok(())
}

pub async fn delete_file_impl(path: &str) -> AgentResult<()> {
    std::fs::remove_file(path)?;
    Ok(())
}

pub async fn create_dir_impl(path: &str, recursive: bool) -> AgentResult<()> {
    if recursive {
        std::fs::create_dir_all(path)?;
    } else {
        std::fs::create_dir(path)?;
    }
    Ok(())
}

pub async fn copy_file_impl(src: &str, dst: &str) -> AgentResult<()> {
    std::fs::copy(src, dst)?;
    Ok(())
}

pub async fn move_file_impl(src: &str, dst: &str) -> AgentResult<()> {
    std::fs::rename(src, dst)?;
    Ok(())
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
            permissions: Some(if metadata.permissions().readonly() { "readonly".to_string() } else { "writable".to_string() }),
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

pub fn get_user_dir_impl(dir_type: UserDirType) -> String {
    let home = dirs::home_dir().unwrap_or_default();
    
    match dir_type {
        UserDirType::Home => home.to_string_lossy().to_string(),
        UserDirType::Desktop => home.join("Desktop").to_string_lossy().to_string(),
        UserDirType::Documents => home.join("Documents").to_string_lossy().to_string(),
        UserDirType::Downloads => home.join("Downloads").to_string_lossy().to_string(),
        UserDirType::Pictures => home.join("Pictures").to_string_lossy().to_string(),
        UserDirType::Music => home.join("Music").to_string_lossy().to_string(),
        UserDirType::Videos => home.join("Videos").to_string_lossy().to_string(),
        UserDirType::Temp => std::env::temp_dir().to_string_lossy().to_string(),
        UserDirType::Cache => home.join(".cache").to_string_lossy().to_string(),
        UserDirType::Config => home.join(".config").to_string_lossy().to_string(),
    }
}
