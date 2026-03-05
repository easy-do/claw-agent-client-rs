use crate::platform::types::ProcessInfo;
use crate::error::AgentResult;
use sysinfo::System;

pub async fn list_processes_impl() -> AgentResult<Vec<ProcessInfo>> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let processes: Vec<ProcessInfo> = sys.processes()
        .iter()
        .map(|(pid, process)| {
            let cmd: Vec<String> = process.cmd().iter().map(|s| s.to_string()).collect();
            
            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cmd: cmd.join(" "),
                cpu_percent: process.cpu_usage(),
                memory_mb: process.memory() / 1024 / 1024,
                status: format!("{:?}", process.status()),
                start_time: Some(process.start_time()),
                user: process.user_id().map(|u| u.to_string()),
            }
        })
        .collect();
    
    Ok(processes)
}
