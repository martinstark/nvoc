//! NVOC - NVIDIA GPU overclocking utility for Linux
//!
//! Command-line utility for GPU overclocking using NVML.
//! Designed for RTX 5000 series GPUs with nvidia-open drivers.

use std::process;

mod cli;
mod constants;
mod gpu;
mod nvml;

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

fn run() -> AppResult<()> {
    let config = cli::Config::from_args();

    config.validate()?;

    if config.reset
        || config.clocks.is_some()
        || config.graphics_offset.is_some()
        || config.memory_offset.is_some()
        || config.power_limit.is_some()
    {
        gpu::validation::check_system_for_modification()?;
    }

    // Initialize NVML with automatic cleanup
    let _cleanup = gpu::init_with_cleanup()?;

    // Get and validate device once
    let device_index = config.device;
    let device = gpu::get_device(device_index)?;

    // Validate device architecture (required for all operations)
    gpu::validation::validate_blackwell_architecture(device)?;

    // Execute the requested operation
    match (config.reset, config.info) {
        (true, _) => gpu::reset::reset_gpu_settings(device, config.dry_run)?,
        (_, true) => gpu::info::show_gpu_info(device, device_index)?,
        _ => gpu::overclock::apply(device, &config)?,
    };

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
