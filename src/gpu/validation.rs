//! GPU validation and safety checks

use crate::nvml::{
    device_get_architecture, NvmlDevice, Result, NVML_DEVICE_ARCH_AMPERE,
    NVML_DEVICE_ARCH_BLACKWELL,
};

/// Validate that the device is a supported GPU architecture.
pub fn validate_supported_architecture(device: NvmlDevice) -> Result<()> {
    match device_get_architecture(device)? {
        NVML_DEVICE_ARCH_AMPERE | NVML_DEVICE_ARCH_BLACKWELL => Ok(()),
        _ => Err(crate::nvml::NvmlError::NotSupported),
    }
}

/// Check system requirements for operations that modify GPU settings
pub fn check_system_for_modification() -> Result<()> {
    let is_root = unsafe { libc::getuid() == 0 };

    if !is_root {
        return Err(crate::nvml::NvmlError::NoPermission);
    }

    Ok(())
}
