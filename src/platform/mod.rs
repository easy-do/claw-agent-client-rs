pub mod traits;
pub mod types;
pub mod common;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "linux")]
pub mod linux;

pub use traits::{Platform, get_platform, platform_name};
pub use types::*;
