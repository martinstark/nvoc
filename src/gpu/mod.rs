//! GPU operations and device management

use crate::constants::hardware;
use crate::nvml::{
    device_get_count, device_get_handle_by_index, init, shutdown, system_get_driver_version,
    NvmlDevice, Result,
};

pub mod domain;
pub mod info;
pub mod overclock;
pub mod power;
pub mod reset;
pub mod validation;

/// Cleanup guard to ensure NVML is properly shut down
pub struct CleanupGuard;

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        let _ = shutdown();
    }
}

pub fn init_nvml() -> Result<()> {
    init()?;

    let driver_version = system_get_driver_version()?;
    let major = driver_version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    if major < hardware::MIN_DRIVER_VERSION {
        eprintln!(
            "Driver {} too old, need {}+",
            driver_version,
            hardware::MIN_DRIVER_VERSION
        );
        return Err(crate::nvml::NvmlError::NotSupported);
    }

    println!("Driver {}", driver_version);
    Ok(())
}

pub fn init_with_cleanup() -> Result<CleanupGuard> {
    init_nvml()?;
    Ok(CleanupGuard)
}

pub fn get_device(device_index: u32) -> Result<NvmlDevice> {
    let device_count = device_get_count()?;

    if device_index >= device_count {
        return Err(crate::nvml::NvmlError::InvalidArgument);
    }

    let device = device_get_handle_by_index(device_index)?;

    Ok(device)
}
