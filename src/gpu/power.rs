//! GPU power management operations

use crate::gpu::domain::{get_power_info, w_to_mw};
use crate::nvml::{self, NvmlDevice, Result};

pub fn apply_power_limit(device: NvmlDevice, percentage: u32, dry_run: bool) -> Result<()> {
    let power_info = get_power_info(device)?;
    let target_watts = power_info.effective_watts_from_percentage(percentage);

    if dry_run {
        println!("[DRY] Power limit: {}% ({}W)", percentage, target_watts);
        return Ok(());
    }

    match nvml::device_set_power_limit(device, w_to_mw(target_watts)) {
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
