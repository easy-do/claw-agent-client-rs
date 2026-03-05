use crate::error::AgentResult;

pub async fn call_dbus_method(
    _service: &str,
    _path: &str,
    _interface: &str,
    _method: &str,
    _args: &[&str],
) -> AgentResult<String> {
    Err(anyhow::anyhow!("D-Bus not fully implemented").into())
}

pub async fn get_dbus_property(
    _service: &str,
    _path: &str,
    _interface: &str,
    _property: &str,
) -> AgentResult<String> {
    Err(anyhow::anyhow!("D-Bus not fully implemented").into())
}
