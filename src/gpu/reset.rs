//! GPU reset operations
//!
//! Unlike other operations that bail on first error, reset attempts all
//! operations and reports individual failures. Errors are printed at the
//! call site rather than bubbled up because the caller needs to see each
//! failure as it continues through remaining operations.

use crate::constants::clocks;
use crate::gpu::domain::reset_power_limit;
use crate::nvml::{
    device_reset_gpu_locked_clocks, device_reset_memory_locked_clocks, device_set_clock_offset,
    device_set_gpu_locked_clocks, device_set_memory_vf_offset, NvmlClockType, NvmlDevice,
    NvmlError, NvmlPerfState, Result,
};
use crate::AppError;

fn try_reset(domain: &str, f: impl FnOnce() -> Result<()>) -> bool {
    match f() {
        Ok(()) => { println!("{domain}: reset"); true }
        Err(e) => { eprintln!("error[{domain}]: {}", e.user_message()); false }
    }
}

pub fn reset_gpu_settings(device: NvmlDevice, dry_run: bool) -> std::result::Result<(), AppError> {
    if dry_run {
        println!("reset: all (dry run)");
        return Ok(());
    }

    let mut ok = true;

    // Blackwell requires setting idle clocks before reset will succeed
    let idle_ok = device_set_gpu_locked_clocks(device, clocks::BLACKWELL_IDLE_MIN, clocks::BLACKWELL_IDLE_MAX).is_ok();
    if idle_ok {
        ok &= try_reset("gpu clocks", || device_reset_gpu_locked_clocks(device));
    } else {
        eprintln!("error[gpu clocks]: failed to set idle clocks for reset");
        ok = false;
    }

    ok &= try_reset("mem clocks", || device_reset_memory_locked_clocks(device));

    if !try_reset("gpu offset", || {
        device_set_clock_offset(device, NvmlClockType::Graphics, NvmlPerfState::P0, clocks::DEFAULT_GRAPHICS_OFFSET)
    }) {
        eprintln!("  hint: clocks may remain elevated, try sudo nvoc -o 0");
        ok = false;
    }

    ok &= try_reset("mem offset", || {
        device_set_memory_vf_offset(device, clocks::DEFAULT_MEMORY_OFFSET)
    });

    ok &= try_reset("power limit", || reset_power_limit(device));

    if !ok {
        return Err(AppError::printed("reset", NvmlError::NotSupported));
    }

    Ok(())
}
