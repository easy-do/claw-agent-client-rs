use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::time::{timeout, Duration};
use anyhow::Result;

pub async fn is_port_available(port: u16) -> bool {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    match TcpListener::bind(addr).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn wait_for_port(port: u16, timeout_secs: u64) -> Result<bool> {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let deadline = Duration::from_secs(timeout_secs);
    
    match timeout(deadline, TcpListener::bind(addr)).await {
        Ok(Ok(_)) => Ok(true),
        _ => Ok(false),
    }
}

pub fn parse_socket_addr(addr: &str) -> Result<SocketAddr> {
    addr.parse().map_err(|e| anyhow::anyhow!("Invalid socket address: {}", e))
}

pub fn format_socket_addr(ip: &str, port: u16) -> String {
    format!("{}:{}", ip, port)
}
