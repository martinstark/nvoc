//! Runtime NVML library loader
//!
//! Dynamically loads the NVML library at runtime instead of build-time linking.
//! This allows distributing standalone binaries without requiring NVML at build time.

use libloading::Library;
use std::sync::OnceLock;

use crate::nvml::types::{NvmlClockOffset, NvmlClockType, NvmlDevice, NvmlReturn};
use libc::{c_char, c_int, c_uint};

/// Global NVML library instance
static NVML_LIB: OnceLock<Result<Library, crate::nvml::NvmlError>> = OnceLock::new();

/// Load the NVML library at runtime
pub fn load_nvml_library() -> Result<&'static Library, crate::nvml::NvmlError> {
    let lib_result = NVML_LIB.get_or_init(|| {
        // Try common library names and paths
        let lib_names = [
            "libnvidia-ml.so.1",
            "libnvidia-ml.so",
            "/usr/lib/x86_64-linux-gnu/libnvidia-ml.so.1",
            "/usr/lib64/libnvidia-ml.so.1",
            "/usr/lib/libnvidia-ml.so.1",
            "/usr/lib/x86_64-linux-gnu/libnvidia-ml.so",
            "/usr/lib64/libnvidia-ml.so",
            "/usr/lib/libnvidia-ml.so",
        ];

        for name in &lib_names {
            if let Ok(lib) = unsafe { Library::new(name) } {
                return Ok(lib);
            }
        }

        // Try loading without path (system will search)
        match unsafe { Library::new("nvidia-ml") } {
            Ok(lib) => Ok(lib),
            Err(_) => Err(crate::nvml::NvmlError::LibraryNotFound),
        }
    });

    lib_result.as_ref().map_err(|e| e.clone())
}

// Individual function wrappers
pub fn nvml_init_v2() -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn() -> NvmlReturn> = unsafe {
        lib.get(b"nvmlInit_v2")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func() })
}

pub fn nvml_shutdown() -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn() -> NvmlReturn> = unsafe {
        lib.get(b"nvmlShutdown")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func() })
}

pub fn nvml_system_get_driver_version(
    version: *mut c_char,
    length: c_uint,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn(*mut c_char, c_uint) -> NvmlReturn> = unsafe {
        lib.get(b"nvmlSystemGetDriverVersion")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(version, length) })
}

pub fn nvml_device_get_count_v2(
    device_count: *mut c_uint,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn(*mut c_uint) -> NvmlReturn> = unsafe {
        lib.get(b"nvmlDeviceGetCount_v2")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device_count) })
}

pub fn nvml_device_get_handle_by_index_v2(
    index: c_uint,
    device: *mut NvmlDevice,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn(c_uint, *mut NvmlDevice) -> NvmlReturn> = unsafe {
        lib.get(b"nvmlDeviceGetHandleByIndex_v2")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(index, device) })
}

pub fn nvml_device_get_name(
    device: NvmlDevice,
    name: *mut c_char,
    length: c_uint,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<
        unsafe extern "C" fn(NvmlDevice, *mut c_char, c_uint) -> NvmlReturn,
    > = unsafe {
        lib.get(b"nvmlDeviceGetName")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, name, length) })
}

pub fn nvml_device_get_clock_offsets(
    device: NvmlDevice,
    clock_offsets: *mut NvmlClockOffset,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<
        unsafe extern "C" fn(NvmlDevice, *mut NvmlClockOffset) -> NvmlReturn,
    > = unsafe {
        lib.get(b"nvmlDeviceGetClockOffsets")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, clock_offsets) })
}

pub fn nvml_device_set_clock_offsets(
    device: NvmlDevice,
    clock_offsets: *const NvmlClockOffset,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<
        unsafe extern "C" fn(NvmlDevice, *const NvmlClockOffset) -> NvmlReturn,
    > = unsafe {
        lib.get(b"nvmlDeviceSetClockOffsets")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, clock_offsets) })
}

pub fn nvml_device_set_gpu_locked_clocks(
    device: NvmlDevice,
    min_gpu_clock: c_uint,
    max_gpu_clock: c_uint,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice, c_uint, c_uint) -> NvmlReturn> = unsafe {
        lib.get(b"nvmlDeviceSetGpuLockedClocks")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, min_gpu_clock, max_gpu_clock) })
}

pub fn nvml_device_reset_gpu_locked_clocks(
    device: NvmlDevice,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice) -> NvmlReturn> = unsafe {
        lib.get(b"nvmlDeviceResetGpuLockedClocks")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device) })
}

pub fn nvml_device_reset_memory_locked_clocks(
    device: NvmlDevice,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice) -> NvmlReturn> = unsafe {
        lib.get(b"nvmlDeviceResetMemoryLockedClocks")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device) })
}

pub fn nvml_device_set_mem_clk_vf_offset(
    device: NvmlDevice,
    offset: c_int,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice, c_int) -> NvmlReturn> = unsafe {
        lib.get(b"nvmlDeviceSetMemClkVfOffset")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, offset) })
}

pub fn nvml_device_get_clock_info(
    device: NvmlDevice,
    clock_type: NvmlClockType,
    clock: *mut c_uint,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<
        unsafe extern "C" fn(NvmlDevice, NvmlClockType, *mut c_uint) -> NvmlReturn,
    > = unsafe {
        lib.get(b"nvmlDeviceGetClockInfo")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, clock_type, clock) })
}

pub fn nvml_device_get_temperature(
    device: NvmlDevice,
    sensor_type: c_uint,
    temp: *mut c_uint,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<
        unsafe extern "C" fn(NvmlDevice, c_uint, *mut c_uint) -> NvmlReturn,
    > = unsafe {
        lib.get(b"nvmlDeviceGetTemperature")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, sensor_type, temp) })
}

pub fn nvml_device_get_power_usage(
    device: NvmlDevice,
    power: *mut c_uint,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice, *mut c_uint) -> NvmlReturn> = unsafe {
        lib.get(b"nvmlDeviceGetPowerUsage")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, power) })
}

pub fn nvml_device_get_power_management_limit_constraints(
    device: NvmlDevice,
    min_limit: *mut c_uint,
    max_limit: *mut c_uint,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<
        unsafe extern "C" fn(NvmlDevice, *mut c_uint, *mut c_uint) -> NvmlReturn,
    > = unsafe {
        lib.get(b"nvmlDeviceGetPowerManagementLimitConstraints")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, min_limit, max_limit) })
}

pub fn nvml_device_get_power_management_limit(
    device: NvmlDevice,
    limit: *mut c_uint,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice, *mut c_uint) -> NvmlReturn> = unsafe {
        lib.get(b"nvmlDeviceGetPowerManagementLimit")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, limit) })
}

pub fn nvml_device_get_power_management_default_limit(
    device: NvmlDevice,
    default_limit: *mut c_uint,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice, *mut c_uint) -> NvmlReturn> = unsafe {
        lib.get(b"nvmlDeviceGetPowerManagementDefaultLimit")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, default_limit) })
}

pub fn nvml_device_set_power_management_limit(
    device: NvmlDevice,
    limit: c_uint,
) -> Result<NvmlReturn, crate::nvml::NvmlError> {
    let lib = load_nvml_library()?;
    let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice, c_uint) -> NvmlReturn> = unsafe {
        lib.get(b"nvmlDeviceSetPowerManagementLimit")
            .map_err(|_| crate::nvml::NvmlError::FunctionNotFound)?
    };
    Ok(unsafe { func(device, limit) })
}
