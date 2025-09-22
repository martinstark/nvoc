//! GPU power management operations

use crate::gpu::domain::{get_power_info, set_power_limit_percentage};
use crate::nvml::{NvmlDevice, Result};

pub fn calculate_power_limit(device: NvmlDevice, percentage: u32) -> Result<u32> {
    let power_info = get_power_info(device)?;
    Ok(power_info.effective_watts_from_percentage(percentage))
}

pub fn apply_power_limit(device: NvmlDevice, percentage: u32, dry_run: bool) -> Result<()> {
    let target_watts = calculate_power_limit(device, percentage)?;

    if dry_run {
        println!("[DRY] Power limit: {}% ({}W)", percentage, target_watts);
        return Ok(());
    }

    match set_power_limit_percentage(device, percentage) {
        Ok(_) => {
            println!("Power limit set to {}% ({}W)", percentage, target_watts);
            Ok(())
        }
        Err(e) => {
            eprintln!("Power limit adjustment failed: {}", e.actionable_message());
            Err(e)
        }
    }
}
