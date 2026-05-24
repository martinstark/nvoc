//! GPU validation and safety checks

use crate::nvml::{device_get_architecture, NvmlDevice, Result, NVML_DEVICE_ARCH_BLACKWELL};

/// Validate that the device is a Blackwell GPU
pub fn validate_blackwell_architecture(device: NvmlDevice) -> Result<()> {
    if device_get_architecture(device)? != NVML_DEVICE_ARCH_BLACKWELL {
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
