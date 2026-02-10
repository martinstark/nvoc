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
            Self::Uninitialized => "nvml not initialized",
            Self::InvalidArgument => "invalid argument",
            Self::NotSupported => "not supported by this gpu",
            Self::NoPermission => "not root, did you forget sudo?",
            Self::AlreadyInitialized => "nvml already initialized",
            Self::NotFound => "gpu not found, check nvidia-smi -L",
            Self::InsufficientSize => "buffer too small",
            Self::InsufficientPower => "insufficient power",
            Self::DriverNotLoaded => "driver not loaded, install nvidia-open and nvidia-utils",
            Self::Timeout => "operation timed out",
            Self::IrqIssue => "hardware interrupt issue",
            Self::LibraryNotFound => "nvml not found, install nvidia-utils",
            Self::FunctionNotFound => "required function not available",
            Self::CorruptedInforom => "inforom corrupted",
            Self::GpuIsLost => "gpu lost, reset required",
            Self::ResetRequired => "gpu reset required",
            Self::OperatingSystem => "operating system error",
            Self::LibRmVersionMismatch => "driver mismatch, run sudo pacman -Syu",
            Self::InUse => "resource in use",
            Self::Memory => "memory allocation failed",
            Self::NoData => "no data available",
            Self::VgpuEccNotSupported => "vgpu ecc not supported",
            Self::InsufficientResources => "insufficient resources",
            Self::FreqNotSupported => "frequency not supported",
            Self::ArgumentVersionMismatch => "api version mismatch",
            Self::Deprecated => "function deprecated",
            Self::NotReady => "system not ready",
            Self::Unknown(_) => "unknown nvml error",
        }
    }
}

impl fmt::Display for NvmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(code) => write!(f, "unknown nvml error (code: {})", code),
            _ => write!(f, "{}", self.user_message()),
        }
    }
}

impl std::error::Error for NvmlError {}
