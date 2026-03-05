use crate::AgentConfig;
use std::path::Path;

impl AgentConfig {
    pub fn load_from_file(path: &Path) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        
        if path.extension().and_then(|s| s.to_str()) == Some("yml") || 
           path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            Ok(serde_yaml::from_str(&content)?)
        } else {
            Err(anyhow::anyhow!("不支持的配置文件格式，请使用 YAML 格式 (.yml 或 .yaml)"))
        }
    }
    
    pub fn save_to_file(&self, path: &Path) -> Result<(), anyhow::Error> {
        let content = serde_yaml::to_string(self)?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(path, content)?;
        Ok(())
    }
}
