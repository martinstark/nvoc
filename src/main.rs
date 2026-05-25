//! NVOC - NVIDIA GPU overclocking utility for Linux
//!
//! Command-line utility for GPU overclocking using NVML.
//! Designed for Blackwell GPUs with nvidia-open drivers.

use std::process;

mod cli;
mod constants;
mod gpu;
mod json;
mod nvml;

use cli::{ListFormat, Operation};
use gpu::info::{collect_gpu_info, render_json, render_json_error, show_human};
use nvml::NvmlDevice;
use nvml::NvmlError;

pub struct AppError {
    domain: &'static str,
    source: Option<NvmlError>,
    message: Option<String>,
    printed: bool,
}

impl AppError {
    pub fn new(domain: &'static str, source: NvmlError) -> Self {
        Self {
            domain,
            source: Some(source),
            message: None,
            printed: false,
        }
    }

    pub fn msg(domain: &'static str, message: String) -> Self {
        Self {
            domain,
            source: None,
            message: Some(message),
            printed: false,
        }
    }

    pub fn printed(domain: &'static str) -> Self {
        Self {
            domain,
            source: None,
            message: None,
            printed: true,
        }
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

fn run_list(format: ListFormat) -> Result<(), AppError> {
    let entries = gpu::list_all().map_err(|e| AppError::new("list", e))?;
    match format {
        ListFormat::Human => {
            for e in &entries {
                println!("{} - {} - {}", e.index, e.name, e.uuid);
            }
        }
        ListFormat::UuidOnly => {
            for e in &entries {
                println!("{}", e.uuid);
            }
        }
        ListFormat::Json => println!("{}", gpu::render_list_json(&entries)),
    }
    Ok(())
}

fn run_info(devices: &[(u32, NvmlDevice)], as_json: bool) -> bool {
    let driver = gpu::driver_version().ok();
    let mut all_ok = true;

    if as_json {
        let mut entries: Vec<String> = Vec::with_capacity(devices.len());
        for &(idx, dev) in devices {
            match gpu::validation::validate_blackwell_architecture(dev) {
                Ok(()) => match collect_gpu_info(dev, idx) {
                    Ok(snap) => entries.push(render_json(&snap)),
                    Err(e) => {
                        all_ok = false;
                        entries.push(render_json_error(idx, "unknown", e.user_message()));
                    }
                },
                Err(e) => {
                    all_ok = false;
                    let name = nvml::device_get_name(dev).unwrap_or_else(|_| "unknown".into());
                    entries.push(render_json_error(idx, &name, e.user_message()));
                }
            }
        }
        let driver_field = match driver {
            Some(d) => json::quoted(&d),
            None => "null".into(),
        };
        println!(
            "{{\"driver\":{driver_field},\"gpus\":[{}]}}",
            entries.join(",")
        );
    } else {
        match driver {
            Some(d) => println!("driver: {d}"),
            None => println!("driver: n/a"),
        }
        for &(idx, dev) in devices {
            match gpu::validation::validate_blackwell_architecture(dev) {
                Ok(()) => match collect_gpu_info(dev, idx) {
                    Ok(snap) => show_human(&snap),
                    Err(e) => {
                        all_ok = false;
                        eprintln!("error[gpu {idx}]: {}", e.user_message());
                    }
                },
                Err(e) => {
                    all_ok = false;
                    eprintln!("error[gpu {idx}]: {}", e.user_message());
                }
            }
        }
    }

    all_ok
}

/// Run a per-device action, isolating failures and tracking aggregate success.
fn for_each_device<F>(devices: &[(u32, NvmlDevice)], mut f: F) -> bool
where
    F: FnMut(u32, NvmlDevice) -> Result<(), AppError>,
{
    let mut all_ok = true;
    for &(idx, dev) in devices {
        match gpu::validation::validate_blackwell_architecture(dev) {
            Ok(()) => {
                if let Err(e) = f(idx, dev) {
                    all_ok = false;
                    if !e.printed {
                        eprintln!("{e}");
                    }
                }
            }
            Err(e) => {
                all_ok = false;
                eprintln!("error[gpu {idx}]: {}", e.user_message());
            }
        }
    }
    all_ok
}

fn run() -> Result<(), AppError> {
    let config = cli::Config::from_args().unwrap_or_else(|e| e.exit());

    if config.operation.requires_root() {
        gpu::validation::check_system_for_modification().map_err(|e| AppError::new("nvoc", e))?;
    }

    let _cleanup = gpu::init_with_cleanup()?;

    if let Operation::List(format) = config.operation {
        return run_list(format);
    }

    let devices = gpu::resolve_devices(&config.devices)?;

    if devices.is_empty() {
        return Err(AppError::msg("device", "no devices selected".into()));
    }

    let all_ok = match config.operation {
        Operation::Info { json } => run_info(&devices, json),
        Operation::Reset { dry_run } => {
            // reset prints "gpu N: name" header per device for clarity in multi-GPU runs
            for_each_device(&devices, |idx, dev| {
                let name = nvml::device_get_name(dev).unwrap_or_else(|_| "unknown".into());
                println!("gpu {idx}: {name}");
                gpu::reset::reset_gpu_settings(dev, dry_run)
            })
        }
        Operation::Overclock(ref params) => for_each_device(&devices, |idx, dev| {
            let name = nvml::device_get_name(dev).unwrap_or_else(|_| "unknown".into());
            println!("gpu {idx}: {name}");
            gpu::overclock::apply(dev, params)
        }),
        Operation::List(_) => unreachable!("handled above"),
    };

    if !all_ok {
        return Err(AppError::printed("aggregate"));
    }
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
