//! Application constants and configuration values

/// Application metadata
pub mod app {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const NAME: &str = "nvoc";
    pub const AUTHOR: &str = "NVOC Contributors";
    pub const DESCRIPTION: &str = "NVIDIA Blackwell GPU overclocking utility";
}

/// Driver and hardware constraints
pub mod hardware {
    /// Minimum supported NVIDIA driver version
    pub const MIN_DRIVER_VERSION: u32 = 555;

    /// GPU temperature sensor index for NVML calls
    pub const GPU_TEMP_SENSOR: u32 = 0;

    /// Power conversion factor (milliwatts to watts)
    pub const MILLIWATTS_TO_WATTS: u32 = 1000;
}

/// Clock management and validation
pub mod clocks {
    /// Blackwell idle clock range for safe reset (min, max in MHz)
    pub const BLACKWELL_IDLE_MIN: u32 = 200;
    pub const BLACKWELL_IDLE_MAX: u32 = 250;

    /// Default graphics offset for reset operations
    pub const DEFAULT_GRAPHICS_OFFSET: i32 = 0;

    /// Default memory offset for reset operations
    pub const DEFAULT_MEMORY_OFFSET: i32 = 0;
}

/// Buffer sizes for NVML operations
pub mod buffers {
    /// Driver version string buffer size
    pub const DRIVER_VERSION_BUFFER_SIZE: usize = 80;

    /// Device name buffer size (NVML_DEVICE_NAME_V2_BUFFER_SIZE = 96)
    pub const DEVICE_NAME_BUFFER_SIZE: usize = 96;

    /// UUID string buffer size (NVML_DEVICE_UUID_V2_BUFFER_SIZE = 96)
    pub const DEVICE_UUID_BUFFER_SIZE: usize = 96;
}
