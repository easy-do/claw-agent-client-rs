use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[serde(flatten)]
    pub commands: HashMap<String, CommandConfig>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            commands: Self::default_commands(),
        }
    }
}

impl Capabilities {
    pub fn default_commands() -> HashMap<String, CommandConfig> {
        let mut commands = HashMap::new();
        
        commands.insert("capabilities".to_string(), CommandConfig {
            enabled: true,
            name: "Get Capabilities".to_string(),
            description: "获取客户端支持的所有命令列表".to_string(),
            category: "system".to_string(),
            parameters: vec![],
        });
        
        commands.insert("system.info".to_string(), CommandConfig {
            enabled: true,
            name: "Get System Info".to_string(),
            description: "获取系统信息，包括主机名、操作系统版本、架构、用户名、运行时间、内存和CPU信息".to_string(),
            category: "system".to_string(),
            parameters: vec![],
        });
        
        commands.insert("process.list".to_string(), CommandConfig {
            enabled: true,
            name: "List Processes".to_string(),
            description: "获取当前运行的进程列表".to_string(),
            category: "process".to_string(),
            parameters: vec![],
        });
        
        commands.insert("process.stop".to_string(), CommandConfig {
            enabled: true,
            name: "Stop Process".to_string(),
            description: "停止指定PID的进程".to_string(),
            category: "process".to_string(),
            parameters: vec![
                ParameterConfig { name: "pid".to_string(), param_type: "number".to_string(), required: true, description: "进程ID".to_string() },
                ParameterConfig { name: "force".to_string(), param_type: "boolean".to_string(), required: false, description: "是否强制终止".to_string() },
            ],
        });
        
        commands.insert("software.list".to_string(), CommandConfig {
            enabled: true,
            name: "List Software".to_string(),
            description: "获取已安装的软件列表".to_string(),
            category: "software".to_string(),
            parameters: vec![],
        });
        
        commands.insert("software.search".to_string(), CommandConfig {
            enabled: true,
            name: "Search Software".to_string(),
            description: "搜索可安装的软件包".to_string(),
            category: "software".to_string(),
            parameters: vec![
                ParameterConfig { name: "query".to_string(), param_type: "string".to_string(), required: true, description: "搜索关键词".to_string() },
            ],
        });
        
        commands.insert("software.install".to_string(), CommandConfig {
            enabled: true,
            name: "Install Software".to_string(),
            description: "安装指定的软件包".to_string(),
            category: "software".to_string(),
            parameters: vec![
                ParameterConfig { name: "package".to_string(), param_type: "string".to_string(), required: true, description: "软件包名称或ID".to_string() },
                ParameterConfig { name: "silent".to_string(), param_type: "boolean".to_string(), required: false, description: "是否静默安装".to_string() },
            ],
        });
        
        commands.insert("software.uninstall".to_string(), CommandConfig {
            enabled: true,
            name: "Uninstall Software".to_string(),
            description: "卸载指定的软件包".to_string(),
            category: "software".to_string(),
            parameters: vec![
                ParameterConfig { name: "package".to_string(), param_type: "string".to_string(), required: true, description: "软件包名称或ID".to_string() },
            ],
        });
        
        commands.insert("env.list".to_string(), CommandConfig {
            enabled: true,
            name: "List Environment Variables".to_string(),
            description: "获取环境变量列表".to_string(),
            category: "environment".to_string(),
            parameters: vec![
                ParameterConfig { name: "scope".to_string(), param_type: "string".to_string(), required: false, description: "作用域: user, system, session".to_string() },
            ],
        });
        
        commands.insert("env.get".to_string(), CommandConfig {
            enabled: true,
            name: "Get Environment Variable".to_string(),
            description: "获取指定环境变量的值".to_string(),
            category: "environment".to_string(),
            parameters: vec![
                ParameterConfig { name: "name".to_string(), param_type: "string".to_string(), required: true, description: "变量名".to_string() },
                ParameterConfig { name: "scope".to_string(), param_type: "string".to_string(), required: false, description: "作用域: user, system, session".to_string() },
            ],
        });
        
        commands.insert("env.set".to_string(), CommandConfig {
            enabled: true,
            name: "Set Environment Variable".to_string(),
            description: "设置环境变量".to_string(),
            category: "environment".to_string(),
            parameters: vec![
                ParameterConfig { name: "name".to_string(), param_type: "string".to_string(), required: true, description: "变量名".to_string() },
                ParameterConfig { name: "value".to_string(), param_type: "string".to_string(), required: true, description: "变量值".to_string() },
                ParameterConfig { name: "scope".to_string(), param_type: "string".to_string(), required: false, description: "作用域: user, system, session".to_string() },
            ],
        });
        
        commands.insert("env.delete".to_string(), CommandConfig {
            enabled: true,
            name: "Delete Environment Variable".to_string(),
            description: "删除指定的环境变量".to_string(),
            category: "environment".to_string(),
            parameters: vec![
                ParameterConfig { name: "name".to_string(), param_type: "string".to_string(), required: true, description: "变量名".to_string() },
                ParameterConfig { name: "scope".to_string(), param_type: "string".to_string(), required: false, description: "作用域: user, system, session".to_string() },
            ],
        });
        
        commands.insert("file.list".to_string(), CommandConfig {
            enabled: true,
            name: "List Directory".to_string(),
            description: "列出指定目录的内容".to_string(),
            category: "file".to_string(),
            parameters: vec![
                ParameterConfig { name: "path".to_string(), param_type: "string".to_string(), required: false, description: "目录路径".to_string() },
            ],
        });
        
        commands.insert("file.read".to_string(), CommandConfig {
            enabled: true,
            name: "Read File".to_string(),
            description: "读取文件内容".to_string(),
            category: "file".to_string(),
            parameters: vec![
                ParameterConfig { name: "path".to_string(), param_type: "string".to_string(), required: true, description: "文件路径".to_string() },
            ],
        });
        
        commands.insert("file.write".to_string(), CommandConfig {
            enabled: true,
            name: "Write File".to_string(),
            description: "写入内容到文件".to_string(),
            category: "file".to_string(),
            parameters: vec![
                ParameterConfig { name: "path".to_string(), param_type: "string".to_string(), required: true, description: "文件路径".to_string() },
                ParameterConfig { name: "content".to_string(), param_type: "string".to_string(), required: true, description: "文件内容".to_string() },
            ],
        });
        
        commands.insert("file.delete".to_string(), CommandConfig {
            enabled: true,
            name: "Delete File".to_string(),
            description: "删除指定的文件或目录".to_string(),
            category: "file".to_string(),
            parameters: vec![
                ParameterConfig { name: "path".to_string(), param_type: "string".to_string(), required: true, description: "文件或目录路径".to_string() },
            ],
        });
        
        commands.insert("file.create_dir".to_string(), CommandConfig {
            enabled: true,
            name: "Create Directory".to_string(),
            description: "创建目录".to_string(),
            category: "file".to_string(),
            parameters: vec![
                ParameterConfig { name: "path".to_string(), param_type: "string".to_string(), required: true, description: "目录路径".to_string() },
                ParameterConfig { name: "recursive".to_string(), param_type: "boolean".to_string(), required: false, description: "是否递归创建".to_string() },
            ],
        });
        
        commands.insert("file.copy".to_string(), CommandConfig {
            enabled: true,
            name: "Copy File".to_string(),
            description: "复制文件或目录".to_string(),
            category: "file".to_string(),
            parameters: vec![
                ParameterConfig { name: "src".to_string(), param_type: "string".to_string(), required: true, description: "源路径".to_string() },
                ParameterConfig { name: "dst".to_string(), param_type: "string".to_string(), required: true, description: "目标路径".to_string() },
            ],
        });
        
        commands.insert("file.move".to_string(), CommandConfig {
            enabled: true,
            name: "Move File".to_string(),
            description: "移动文件或目录".to_string(),
            category: "file".to_string(),
            parameters: vec![
                ParameterConfig { name: "src".to_string(), param_type: "string".to_string(), required: true, description: "源路径".to_string() },
                ParameterConfig { name: "dst".to_string(), param_type: "string".to_string(), required: true, description: "目标路径".to_string() },
            ],
        });
        
        commands.insert("file.download".to_string(), CommandConfig {
            enabled: true,
            name: "Download File".to_string(),
            description: "从URL下载文件".to_string(),
            category: "file".to_string(),
            parameters: vec![
                ParameterConfig { name: "url".to_string(), param_type: "string".to_string(), required: true, description: "下载链接".to_string() },
                ParameterConfig { name: "dest".to_string(), param_type: "string".to_string(), required: true, description: "目标保存路径".to_string() },
            ],
        });
        
        commands.insert("config.get".to_string(), CommandConfig {
            enabled: true,
            name: "Get Config".to_string(),
            description: "获取配置值".to_string(),
            category: "config".to_string(),
            parameters: vec![
                ParameterConfig { name: "path".to_string(), param_type: "string".to_string(), required: true, description: "配置路径".to_string() },
            ],
        });
        
        commands.insert("config.set".to_string(), CommandConfig {
            enabled: true,
            name: "Set Config".to_string(),
            description: "设置配置值".to_string(),
            category: "config".to_string(),
            parameters: vec![
                ParameterConfig { name: "path".to_string(), param_type: "string".to_string(), required: true, description: "配置路径".to_string() },
                ParameterConfig { name: "value".to_string(), param_type: "any".to_string(), required: true, description: "配置值".to_string() },
            ],
        });
        
        commands.insert("system.reboot".to_string(), CommandConfig {
            enabled: true,
            name: "Reboot System".to_string(),
            description: "重启系统".to_string(),
            category: "system".to_string(),
            parameters: vec![],
        });
        
        commands.insert("system.shutdown".to_string(), CommandConfig {
            enabled: true,
            name: "Shutdown System".to_string(),
            description: "关闭系统".to_string(),
            category: "system".to_string(),
            parameters: vec![],
        });
        
        commands.insert("shell.execute".to_string(), CommandConfig {
            enabled: true,
            name: "Execute Shell Command".to_string(),
            description: "执行Shell命令并返回输出".to_string(),
            category: "shell".to_string(),
            parameters: vec![
                ParameterConfig { name: "command".to_string(), param_type: "string".to_string(), required: true, description: "要执行的命令".to_string() },
                ParameterConfig { name: "timeout".to_string(), param_type: "number".to_string(), required: false, description: "超时时间（秒）".to_string() },
            ],
        });
        
        commands
    }

    pub fn is_enabled(&self, command_id: &str) -> bool {
        self.commands
            .get(command_id)
            .map(|c| c.enabled)
            .unwrap_or(false)
    }

    pub fn get_command_config(&self, command_id: &str) -> Option<&CommandConfig> {
        self.commands.get(command_id)
    }

    pub fn get_enabled_commands(&self) -> Vec<&CommandConfig> {
        self.commands.values().filter(|c| c.enabled).collect()
    }

    pub fn to_capabilities_list(&self) -> serde_json::Value {
        let list: Vec<serde_json::Value> = self.commands
            .iter()
            .map(|(id, config)| {
                serde_json::json!({
                    "id": id,
                    "name": config.name,
                    "description": config.description,
                    "category": config.category,
                    "parameters": config.parameters,
                    "enabled": config.enabled
                })
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
    pub description: String,
}
