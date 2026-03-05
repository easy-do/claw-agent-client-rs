use crate::platform::types::EnvScope;
use crate::error::AgentResult;
use std::collections::HashMap;
use std::process::Command;
use std::io::Write;

pub async fn get_env_var_impl(name: &str, scope: EnvScope) -> AgentResult<Option<String>> {
    match scope {
        EnvScope::Session => Ok(std::env::var(name).ok()),
        EnvScope::User => {
            let output = Command::new("printenv")
                .arg(name)
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
        EnvScope::System => {
            let output = Command::new("printenv")
                .arg(name)
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
    }
}

pub async fn set_env_var_impl(name: &str, value: &str, scope: EnvScope) -> AgentResult<()> {
    match scope {
        EnvScope::Session => {
            std::env::set_var(name, value);
            Ok(())
        }
        EnvScope::User => {
            let shell_profile = std::env::var("SHELL")
                .map(|s| s.contains("zsh"))
                .unwrap_or(false)
                .then(|| ".zshrc")
                .unwrap_or(".bashrc");
            
            let home = dirs::home_dir().unwrap();
            let profile_path = home.join(shell_profile);
            
            let export_line = format!("export {}='{}'", name, value);
            
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(profile_path)?
                .write_all(format!("\n{}\n", export_line).as_bytes())?;
            
            Ok(())
        }
        EnvScope::System => {
            Err(anyhow::anyhow!("Cannot set system environment variable without root").into())
        }
    }
}

pub async fn delete_env_var_impl(name: &str, scope: EnvScope) -> AgentResult<()> {
    match scope {
        EnvScope::Session => {
            std::env::remove_var(name);
            Ok(())
        }
        _ => {
            Err(anyhow::anyhow!("Cannot delete persistent environment variable without elevated privileges").into())
        }
    }
}

pub async fn list_env_vars_impl(scope: EnvScope) -> AgentResult<HashMap<String, String>> {
    let env: HashMap<String, String> = std::env::vars().collect();
    Ok(env)
}
