//! GPU overclocking operations

use crate::gpu::power::apply_power_limit;
use crate::nvml::{
    device_set_clock_offset, device_set_gpu_locked_clocks, device_set_memory_vf_offset,
    NvmlClockType, NvmlDevice, NvmlPerfState, Result,
};

pub fn apply_clocks(device: NvmlDevice, clocks: (u32, u32), dry_run: bool) -> Result<()> {
    let (min_clock, max_clock) = clocks;

    if dry_run {
        println!("[DRY] Clocks: {}-{}MHz", min_clock, max_clock);
        return Ok(());
    }

    match device_set_gpu_locked_clocks(device, min_clock, max_clock) {
        Ok(_) => {
            println!("Clocks set");
            Ok(())
        }
        Err(e) => {
            eprintln!("Clocks failed: {}", e.actionable_message());
            Err(e)
        }
    }
}

pub fn apply_graphics_offset(device: NvmlDevice, offset: i32, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[DRY] Graphics: {}MHz", offset);
        return Ok(());
    }

    match device_set_clock_offset(device, NvmlClockType::Graphics, NvmlPerfState::P0, offset) {
        Ok(_) => {
            println!("Graphics offset: {}MHz", offset);
            Ok(())
        }
        Err(e) => {
            eprintln!("Graphics offset failed: {}", e.actionable_message());
            Err(e)
        }
    }
}

pub fn apply_memory_offset(device: NvmlDevice, offset: i32, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[DRY] Memory: {}MHz", offset);
        return Ok(());
    }

    match device_set_memory_vf_offset(device, offset) {
        Ok(_) => {
            println!("Memory offset: {}MHz", offset);
            Ok(())
        }
        Err(e) => {
            eprintln!("Memory offset failed: {}", e.actionable_message());
            Err(e)
        }
    }
}

pub fn apply(
    device: NvmlDevice,
    clocks: Option<(u32, u32)>,
    graphics_offset: Option<i32>,
    memory_offset: Option<i32>,
    power_limit: Option<u32>,
    dry_run: bool,
) -> Result<()> {
    if let Some(clocks) = clocks {
        apply_clocks(device, clocks, dry_run)?;
    }

    if let Some(offset) = graphics_offset {
        apply_graphics_offset(device, offset, dry_run)?;
    }

    if let Some(offset) = memory_offset {
        apply_memory_offset(device, offset, dry_run)?;
    }

    if let Some(percentage) = power_limit {
        apply_power_limit(device, percentage, dry_run)?;
    }

    Ok(())
}
