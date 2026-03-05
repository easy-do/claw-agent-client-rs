use crate::platform::types::EnvScope;
use crate::error::AgentResult;
use std::collections::HashMap;
use std::process::Command;

pub async fn get_env_var_impl(name: &str, scope: EnvScope) -> AgentResult<Option<String>> {
    let reg_path = match scope {
        EnvScope::User => "HKCU\\Environment",
        EnvScope::System => "HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
        EnvScope::Session => return Ok(std::env::var(name).ok()),
    };
    
    let output = Command::new("reg")
        .args(&["query", reg_path, "/v", name])
        .output()?;
    
    if !output.status.success() {
        return Ok(None);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(value) = stdout.split_whitespace().nth(2) {
        Ok(Some(value.to_string()))
    } else {
        Ok(None)
    }
}

pub async fn set_env_var_impl(name: &str, value: &str, scope: EnvScope) -> AgentResult<()> {
    let reg_path = match scope {
        EnvScope::User => "HKCU\\Environment",
        EnvScope::System => "HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
        EnvScope::Session => {
            std::env::set_var(name, value);
            return Ok(());
        }
    };
    
    let output = Command::new("reg")
        .args(&["add", reg_path, "/v", name, "/t", "REG_EXPAND_SZ", "/d", value, "/f"])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to set environment variable: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}

pub async fn delete_env_var_impl(name: &str, scope: EnvScope) -> AgentResult<()> {
    let reg_path = match scope {
        EnvScope::User => "HKCU\\Environment",
        EnvScope::System => "HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
        EnvScope::Session => {
            std::env::remove_var(name);
            return Ok(());
        }
    };
    
    let output = Command::new("reg")
        .args(&["delete", reg_path, "/v", name, "/f"])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to delete environment variable: {}", 
            String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}

pub async fn list_env_vars_impl(scope: EnvScope) -> AgentResult<HashMap<String, String>> {
    match scope {
        EnvScope::Session => {
            let env: HashMap<String, String> = std::env::vars().collect();
            Ok(env)
        }
        _ => {
            let reg_path = match scope {
                EnvScope::User => "HKCU\\Environment",
                EnvScope::System => "HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
                EnvScope::Session => unreachable!(),
            };
            
            let output = Command::new("reg")
                .args(&["query", reg_path])
                .output()?;
            
            if !output.status.success() {
                return Ok(HashMap::new());
            }
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut vars = HashMap::new();
            
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[0];
                    let value = parts[2..].join(" ");
                    vars.insert(name.to_string(), value);
                }
            }
            
            Ok(vars)
        }
    }
}
