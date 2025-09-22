//! GPU overclocking operations

use crate::cli::Config;
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
        Ok(_) => Ok(()),
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

pub fn apply(device: NvmlDevice, config: &Config) -> Result<()> {
    if let Some(clocks) = config.clocks {
        apply_clocks(device, clocks, config.dry_run)?;
    }

    if let Some(offset) = config.graphics_offset {
        apply_graphics_offset(device, offset, config.dry_run)?;
    }

    if let Some(offset) = config.memory_offset {
        apply_memory_offset(device, offset, config.dry_run)?;
    }

    if let Some(percentage) = config.power_limit {
        apply_power_limit(device, percentage, config.dry_run)?;
    }

    Ok(())
}
