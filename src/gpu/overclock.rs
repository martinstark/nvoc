//! GPU overclocking operations

use crate::cli::Operation;
use crate::gpu::power::apply_power_limit;
use crate::nvml::{
    device_set_clock_offset, device_set_gpu_locked_clocks, device_set_memory_vf_offset,
    NvmlClockType, NvmlDevice, NvmlPerfState,
};
use crate::AppError;

fn apply_clocks(device: NvmlDevice, clocks: (u32, u32), dry_run: bool) -> Result<(), AppError> {
    let (min, max) = clocks;
    if dry_run {
        println!("clocks: {min}-{max}MHz (dry run)");
        return Ok(());
    }
    device_set_gpu_locked_clocks(device, min, max)
        .map_err(|e| AppError::new("clocks", e))?;
    println!("clocks: {min}-{max}MHz");
    Ok(())
}

fn apply_graphics_offset(device: NvmlDevice, offset: i32, dry_run: bool) -> Result<(), AppError> {
    if dry_run {
        println!("gpu offset: {:+}MHz (dry run)", offset);
        return Ok(());
    }
    device_set_clock_offset(device, NvmlClockType::Graphics, NvmlPerfState::P0, offset)
        .map_err(|e| AppError::new("gpu offset", e))?;
    println!("gpu offset: {:+}MHz", offset);
    Ok(())
}

fn apply_memory_offset(device: NvmlDevice, offset: i32, dry_run: bool) -> Result<(), AppError> {
    if dry_run {
        println!("mem offset: {:+}MHz (dry run)", offset);
        return Ok(());
    }
    device_set_memory_vf_offset(device, offset)
        .map_err(|e| AppError::new("mem offset", e))?;
    println!("mem offset: {:+}MHz", offset);
    Ok(())
}

pub fn apply(device: NvmlDevice, op: &Operation) -> Result<(), AppError> {
    let Operation::Overclock { clocks, graphics_offset, memory_offset, power_limit, dry_run } = op
    else { unreachable!() };

    if let Some(clocks) = clocks {
        apply_clocks(device, *clocks, *dry_run)?;
    }
    if let Some(offset) = graphics_offset {
        apply_graphics_offset(device, *offset, *dry_run)?;
    }
    if let Some(offset) = memory_offset {
        apply_memory_offset(device, *offset, *dry_run)?;
    }
    if let Some(percentage) = power_limit {
        apply_power_limit(device, *percentage, *dry_run)?;
    }
    Ok(())
}
