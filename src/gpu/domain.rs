//! Domain layer for GPU operations and business logic
//!
//! This module provides high-level abstractions over raw NVML operations,
//! handling unit conversions, business logic, and domain-specific calculations.
//! It keeps the NVML wrapper purely focused on API bindings.

use crate::constants::hardware;
use crate::nvml::{self, NvmlDevice, Result};

/// Power information for a GPU device
#[derive(Debug, Clone)]
pub struct PowerInfo {
    /// Current power limit in watts
    pub limit_watts: u32,
    /// Default (100%) power limit in watts
    pub default_watts: u32,
    /// Minimum allowed power limit in watts
    pub min_watts: u32,
    /// Maximum allowed power limit in watts
    pub max_watts: u32,
}

impl PowerInfo {
    /// Get current power limit as percentage of default
    pub fn current_percentage(&self) -> u32 {
        (self.limit_watts as f32 / self.default_watts as f32 * 100.0) as u32
    }

    /// Calculate watts from percentage of default
    pub fn calculate_watts_from_percentage(&self, percentage: u32) -> u32 {
        (self.default_watts as f32 * percentage as f32 / 100.0) as u32
    }

    /// Get the effective watts that would be set (clamped to hardware limits)
    pub fn effective_watts_from_percentage(&self, percentage: u32) -> u32 {
        let target_watts = self.calculate_watts_from_percentage(percentage);
        target_watts.max(self.min_watts).min(self.max_watts)
    }
}

/// Get comprehensive power information for a device
pub fn get_power_info(device: NvmlDevice) -> Result<PowerInfo> {
    let limit_watts = mw_to_w(nvml::device_get_power_limit(device)?);
    let default_watts = mw_to_w(nvml::device_get_power_default_limit(device)?);
    let (min_mw, max_mw) = nvml::device_get_power_limit_constraints(device)?;

    Ok(PowerInfo {
        limit_watts,
        default_watts,
        min_watts: mw_to_w(min_mw),
        max_watts: mw_to_w(max_mw),
    })
}

/// Get current power usage in watts
pub fn get_power_usage_watts(device: NvmlDevice) -> Result<u32> {
    let power_mw = nvml::device_get_power_usage(device)?;
    Ok(mw_to_w(power_mw))
}

/// Reset power limit to default
pub fn reset_power_limit(device: NvmlDevice) -> Result<()> {
    let default_mw = nvml::device_get_power_default_limit(device)?;
    nvml::device_set_power_limit(device, default_mw)
}

#[inline]
pub fn mw_to_w(milliwatts: u32) -> u32 {
    milliwatts / hardware::MILLIWATTS_TO_WATTS
}

#[inline]
pub fn w_to_mw(watts: u32) -> u32 {
    watts * hardware::MILLIWATTS_TO_WATTS
}
