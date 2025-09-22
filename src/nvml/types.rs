//! NVML type definitions and constants
//!
//! This module defines the types, enums, and constants used by the NVML API
//! for GPU management and overclocking operations.

use crate::constants::{buffers, errors};

use libc::{c_int, c_uint, c_void};

/// NVML device handle (opaque pointer)
pub type NvmlDevice = *mut c_void;

/// NVML return codes
pub type NvmlReturn = c_uint;

/// NVML clock types
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum NvmlClockType {
    #[default]
    Graphics = 0,
    Memory = 2,
}

/// NVML performance states (P-states)
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum NvmlPerfState {
    #[default]
    P0 = 0, // Maximum performance
}

/// NVML enable state
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum NvmlEnableState {
    #[default]
    Disabled = 0,
}

// NVML Return Codes
pub const NVML_SUCCESS: NvmlReturn = 0;
pub const NVML_ERROR_UNINITIALIZED: NvmlReturn = 1;
pub const NVML_ERROR_INVALID_ARGUMENT: NvmlReturn = 2;
pub const NVML_ERROR_NOT_SUPPORTED: NvmlReturn = 3;
pub const NVML_ERROR_NO_PERMISSION: NvmlReturn = 4;
pub const NVML_ERROR_ALREADY_INITIALIZED: NvmlReturn = 5;
pub const NVML_ERROR_NOT_FOUND: NvmlReturn = 6;
pub const NVML_ERROR_INSUFFICIENT_SIZE: NvmlReturn = 7;
pub const NVML_ERROR_INSUFFICIENT_POWER: NvmlReturn = 8;
pub const NVML_ERROR_DRIVER_NOT_LOADED: NvmlReturn = 9;
pub const NVML_ERROR_TIMEOUT: NvmlReturn = 10;
pub const NVML_ERROR_IRQ_ISSUE: NvmlReturn = 11;
pub const NVML_ERROR_LIBRARY_NOT_FOUND: NvmlReturn = 12;
pub const NVML_ERROR_FUNCTION_NOT_FOUND: NvmlReturn = 13;
pub const NVML_ERROR_CORRUPTED_INFOROM: NvmlReturn = 14;
pub const NVML_ERROR_GPU_IS_LOST: NvmlReturn = 15;
pub const NVML_ERROR_RESET_REQUIRED: NvmlReturn = 16;
pub const NVML_ERROR_OPERATING_SYSTEM: NvmlReturn = 17;
pub const NVML_ERROR_LIB_RM_VERSION_MISMATCH: NvmlReturn = 18;
pub const NVML_ERROR_IN_USE: NvmlReturn = 19;
pub const NVML_ERROR_MEMORY: NvmlReturn = 20;
pub const NVML_ERROR_NO_DATA: NvmlReturn = 21;
pub const NVML_ERROR_VGPU_ECC_NOT_SUPPORTED: NvmlReturn = 22;
pub const NVML_ERROR_INSUFFICIENT_RESOURCES: NvmlReturn = 23;
pub const NVML_ERROR_FREQ_NOT_SUPPORTED: NvmlReturn = 24;
pub const NVML_ERROR_ARGUMENT_VERSION_MISMATCH: NvmlReturn = 25;
pub const NVML_ERROR_DEPRECATED: NvmlReturn = 26;
pub const NVML_ERROR_NOT_READY: NvmlReturn = 27;
pub const NVML_ERROR_UNKNOWN: NvmlReturn = errors::NVML_UNKNOWN_ERROR_CODE;

// Buffer sizes
pub const NVML_DEVICE_NAME_BUFFER_SIZE: usize = buffers::DEVICE_NAME_BUFFER_SIZE;

// NVML Clock Offset Version Constants
pub const NVML_CLOCK_OFFSET_V1: u32 = 0x1000018; // 16777240 - Blackwell

/// Clock offset structure for NVML (v1: Blackwell)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]
pub struct NvmlClockOffset {
    pub version: u32,
    pub type_: NvmlClockType,
    pub pstate: NvmlPerfState,
    pub clockOffsetMHz: c_int,    // Clock offset in MHz
    pub minClockOffsetMHz: c_int, // Minimum clock offset in MHz
    pub maxClockOffsetMHz: c_int, // Maximum clock offset in MHz
}

impl NvmlClockOffset {
    /// Create v1 struct (Blackwell)
    pub fn new_v1(clock_type: NvmlClockType, pstate: NvmlPerfState, offset: i32) -> Self {
        NvmlClockOffset {
            version: NVML_CLOCK_OFFSET_V1,
            type_: clock_type,
            pstate,
            clockOffsetMHz: offset,
            minClockOffsetMHz: 0,
            maxClockOffsetMHz: 0,
        }
    }
}

impl Default for NvmlClockOffset {
    fn default() -> Self {
        Self::new_v1(NvmlClockType::Graphics, NvmlPerfState::P0, 0)
    }
}

/// GPU Architecture detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuArchitecture {
    Blackwell, // RTX 50-series
    Unknown,
}

impl GpuArchitecture {
    /// Detect GPU architecture from device name
    pub fn from_device_name(name: &str) -> Self {
        let name_upper = name.to_uppercase();

        // Blackwell (RTX 50-series)
        if name_upper.contains("RTX 50")
            || name_upper.contains("5090")
            || name_upper.contains("5080")
            || name_upper.contains("5070")
            || name_upper.contains("5060")
        {
            GpuArchitecture::Blackwell
        } else {
            GpuArchitecture::Unknown
        }
    }

    /// Get struct version for clock offsets (Blackwell uses v1)
    pub fn get_clock_offset_version(&self) -> u32 {
        match self {
            GpuArchitecture::Blackwell => NVML_CLOCK_OFFSET_V1,
            GpuArchitecture::Unknown => NVML_CLOCK_OFFSET_V1, // Default to Blackwell version
        }
    }
}

impl std::fmt::Display for NvmlClockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NvmlClockType::Graphics => write!(f, "Graphics"),
            NvmlClockType::Memory => write!(f, "Memory"),
        }
    }
}

/// Convert performance state to string representation
impl std::fmt::Display for NvmlPerfState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NvmlPerfState::P0 => write!(f, "P0"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blackwell_detection() {
        assert_eq!(
            GpuArchitecture::from_device_name("NVIDIA GeForce RTX 5090"),
            GpuArchitecture::Blackwell
        );
        assert_eq!(
            GpuArchitecture::from_device_name("GeForce RTX 5080"),
            GpuArchitecture::Blackwell
        );
        assert_eq!(
            GpuArchitecture::from_device_name("RTX 5070 Ti"),
            GpuArchitecture::Blackwell
        );
        assert_eq!(
            GpuArchitecture::from_device_name("GeForce RTX 5060"),
            GpuArchitecture::Blackwell
        );
    }

    #[test]
    fn test_version_selection() {
        assert_eq!(
            GpuArchitecture::Blackwell.get_clock_offset_version(),
            NVML_CLOCK_OFFSET_V1
        );
        assert_eq!(
            GpuArchitecture::Unknown.get_clock_offset_version(),
            NVML_CLOCK_OFFSET_V1
        );
    }

    #[test]
    fn test_unknown_gpu() {
        assert_eq!(
            GpuArchitecture::from_device_name("Some Unknown GPU"),
            GpuArchitecture::Unknown
        );
    }

    #[test]
    fn test_struct_creation() {
        let v1_struct = NvmlClockOffset::new_v1(NvmlClockType::Graphics, NvmlPerfState::P0, 100);
        assert_eq!(v1_struct.version, NVML_CLOCK_OFFSET_V1);
        assert_eq!(v1_struct.clockOffsetMHz, 100);
        assert_eq!(
            v1_struct.minClockOffsetMHz,
            crate::constants::clocks::DEFAULT_GRAPHICS_OFFSET
        );
    }
}
