use crate::platform::types::EnvScope;
use crate::error::AgentResult;
use std::collections::HashMap;
use std::process::Command;

pub async fn get_env_var_impl(name: &str, scope: EnvScope) -> AgentResult<Option<String>> {
    match scope {
        EnvScope::Session => Ok(std::env::var(name).ok()),
        EnvScope::User => {
            let output = Command::new("launchctl")
                .args(&["getenv", name])
                .output()?;
            
            if !output.status.success() {
                return Ok(None);
            }
            
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }
        EnvScope::System => Ok(None),
    }
}

pub async fn set_env_var_impl(name: &str, value: &str, scope: EnvScope) -> AgentResult<()> {
    match scope {
        EnvScope::Session => {
            std::env::set_var(name, value);
            Ok(())
        }
        EnvScope::User => {
            let output = Command::new("launchctl")
                .args(&["setenv", name, value])
                .output()?;
            
            if !output.status.success() {
                return Err(anyhow::anyhow!("Failed to set environment variable").into());
            }
            Ok(())
        }
        EnvScope::System => Err(anyhow::anyhow!("Cannot set system environment variable").into()),
    }
}

pub async fn delete_env_var_impl(name: &str, scope: EnvScope) -> AgentResult<()> {
    match scope {
        EnvScope::Session => {
            std::env::remove_var(name);
            Ok(())
        }
        EnvScope::User => {
            let output = Command::new("launchctl")
                .args(&["unsetenv", name])
                .output()?;
            
            if !output.status.success() {
                return Err(anyhow::anyhow!("Failed to delete environment variable").into());
            }
            Ok(())
        }
        EnvScope::System => Err(anyhow::anyhow!("Cannot delete system environment variable").into()),
    }
}

pub async fn list_env_vars_impl(scope: EnvScope) -> AgentResult<HashMap<String, String>> {
    match scope {
        EnvScope::Session => {
            let env: HashMap<String, String> = std::env::vars().collect();
            Ok(env)
        }
        _ => {
            let output = Command::new("launchctl")
                .args(&["export"])
                .output()?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut vars = HashMap::new();
            
            for line in stdout.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    vars.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
            
            Ok(vars)
        }
    }
}
