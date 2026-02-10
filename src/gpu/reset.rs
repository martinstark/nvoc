//! GPU reset operations

use crate::constants::clocks;
use crate::gpu::domain::reset_power_limit;
use crate::nvml::{
    device_reset_gpu_locked_clocks, device_reset_memory_locked_clocks, device_set_clock_offset,
    device_set_gpu_locked_clocks, device_set_memory_vf_offset, system_get_driver_version,
    NvmlClockType, NvmlDevice, NvmlPerfState, Result,
};

fn try_reset<'a>(label: &'a str, succeeded: &mut Vec<&'a str>, f: impl FnOnce() -> Result<()>) -> bool {
    match f() {
        Ok(_) => {
            succeeded.push(label);
            true
        }
        Err(e) => {
            eprintln!("{} failed: {}", label, e.actionable_message());
            false
        }
    }
}

pub fn reset_gpu_settings(device: NvmlDevice, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[DRY] Reset");
        return Ok(());
    }

    let mut failed = false;
    let mut succeeded = Vec::new();

    // Set to idle range before resetting locked clocks (required for Blackwell)
    if try_reset("Idle clocks pre-set", &mut succeeded, || {
        device_set_gpu_locked_clocks(device, clocks::BLACKWELL_IDLE_MIN, clocks::BLACKWELL_IDLE_MAX)
    }) {
        if !try_reset("GPU locked clocks", &mut succeeded, || {
            device_reset_gpu_locked_clocks(device)
        }) {
            failed = true;
        }
    } else {
        failed = true;
    }

    if !try_reset("Memory locked clocks", &mut succeeded, || {
        device_reset_memory_locked_clocks(device)
    }) {
        failed = true;
    }

    if !try_reset("Graphics offset", &mut succeeded, || {
        device_set_clock_offset(
            device,
            NvmlClockType::Graphics,
            NvmlPerfState::P0,
            clocks::DEFAULT_GRAPHICS_OFFSET,
        )
    }) {
        eprintln!(
            "Driver: {}",
            system_get_driver_version().unwrap_or_else(|_| "unknown".to_owned())
        );
        eprintln!("Clocks may remain elevated due to active graphics offset");
        eprintln!("Manual reset: sudo nvoc -o 0");
        failed = true;
    }

    if !try_reset("Memory VF offset", &mut succeeded, || {
        device_set_memory_vf_offset(device, clocks::DEFAULT_MEMORY_OFFSET)
    }) {
        failed = true;
    }

    if !try_reset("Power limit", &mut succeeded, || reset_power_limit(device)) {
        failed = true;
    }

    if !succeeded.is_empty() {
        println!("Reset: {}", succeeded.join(", "));
    }

    if failed {
        return Err(crate::nvml::NvmlError::NotSupported);
    }

    Ok(())
}
