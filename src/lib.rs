pub mod utils;
pub mod config;
pub mod auth;
pub mod client;

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
pub mod platform;

pub use config::{load_config, AgentConfig, init_logging};
pub use client::run_client;

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
pub use platform::traits::{Platform, get_platform, platform_name};

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
pub use platform::types::{
    SystemInfo, ProcessInfo, SoftwarePackage,
    BrowserType, UserDirType, FileInfo, EnvScope,
};

pub mod error {
    use thiserror::Error;
    use serde::{Deserialize, Serialize};

    pub type AgentResult<T> = Result<T, AgentError>;

    #[derive(Debug, Error, Serialize, Deserialize)]
    pub enum AgentError {
        #[error("IO error: {0}")]
        Io(String),
        
        #[error("JSON error: {0}")]
        Json(String),
        
        #[error("Configuration error: {0}")]
        Config(String),
        
        #[error("Authentication error: {0}")]
        Auth(String),
        
        #[error("Permission denied: {0}")]
        Permission(String),
        
        #[error("Platform error: {0}")]
        Platform(String),
        
        #[error("Network error: {0}")]
        Network(String),
        
        #[error("Command execution error: {0}")]
        Command(String),
        
        #[error("Not supported: {0}")]
        NotSupported(String),
        
        #[error("Invalid request: {0}")]
        InvalidRequest(String),
        
        #[error("Internal error: {0}")]
        Internal(String),
    }

    impl AgentError {
        pub fn code(&self) -> i32 {
            match self {
                AgentError::Io(_) => 1001,
                AgentError::Json(_) => 1002,
                AgentError::Config(_) => 1003,
                AgentError::Auth(_) => 1004,
                AgentError::Permission(_) => 1005,
                AgentError::Platform(_) => 1006,
                AgentError::Network(_) => 1007,
                AgentError::Command(_) => 1008,
                AgentError::NotSupported(_) => 1009,
                AgentError::InvalidRequest(_) => 1010,
                AgentError::Internal(_) => 1011,
            }
        }
    }

    impl From<std::io::Error> for AgentError {
        fn from(err: std::io::Error) -> Self {
            AgentError::Io(err.to_string())
        }
    }

    impl From<std::num::ParseIntError> for AgentError {
        fn from(err: std::num::ParseIntError) -> Self {
            AgentError::Command(err.to_string())
        }
    }

    impl From<serde_json::Error> for AgentError {
        fn from(err: serde_json::Error) -> Self {
            AgentError::Json(err.to_string())
        }
    }

    impl From<anyhow::Error> for AgentError {
        fn from(err: anyhow::Error) -> Self {
            AgentError::Internal(err.to_string())
        }
    }

    impl From<reqwest::Error> for AgentError {
        fn from(err: reqwest::Error) -> Self {
            AgentError::Network(err.to_string())
        }
    }

    impl From<serde_yaml::Error> for AgentError {
        fn from(err: serde_yaml::Error) -> Self {
            AgentError::Json(err.to_string())
        }
    }
}
