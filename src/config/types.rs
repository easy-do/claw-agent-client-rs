use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub server_url: Option<String>,
    pub auth: AuthConfig,
    pub capabilities: Capabilities,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_id: whoami::hostname(),
            server_url: None,
            auth: AuthConfig::default(),
            capabilities: Capabilities::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub token: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub system_info: bool,
    pub process_control: bool,
    pub env_management: bool,
    pub software_install: bool,
    pub file_operations: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            system_info: true,
            process_control: true,
            env_management: true,
            software_install: true,
            file_operations: true,
        }
    }
}
