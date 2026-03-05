pub const DEFAULT_PORT: u16 = 8443;
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";
pub const DEFAULT_LOG_LEVEL: &str = "info";
pub const DEFAULT_TOKEN_TTL_SECONDS: u64 = 3600;
pub const DEFAULT_MAX_SESSION_DURATION_MINUTES: u64 = 120;
pub const DEFAULT_MAX_LOG_SIZE_MB: u64 = 100;
pub const DEFAULT_LOG_RETENTION_DAYS: u32 = 30;

pub const AGENT_CONFIG_ENV: &str = "AGENT_CONFIG";
pub const AGENT_LOG_LEVEL_ENV: &str = "AGENT_LOG_LEVEL";
pub const AGENT_SECRET_KEY_ENV: &str = "AGENT_SECRET_KEY";

pub const ALLOWED_COMMANDS: &[&str] = &[
    "code",
    "notepad",
    "calc",
    "mspaint",
    "powershell",
    "cmd",
    "winget",
    "choco",
    "msiexec",
    "brew",
    "open",
    "osascript",
    "defaults",
    "apt",
    "apt-get",
    "dnf",
    "yum",
    "pacman",
    "zypper",
    "snap",
    "flatpak",
    "chrome",
    "firefox",
    "msedge",
    "brave",
    "google-chrome",
    "microsoft-edge",
];

pub const BLOCKED_COMMANDS: &[&str] = &[
    "format",
    "del /s",
    "shutdown /s",
    "rd /s",
    "rm -rf /",
    "mkfs",
    "dd if=",
];

pub fn get_default_config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("cross-platform-agent")
        .join("agent.yml")
}

pub fn get_default_log_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("log")
        .join("agent.log")
}
