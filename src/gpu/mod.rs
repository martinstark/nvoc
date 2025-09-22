//! GPU operations and device management

use crate::constants::hardware;
use crate::nvml::{
    device_get_count, device_get_handle_by_index, init, shutdown, system_get_driver_version,
    system_get_nvml_version, NvmlDevice, Result,
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
    match init() {
        Ok(_) => {
            // Check driver and NVML versions for GPU support
            if let Ok(driver_version) = system_get_driver_version() {
                if let Ok(_nvml_version) = system_get_nvml_version() {
                    // Extract major version number from driver (e.g., "560.35.03" -> 560)
                    if let Some(major_str) = driver_version.split('.').next() {
                        if let Ok(major) = major_str.parse::<u32>() {
                            if major < hardware::MIN_DRIVER_VERSION {
                                eprintln!(
                                    "Driver {} too old, need {}+",
                                    driver_version,
                                    hardware::MIN_DRIVER_VERSION
                                );
                                return Err(crate::nvml::NvmlError::NotSupported);
                            } else if major >= hardware::MIN_DRIVER_VERSION {
                                println!("Driver {}", driver_version);
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
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
