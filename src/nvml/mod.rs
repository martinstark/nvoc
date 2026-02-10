//! NVML wrapper for GPU overclocking operations

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::constants::{buffers, hardware};
use libc::c_uint;
use std::ffi::CStr;
use std::ptr;

pub mod error;
pub mod loader;
pub mod types;

pub use error::{NvmlError, Result};
pub use types::{
    GpuArchitecture, NvmlClockOffset, NvmlClockType, NvmlDevice, NvmlPerfState,
    NVML_DEVICE_NAME_BUFFER_SIZE, NVML_SUCCESS,
};

pub fn init() -> Result<()> {
    let result = loader::nvml_init_v2()?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(())
}

pub fn shutdown() -> Result<()> {
    let result = loader::nvml_shutdown()?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(())
}

pub fn system_get_driver_version() -> Result<String> {
    let mut version = [0i8; buffers::DRIVER_VERSION_BUFFER_SIZE];
    let result = loader::nvml_system_get_driver_version(
        version.as_mut_ptr(),
        buffers::DRIVER_VERSION_BUFFER_SIZE as c_uint,
    )?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    unsafe {
        let c_str = CStr::from_ptr(version.as_ptr());
        Ok(c_str.to_string_lossy().to_string())
    }
}

pub fn device_get_count() -> Result<u32> {
    let mut device_count: c_uint = 0;
    let result = loader::nvml_device_get_count_v2(&mut device_count)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(device_count)
}

pub fn device_get_handle_by_index(index: u32) -> Result<NvmlDevice> {
    let mut device: NvmlDevice = ptr::null_mut();
    let result = loader::nvml_device_get_handle_by_index_v2(index, &mut device)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(device)
}

pub fn device_get_name(device: NvmlDevice) -> Result<String> {
    let mut name = [0i8; NVML_DEVICE_NAME_BUFFER_SIZE];
    let result = loader::nvml_device_get_name(
        device,
        name.as_mut_ptr(),
        NVML_DEVICE_NAME_BUFFER_SIZE as c_uint,
    )?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    unsafe {
        let c_str = CStr::from_ptr(name.as_ptr());
        Ok(c_str.to_string_lossy().to_string())
    }
}

pub fn device_get_clock_offsets(
    device: NvmlDevice,
    clock_type: NvmlClockType,
) -> Result<NvmlClockOffset> {
    let mut offset = NvmlClockOffset::new_v1(clock_type, NvmlPerfState::P0, 0);
    let result = loader::nvml_device_get_clock_offsets(device, &mut offset)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(offset)
}

pub fn device_set_clock_offset(
    device: NvmlDevice,
    clock_type: NvmlClockType,
    perf_state: NvmlPerfState,
    offset: i32,
) -> Result<()> {
    let clock_offset = NvmlClockOffset::new_v1(clock_type, perf_state, offset);
    let result = loader::nvml_device_set_clock_offsets(device, &clock_offset)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(())
}

pub fn device_set_memory_vf_offset(device: NvmlDevice, offset: i32) -> Result<()> {
    let result = loader::nvml_device_set_mem_clk_vf_offset(device, offset)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(())
}

pub fn device_set_gpu_locked_clocks(
    device: NvmlDevice,
    min_gpu_clock: u32,
    max_gpu_clock: u32,
) -> Result<()> {
    let result = loader::nvml_device_set_gpu_locked_clocks(device, min_gpu_clock, max_gpu_clock)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(())
}

pub fn device_reset_gpu_locked_clocks(device: NvmlDevice) -> Result<()> {
    let result = loader::nvml_device_reset_gpu_locked_clocks(device)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(())
}

pub fn device_reset_memory_locked_clocks(device: NvmlDevice) -> Result<()> {
    let result = loader::nvml_device_reset_memory_locked_clocks(device)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(())
}

pub fn device_get_temperature(device: NvmlDevice) -> Result<u32> {
    let mut temp: c_uint = 0;
    let result = loader::nvml_device_get_temperature(device, hardware::GPU_TEMP_SENSOR, &mut temp)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(temp)
}

pub fn device_get_power_usage(device: NvmlDevice) -> Result<u32> {
    let mut power: c_uint = 0;
    let result = loader::nvml_device_get_power_usage(device, &mut power)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(power)
}

pub fn device_get_clock_info(device: NvmlDevice, clock_type: NvmlClockType) -> Result<u32> {
    let mut clock: c_uint = 0;
    let result = loader::nvml_device_get_clock_info(device, clock_type, &mut clock)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(clock)
}

pub fn device_get_power_limit_constraints(device: NvmlDevice) -> Result<(u32, u32)> {
    let mut min_limit: c_uint = 0;
    let mut max_limit: c_uint = 0;
    let result = loader::nvml_device_get_power_management_limit_constraints(
        device,
        &mut min_limit,
        &mut max_limit,
    )?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok((min_limit, max_limit))
}

pub fn device_get_power_limit(device: NvmlDevice) -> Result<u32> {
    let mut limit: c_uint = 0;
    let result = loader::nvml_device_get_power_management_limit(device, &mut limit)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(limit)
}

pub fn device_get_power_default_limit(device: NvmlDevice) -> Result<u32> {
    let mut default_limit: c_uint = 0;
    let result =
        loader::nvml_device_get_power_management_default_limit(device, &mut default_limit)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(default_limit)
}

pub fn device_set_power_limit(device: NvmlDevice, limit_mw: u32) -> Result<()> {
    let result = loader::nvml_device_set_power_management_limit(device, limit_mw)?;
    if result != NVML_SUCCESS {
        return Err(NvmlError::from_nvml_return(result));
    }
    Ok(())
}
