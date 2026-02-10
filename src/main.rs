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
use nvml::NvmlError;

pub struct AppError {
    domain: &'static str,
    source: Option<NvmlError>,
    message: Option<String>,
    printed: bool,
}

impl AppError {
    pub fn new(domain: &'static str, source: NvmlError) -> Self {
        Self { domain, source: Some(source), message: None, printed: false }
    }

    pub fn msg(domain: &'static str, message: String) -> Self {
        Self { domain, source: None, message: Some(message), printed: false }
    }

    pub fn printed(domain: &'static str) -> Self {
        Self { domain, source: None, message: None, printed: true }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (&self.source, &self.message) {
            (Some(source), _) => write!(f, "error[{}]: {}", self.domain, source.user_message()),
            (_, Some(msg)) => write!(f, "error[{}]: {}", self.domain, msg),
            _ => write!(f, "error[{}]", self.domain),
        }
    }
}

fn run() -> Result<(), AppError> {
    let config = cli::Config::from_args().unwrap_or_else(|e| e.exit());

    if config.operation.modifies_gpu() {
        gpu::validation::check_system_for_modification()
            .map_err(|e| AppError::new("nvoc", e))?;
    }

    let _cleanup = gpu::init_with_cleanup()?;
    let device = gpu::get_device(config.device).map_err(|e| AppError::new("device", e))?;
    gpu::validation::validate_blackwell_architecture(device)
        .map_err(|e| AppError::new("gpu", e))?;

    match config.operation {
        Operation::Info => {
            let version = gpu::driver_version().map_err(|e| AppError::new("driver", e))?;
            println!("driver: {version}");
            gpu::info::show_gpu_info(device, config.device)
                .map_err(|e| AppError::new("info", e))?;
        }
        Operation::Reset { dry_run } => {
            gpu::reset::reset_gpu_settings(device, dry_run)?;
        }
        Operation::Overclock(ref params) => {
            gpu::overclock::apply(device, params)?;
        }
    };

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        if !e.printed {
            eprintln!("{e}");
        }
        process::exit(1);
    }
}
