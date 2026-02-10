//! GPU information display

use crate::gpu::domain::{get_power_info, get_power_usage_watts};
use crate::nvml::{
    device_get_clock_info, device_get_clock_offsets, device_get_name, device_get_temperature,
    NvmlClockType, NvmlDevice, Result,
};

fn print_field<T: std::fmt::Display>(label: &str, unit: &str, result: Result<T>) {
    match result {
        Ok(val) => println!("{label}: {val}{unit}"),
        Err(_) => println!("{label}: n/a"),
    }
}

/// Display GPU info. Only device name is required; individual fields
/// degrade to "n/a" on error via print_field.
pub fn show_gpu_info(device: NvmlDevice, device_index: u32) -> Result<()> {
    let name = device_get_name(device)?;
    println!("gpu {device_index}: {name}");

    print_field("gpu clock", "MHz", device_get_clock_info(device, NvmlClockType::Graphics));
    print_field("gpu offset", "MHz", device_get_clock_offsets(device, NvmlClockType::Graphics).map(|o| o.clockOffsetMHz));
    print_field("mem clock", "MHz", device_get_clock_info(device, NvmlClockType::Memory));
    print_field("mem offset", "MHz", device_get_clock_offsets(device, NvmlClockType::Memory).map(|o| o.clockOffsetMHz));
    print_field("temp", "°C", device_get_temperature(device));
    print_field("power", "W", get_power_usage_watts(device));

    match get_power_info(device) {
        Ok(info) => {
            println!("power limit: {}W ({}%)", info.limit_watts, info.current_percentage());
            println!("power range: {}W-{}W ({}W hard limit)", info.min_watts, info.default_watts, info.max_watts);
        }
        Err(_) => println!("power limit: n/a"),
    }

    Ok(())
}
