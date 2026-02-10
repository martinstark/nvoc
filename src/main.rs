//! NVOC - NVIDIA GPU overclocking utility for Linux
//!
//! Command-line utility for GPU overclocking using NVML.
//! Designed for RTX 5000 series GPUs with nvidia-open drivers.

use std::process;

mod cli;
mod constants;
mod gpu;
mod nvml;

use cli::Operation;

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

fn run() -> AppResult<()> {
    let config = cli::Config::from_args()?;

    if config.operation.modifies_gpu() {
        gpu::validation::check_system_for_modification()?;
    }

    let _cleanup = gpu::init_with_cleanup()?;
    let device = gpu::get_device(config.device)?;
    gpu::validation::validate_blackwell_architecture(device)?;

    match config.operation {
        Operation::Info => gpu::info::show_gpu_info(device, config.device)?,
        Operation::Reset { dry_run } => gpu::reset::reset_gpu_settings(device, dry_run)?,
        Operation::Overclock {
            clocks,
            graphics_offset,
            memory_offset,
            power_limit,
            dry_run,
        } => gpu::overclock::apply(device, clocks, graphics_offset, memory_offset, power_limit, dry_run)?,
    };

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
