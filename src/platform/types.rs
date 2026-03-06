use serde::{Deserialize, Serialize};

pub const DEFAULT_MAX_FILE_SIZE: u64 = 10 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadOptions {
    pub max_size: Option<u64>,
    pub encoding: Option<String>,
}

impl Default for FileReadOptions {
    fn default() -> Self {
        Self {
            max_size: Some(DEFAULT_MAX_FILE_SIZE),
            encoding: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadResult {
    pub content: String,
    pub is_base64: bool,
    pub size_bytes: u64,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteOptions {
    pub append: bool,
    pub encoding: Option<String>,
}

impl Default for FileWriteOptions {
    fn default() -> Self {
        Self {
            append: false,
            encoding: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteResult {
    pub bytes_written: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_type: String,
    pub os_version: String,
    pub arch: String,
    pub username: String,
    pub uptime_secs: u64,
    pub total_memory_gb: f64,
    pub available_memory_gb: f64,
    pub cpu_count: usize,
    pub cpu_usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmd: String,
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub status: String,
    pub start_time: Option<u64>,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvScope {
    User,
    System,
    Session,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwarePackage {
    pub name: String,
    pub version: String,
    pub publisher: Option<String>,
    pub install_path: Option<String>,
    pub install_date: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Brave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserDirType {
    Home,
    Desktop,
    Documents,
    Downloads,
    Pictures,
    Music,
    Videos,
    Temp,
    Cache,
    Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub is_dir: bool,
    pub is_file: bool,
    pub modified: Option<u64>,
    pub created: Option<u64>,
    pub permissions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_gb: f64,
    pub available_gb: f64,
    pub used_gb: f64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub count: usize,
    pub usage_percent: f32,
    pub brand: String,
    pub frequency_mhz: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsVersion {
    pub major: u32,
    pub minor: u32,
    pub build: Option<u32>,
    pub name: String,
}
