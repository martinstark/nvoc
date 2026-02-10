//! GPU information operations

use crate::gpu::domain::{get_power_info, get_power_usage_watts};
use crate::nvml::{
    device_get_clock_info, device_get_clock_offsets, device_get_name, device_get_temperature,
    GpuArchitecture, NvmlClockType, NvmlDevice, Result,
};

fn print_field<T: std::fmt::Display>(label: &str, unit: &str, result: Result<T>) {
    match result {
        Ok(val) => println!("{}: {}{}", label, val, unit),
        Err(_) => println!("{}: N/A", label),
    }
}

pub fn show_gpu_info(device: NvmlDevice, device_index: u32) -> Result<()> {
    let name = device_get_name(device)?;
    let arch = GpuArchitecture::from_device_name(&name);

    println!("{}: {}", device_index, name);
    println!("Arch: {:?}", arch);

    print_field("GPU", "MHz", device_get_clock_info(device, NvmlClockType::Graphics));
    print_field("GPU Offset", "MHz", device_get_clock_offsets(device).map(|o| o.clockOffsetMHz));
    print_field("Mem", "MHz", device_get_clock_info(device, NvmlClockType::Memory));
    print_field("Temp", "°C", device_get_temperature(device));
    print_field("Power", "W", get_power_usage_watts(device));

    match get_power_info(device) {
        Ok(power_info) => {
            print!("Power Limit: {}W", power_info.limit_watts);
            let percentage = power_info.current_percentage();
            print!(" ({}% of default)", percentage);
            println!();

            println!(
                "Power Range: {}W-{}W (hard limit: {}W)",
                power_info.min_watts, power_info.default_watts, power_info.max_watts
            );
        }
        Err(_) => println!("Power Limit: N/A"),
    }

    Ok(())
}
