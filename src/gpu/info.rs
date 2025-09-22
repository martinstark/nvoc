//! GPU information operations

use crate::gpu::domain::{get_power_info, get_power_usage_watts};
use crate::nvml::{
    device_get_clock_info, device_get_clock_offsets, device_get_name, device_get_temperature,
    GpuArchitecture, NvmlClockType, NvmlDevice, Result,
};

pub fn show_gpu_info(device: NvmlDevice, device_index: u32) -> Result<()> {
    let name = device_get_name(device)?;
    let arch = GpuArchitecture::from_device_name(&name);

    println!("{}: {}", device_index, name);
    println!("{:?} v{}", arch, arch.get_clock_offset_version());

    match device_get_clock_info(device, NvmlClockType::Graphics) {
        Ok(clock) => println!("GPU: {}MHz", clock),
        Err(_) => println!("GPU: N/A"),
    }

    match device_get_clock_offsets(device) {
        Ok(offset) => println!("GPU Offset: {}MHz", offset.clockOffsetMHz),
        Err(_) => println!("GPU Offset: N/A"),
    }

    match device_get_clock_info(device, NvmlClockType::Memory) {
        Ok(clock) => println!("Mem: {}MHz", clock),
        Err(_) => println!("Mem: N/A"),
    }

    match device_get_temperature(device) {
        Ok(temp) => println!("Temp: {}°C", temp),
        Err(_) => println!("Temp: N/A"),
    }

    match get_power_usage_watts(device) {
        Ok(power_watts) => println!("Power: {}W", power_watts),
        Err(_) => println!("Power: N/A"),
    }

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
