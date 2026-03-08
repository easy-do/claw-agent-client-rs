use crate::config::AgentConfig;
use crate::error::AgentResult;

pub fn validate_config(config: &AgentConfig) -> AgentResult<()> {
    if config.agent_id.is_empty() {
        return Err(crate::error::AgentError::Config("agent_id 不能为空".to_string()).into());
    }

    if config.auth.token.as_deref().map(str::is_empty).unwrap_or(true) {
        return Err(crate::error::AgentError::Config("认证 token 是必需的".to_string()).into());
    }
    
    Ok(())
}
