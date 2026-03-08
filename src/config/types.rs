use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub server_url: Option<String>,
    pub auth: AuthConfig,
    #[serde(default)]
    pub capabilities: Capabilities,
    #[serde(default)]
    pub metadata_path: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_id: whoami::hostname(),
            server_url: None,
            auth: AuthConfig::default(),
            capabilities: Capabilities::default(),
            metadata_path: Some("config/metadata.json".to_string()),
        }
    }
}

impl AgentConfig {
    pub fn load_metadata(&self) -> Result<CommandMetadata, anyhow::Error> {
        let metadata_path = self.metadata_path.as_deref().unwrap_or("config/metadata.json");
        let path = Path::new(metadata_path);
        
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let metadata: CommandMetadata = serde_json::from_str(&content)?;
            Ok(metadata)
        } else {
            tracing::warn!("Metadata file not found: {:?}", path);
            Ok(CommandMetadata::default())
        }
    }

    pub fn merge_with_metadata(&self) -> Result<Capabilities, anyhow::Error> {
        let metadata = self.load_metadata()?;
        let mut capabilities = self.capabilities.clone();
        
        for (cmd_id, cmd_metadata) in &metadata.commands {
            let new_value = if let Some(_existing_value) = capabilities.commands.get(cmd_id) {
                let config = capabilities.get_command_config(cmd_id).unwrap_or_else(|| CommandConfig {
                    enabled: false,
                    name: String::new(),
                    description: String::new(),
                    category: String::new(),
                    parameters: vec![],
                    returns: None,
                });
                let mut config = config;
                config.merge_with_metadata(cmd_metadata);
                serde_json::json!(config)
            } else {
                let mut config = CommandConfig {
                    enabled: false,
                    name: cmd_metadata.name.clone(),
                    description: cmd_metadata.description.clone(),
                    category: cmd_metadata.category.clone(),
                    parameters: cmd_metadata.parameters.clone(),
                    returns: cmd_metadata.returns.clone(),
                };
                config.merge_with_metadata(cmd_metadata);
                serde_json::json!(config)
            };
            
            capabilities.commands.insert(cmd_id.clone(), new_value);
        }
        
        Ok(capabilities)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandMetadata {
    #[serde(default)]
    pub commands: HashMap<String, CommandMetadataItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadataItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    #[serde(default)]
    pub parameters: Vec<ParameterConfig>,
    #[serde(default)]
    pub returns: Option<ReturnConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnConfig {
    #[serde(rename = "type")]
    pub return_type: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub properties: Option<HashMap<String, ReturnProperty>>,
    #[serde(default)]
    pub items: Option<ReturnItems>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnProperty {
    #[serde(rename = "type")]
    pub prop_type: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnItems {
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(default)]
    pub properties: Option<HashMap<String, ReturnProperty>>,
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
    #[serde(flatten)]
    pub commands: HashMap<String, serde_json::Value>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            commands: Self::default_commands(),
        }
    }
}

impl Capabilities {
    pub fn default_commands() -> HashMap<String, serde_json::Value> {
        let mut commands = HashMap::new();
        
        commands.insert("capabilities".to_string(), serde_json::json!(true));
        commands.insert("system.info".to_string(), serde_json::json!(true));
        commands.insert("system.reboot".to_string(), serde_json::json!(true));
        commands.insert("system.shutdown".to_string(), serde_json::json!(true));
        commands.insert("process.list".to_string(), serde_json::json!(true));
        commands.insert("process.stop".to_string(), serde_json::json!(true));
        commands.insert("software.list".to_string(), serde_json::json!(true));
        commands.insert("software.search".to_string(), serde_json::json!(true));
        commands.insert("software.install".to_string(), serde_json::json!(true));
        commands.insert("software.uninstall".to_string(), serde_json::json!(true));
        commands.insert("env.list".to_string(), serde_json::json!(true));
        commands.insert("env.get".to_string(), serde_json::json!(true));
        commands.insert("env.set".to_string(), serde_json::json!(true));
        commands.insert("env.delete".to_string(), serde_json::json!(true));
        commands.insert("file.list".to_string(), serde_json::json!(true));
        commands.insert("file.read".to_string(), serde_json::json!(true));
        commands.insert("file.write".to_string(), serde_json::json!(true));
        commands.insert("file.delete".to_string(), serde_json::json!(true));
        commands.insert("file.create_dir".to_string(), serde_json::json!(true));
        commands.insert("file.copy".to_string(), serde_json::json!(true));
        commands.insert("file.move".to_string(), serde_json::json!(true));
        commands.insert("file.download".to_string(), serde_json::json!(true));
        commands.insert("config.get".to_string(), serde_json::json!(true));
        commands.insert("config.set".to_string(), serde_json::json!(true));
        commands.insert("shell.execute".to_string(), serde_json::json!(true));
        
        commands
    }

    pub fn is_enabled(&self, command_id: &str) -> bool {
        match self.commands.get(command_id) {
            Some(value) => {
                if let Some(b) = value.as_bool() {
                    b
                } else if let Some(obj) = value.as_object() {
                    obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false)
                } else {
                    false
                }
            }
            None => false,
        }
    }

    pub fn get_command_config(&self, command_id: &str) -> Option<CommandConfig> {
        self.commands.get(command_id).map(|value| {
            if let Some(obj) = value.as_object() {
                CommandConfig {
                    enabled: obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false),
                    name: obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    description: obj.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    category: obj.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    parameters: obj.get("parameters")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default(),
                    returns: obj.get("returns")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                }
            } else if let Some(b) = value.as_bool() {
                CommandConfig {
                    enabled: b,
                    name: String::new(),
                    description: String::new(),
                    category: String::new(),
                    parameters: vec![],
                    returns: None,
                }
            } else {
                CommandConfig {
                    enabled: false,
                    name: String::new(),
                    description: String::new(),
                    category: String::new(),
                    parameters: vec![],
                    returns: None,
                }
            }
        })
    }

    pub fn get_enabled_commands(&self) -> Vec<(String, CommandConfig)> {
        self.commands
            .iter()
            .filter_map(|(id, _value)| {
                let config = self.get_command_config(id)?;
                if config.enabled {
                    Some((id.clone(), config))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn to_capabilities_list(&self) -> serde_json::Value {
        let list: Vec<serde_json::Value> = self.commands
            .iter()
            .map(|(id, _value)| {
                let config = self.get_command_config(id).unwrap();
                let mut cap = serde_json::json!({
                    "id": id,
                    "name": config.name,
                    "description": config.description,
                    "category": config.category,
                    "parameters": config.parameters,
                    "enabled": config.enabled
                });
                if let Some(returns) = &config.returns {
                    cap["returns"] = serde_json::json!(returns);
                }
                cap
            })
            .collect();
        serde_json::json!({ "capabilities": list })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    pub enabled: bool,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ParameterConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnConfig>,
}

impl CommandConfig {
    pub fn merge_with_metadata(&mut self, metadata: &CommandMetadataItem) {
        if self.name.is_empty() {
            self.name = metadata.name.clone();
        }
        if self.description.is_empty() {
            self.description = metadata.description.clone();
        }
        if self.category.is_empty() {
            self.category = metadata.category.clone();
        }
        if self.parameters.is_empty() {
            self.parameters = metadata.parameters.clone();
        }
        if self.returns.is_none() {
            self.returns = metadata.returns.clone();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
    pub description: String,
}
