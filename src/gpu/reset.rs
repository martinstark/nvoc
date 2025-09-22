//! GPU reset operations

use crate::constants::clocks;
use crate::gpu::domain::reset_power_limit;
use crate::nvml::{
    device_get_name, device_reset_gpu_locked_clocks, device_reset_memory_locked_clocks,
    device_set_clock_offset, device_set_gpu_locked_clocks, device_set_memory_vf_offset,
    system_get_driver_version, GpuArchitecture, NvmlClockType, NvmlDevice, NvmlPerfState, Result,
};

pub fn reset_gpu_settings(device: NvmlDevice, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[DRY] Reset");
        return Ok(());
    }

    let mut reset_operations = Vec::new();
    let mut failed_operations = Vec::new();

    // Reset GPU locked clocks - set to idle range first for Blackwell
    let device_name = device_get_name(device)?;
    let _arch = GpuArchitecture::from_device_name(&device_name);

    // For Blackwell, explicitly set to idle range then reset
    let _ = device_set_gpu_locked_clocks(
        device,
        clocks::BLACKWELL_IDLE_MIN,
        clocks::BLACKWELL_IDLE_MAX,
    );

    match device_reset_gpu_locked_clocks(device) {
        Ok(_) => {
            reset_operations.push("GPU locked clocks");
        }
        Err(_) => {
            failed_operations.push("GPU locked clocks");
            eprintln!("GPU clocks reset failed");
        }
    }

    // Reset memory locked clocks
    match device_reset_memory_locked_clocks(device) {
        Ok(_) => {
            reset_operations.push("Memory locked clocks");
        }
        Err(_) => {
            failed_operations.push("Memory locked clocks");
            eprintln!("Memory clocks reset failed");
        }
    }

    // Reset graphics clock offset to 0 (P0 performance state)
    // Reset graphics offset to 0 - critical for Blackwell where offset adds to base clocks
    match device_set_clock_offset(
        device,
        NvmlClockType::Graphics,
        NvmlPerfState::P0,
        clocks::DEFAULT_GRAPHICS_OFFSET,
    ) {
        Ok(_) => {
            reset_operations.push("Graphics offset");
        }
        Err(_) => {
            eprintln!(
                "Graphics offset reset not supported by driver {}",
                system_get_driver_version().unwrap_or_else(|_| "unknown".to_owned())
            );
            eprintln!("Current clocks may remain elevated due to active graphics offset");
            eprintln!("Use: sudo nvoc -o 0 to manually reset graphics offset");
        }
    }

    // Reset memory VF offset to 0
    match device_set_memory_vf_offset(device, clocks::DEFAULT_MEMORY_OFFSET) {
        Ok(_) => {
            reset_operations.push("Memory VF offset");
        }
        Err(_) => {
            failed_operations.push("Memory VF offset");
            eprintln!("Memory offset reset failed");
        }
    }

    // Reset power limit to default
    match reset_power_limit(device) {
        Ok(_) => {
            reset_operations.push("Power limit");
        }
        Err(_) => {
            failed_operations.push("Power limit");
            eprintln!("Power limit reset failed");
        }
    }

    // Print summary
    if !reset_operations.is_empty() {
        println!("Reset: {}", reset_operations.join(", "));
    }

    if !failed_operations.is_empty() {
        eprintln!("Failed: {}", failed_operations.join(", "));
    }

    // Only fail if critical resets (clocks/memory) failed
    if reset_operations.is_empty() && !failed_operations.is_empty() {
        let critical_failed = failed_operations.iter().any(|op| {
            op.contains("GPU locked clocks")
                || op.contains("Memory locked clocks")
                || op.contains("Memory VF offset")
        });

        if critical_failed {
            eprintln!("Critical resets failed");
            return Err(crate::nvml::NvmlError::NotSupported);
        }
    }

    println!("Reset done");
    Ok(())
}
