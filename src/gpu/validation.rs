//! GPU validation and safety checks

use crate::nvml::{device_get_name, GpuArchitecture, NvmlDevice, Result};

/// Validate that the device is a Blackwell GPU
pub fn validate_blackwell_architecture(device: NvmlDevice) -> Result<()> {
    let device_name = device_get_name(device)?;
    let arch = GpuArchitecture::from_device_name(&device_name);

    if arch != GpuArchitecture::Blackwell {
        return Err(crate::nvml::NvmlError::NotSupported);
    }

    Ok(())
}

/// Check system requirements for operations that modify GPU settings
pub fn check_system_for_modification() -> Result<()> {
    let is_root = unsafe { libc::getuid() == 0 };

    if !is_root {
        return Err(crate::nvml::NvmlError::NoPermission);
    }

    Ok(())
}
