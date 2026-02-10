//! GPU power management operations

use crate::gpu::domain::{get_power_info, w_to_mw};
use crate::nvml::{self, NvmlDevice};
use crate::AppError;

pub fn apply_power_limit(device: NvmlDevice, percentage: u32, dry_run: bool) -> Result<(), AppError> {
    let power_info = get_power_info(device).map_err(|e| AppError::new("power limit", e))?;
    let target_watts = power_info.effective_watts_from_percentage(percentage);

    if dry_run {
        println!("power limit: {percentage}% ({target_watts}W) (dry run)");
        return Ok(());
    }

    nvml::device_set_power_limit(device, w_to_mw(target_watts))
        .map_err(|e| AppError::new("power limit", e))?;
    println!("power limit: {percentage}% ({target_watts}W)");
    Ok(())
}
