//! NVML error handling

use crate::nvml::types::{
    NvmlReturn, NVML_ERROR_ALREADY_INITIALIZED, NVML_ERROR_ARGUMENT_VERSION_MISMATCH,
    NVML_ERROR_CORRUPTED_INFOROM, NVML_ERROR_DEPRECATED, NVML_ERROR_DRIVER_NOT_LOADED,
    NVML_ERROR_FREQ_NOT_SUPPORTED, NVML_ERROR_FUNCTION_NOT_FOUND, NVML_ERROR_GPU_IS_LOST,
    NVML_ERROR_INSUFFICIENT_POWER, NVML_ERROR_INSUFFICIENT_RESOURCES, NVML_ERROR_INSUFFICIENT_SIZE,
    NVML_ERROR_INVALID_ARGUMENT, NVML_ERROR_IN_USE, NVML_ERROR_IRQ_ISSUE,
    NVML_ERROR_LIBRARY_NOT_FOUND, NVML_ERROR_LIB_RM_VERSION_MISMATCH, NVML_ERROR_MEMORY,
    NVML_ERROR_NOT_FOUND, NVML_ERROR_NOT_READY, NVML_ERROR_NOT_SUPPORTED, NVML_ERROR_NO_DATA,
    NVML_ERROR_NO_PERMISSION, NVML_ERROR_OPERATING_SYSTEM, NVML_ERROR_RESET_REQUIRED,
    NVML_ERROR_TIMEOUT, NVML_ERROR_UNINITIALIZED, NVML_ERROR_UNKNOWN,
    NVML_ERROR_VGPU_ECC_NOT_SUPPORTED,
};
use std::fmt;

/// NVML operation result type
pub type Result<T> = std::result::Result<T, NvmlError>;

/// NVML error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NvmlError {
    /// NVML library not initialized
    Uninitialized,
    /// Invalid argument passed to NVML function
    InvalidArgument,
    /// Operation not supported by this device
    NotSupported,
    /// Insufficient permissions to perform operation
    NoPermission,
    /// NVML library already initialized
    AlreadyInitialized,
    /// Device or resource not found
    NotFound,
    /// Buffer size insufficient for operation
    InsufficientSize,
    /// Insufficient power for operation
    InsufficientPower,
    /// NVIDIA driver not loaded
    DriverNotLoaded,
    /// Operation timed out
    Timeout,
    /// IRQ issue detected
    IrqIssue,
    /// NVML library not found
    LibraryNotFound,
    /// Required function not found in library
    FunctionNotFound,
    /// `InfoROM` corrupted
    CorruptedInforom,
    /// GPU is lost and needs reset
    GpuIsLost,
    /// GPU reset required
    ResetRequired,
    /// Operating system error
    OperatingSystem,
    /// Library version mismatch
    LibRmVersionMismatch,
    /// Resource is currently in use
    InUse,
    /// Memory allocation failed
    Memory,
    /// No data available
    NoData,
    /// vGPU ECC not supported
    VgpuEccNotSupported,
    /// Insufficient resources
    InsufficientResources,
    /// Frequency not supported
    FreqNotSupported,
    /// Argument version mismatch
    ArgumentVersionMismatch,
    /// Function deprecated
    Deprecated,
    /// System not ready
    NotReady,
    /// Unknown error
    Unknown(u32),
}

impl NvmlError {
    /// Convert NVML return code to error type
    #[must_use]
    pub const fn from_nvml_return(code: NvmlReturn) -> Self {
        match code {
            NVML_ERROR_UNINITIALIZED => Self::Uninitialized,
            NVML_ERROR_INVALID_ARGUMENT => Self::InvalidArgument,
            NVML_ERROR_NOT_SUPPORTED => Self::NotSupported,
            NVML_ERROR_NO_PERMISSION => Self::NoPermission,
            NVML_ERROR_ALREADY_INITIALIZED => Self::AlreadyInitialized,
            NVML_ERROR_NOT_FOUND => Self::NotFound,
            NVML_ERROR_INSUFFICIENT_SIZE => Self::InsufficientSize,
            NVML_ERROR_INSUFFICIENT_POWER => Self::InsufficientPower,
            NVML_ERROR_DRIVER_NOT_LOADED => Self::DriverNotLoaded,
            NVML_ERROR_TIMEOUT => Self::Timeout,
            NVML_ERROR_IRQ_ISSUE => Self::IrqIssue,
            NVML_ERROR_LIBRARY_NOT_FOUND => Self::LibraryNotFound,
            NVML_ERROR_FUNCTION_NOT_FOUND => Self::FunctionNotFound,
            NVML_ERROR_CORRUPTED_INFOROM => Self::CorruptedInforom,
            NVML_ERROR_GPU_IS_LOST => Self::GpuIsLost,
            NVML_ERROR_RESET_REQUIRED => Self::ResetRequired,
            NVML_ERROR_OPERATING_SYSTEM => Self::OperatingSystem,
            NVML_ERROR_LIB_RM_VERSION_MISMATCH => Self::LibRmVersionMismatch,
            NVML_ERROR_IN_USE => Self::InUse,
            NVML_ERROR_MEMORY => Self::Memory,
            NVML_ERROR_NO_DATA => Self::NoData,
            NVML_ERROR_VGPU_ECC_NOT_SUPPORTED => Self::VgpuEccNotSupported,
            NVML_ERROR_INSUFFICIENT_RESOURCES => Self::InsufficientResources,
            NVML_ERROR_FREQ_NOT_SUPPORTED => Self::FreqNotSupported,
            NVML_ERROR_ARGUMENT_VERSION_MISMATCH => Self::ArgumentVersionMismatch,
            NVML_ERROR_DEPRECATED => Self::Deprecated,
            NVML_ERROR_NOT_READY => Self::NotReady,
            NVML_ERROR_UNKNOWN => Self::Unknown(999),
            code => Self::Unknown(code),
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Uninitialized => "NVML library not initialized",
            Self::InvalidArgument => "Invalid argument provided",
            Self::NotSupported => "Operation not supported by this GPU",
            Self::NoPermission => "Not root",
            Self::AlreadyInitialized => "NVML library already initialized",
            Self::NotFound => "GPU device not found",
            Self::InsufficientSize => "Buffer size too small",
            Self::InsufficientPower => "Insufficient power for operation",
            Self::DriverNotLoaded => "NVIDIA driver not loaded",
            Self::Timeout => "Operation timed out",
            Self::IrqIssue => "Hardware interrupt issue",
            Self::LibraryNotFound => "NVML library not found",
            Self::FunctionNotFound => "Required function not available",
            Self::CorruptedInforom => "GPU `InfoROM` corrupted",
            Self::GpuIsLost => "GPU is lost and requires reset",
            Self::ResetRequired => "GPU reset required",
            Self::OperatingSystem => "Operating system error",
            Self::LibRmVersionMismatch => "Driver version mismatch",
            Self::InUse => "Resource currently in use",
            Self::Memory => "Memory allocation failed",
            Self::NoData => "No data available",
            Self::VgpuEccNotSupported => "vGPU ECC not supported",
            Self::InsufficientResources => "Insufficient system resources",
            Self::FreqNotSupported => "Frequency not supported",
            Self::ArgumentVersionMismatch => "API version mismatch",
            Self::Deprecated => "Function deprecated",
            Self::NotReady => "System not ready",
            Self::Unknown(_) => "Unknown NVML error",
        }
    }

    /// Get actionable error message with specific guidance
    pub fn actionable_message(&self) -> &'static str {
        match self {
            Self::NoPermission => {
                "Insufficient permissions. Run with root privileges:\n  sudo nvoc <your-options>"
            }
            Self::DriverNotLoaded => {
                "NVIDIA driver not loaded. Install nvidia-open drivers and nvidia-utils package"
            }
            Self::LibraryNotFound => {
                "NVML library not found. Install nvidia-utils package"
            }
            Self::NotSupported => {
                "Operation not supported by this GPU. Check if:\n  1. GPU supports overclocking\n  2. Coolbits are properly configured"
            }
            Self::NotFound => {
                "GPU device not found. Check available devices:\n  nvidia-smi -L"
            }
            Self::LibRmVersionMismatch => {
                "Driver version mismatch. Update packages:\n  sudo pacman -Syu"
            }
            _ => self.user_message(),
        }
    }
}

impl fmt::Display for NvmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(code) => write!(f, "Unknown NVML error (code: {})", code),
            _ => write!(f, "{}", self.user_message()),
        }
    }
}

impl std::error::Error for NvmlError {}
