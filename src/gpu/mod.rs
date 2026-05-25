//! GPU operations and device management

use crate::cli::{DeviceRef, Devices};
use crate::constants::hardware;
use crate::json;
use crate::nvml::{
    device_get_count, device_get_handle_by_index, device_get_handle_by_uuid, device_get_index,
    device_get_name, device_get_uuid, init, shutdown, system_get_driver_version, NvmlDevice,
    Result,
};

pub mod domain;
pub mod info;
pub mod overclock;
pub mod power;
pub mod reset;
pub mod validation;

/// Cleanup guard to ensure NVML is properly shut down
pub struct CleanupGuard;

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        let _ = shutdown();
    }
}

pub fn init_nvml() -> std::result::Result<(), crate::AppError> {
    init().map_err(|e| crate::AppError::new("driver", e))?;
    let driver_version =
        system_get_driver_version().map_err(|e| crate::AppError::new("driver", e))?;
    let major: u32 = driver_version
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| {
            crate::AppError::msg("driver", format!("unparseable version: {driver_version}"))
        })?;
    if major < hardware::MIN_DRIVER_VERSION {
        return Err(crate::AppError::msg(
            "driver",
            format!(
                "version {driver_version} too old, need {}+",
                hardware::MIN_DRIVER_VERSION
            ),
        ));
    }
    Ok(())
}

pub fn init_with_cleanup() -> std::result::Result<CleanupGuard, crate::AppError> {
    init_nvml()?;
    Ok(CleanupGuard)
}

pub fn driver_version() -> Result<String> {
    system_get_driver_version()
}

/// A device entry returned by `nvoc list`.
#[derive(Debug)]
pub struct DeviceListing {
    pub index: u32,
    pub name: String,
    pub uuid: String,
}

/// Render a slice of device listings as a JSON array.
pub fn render_list_json(entries: &[DeviceListing]) -> String {
    let items: Vec<String> = entries
        .iter()
        .map(|e| {
            format!(
                "{{\"index\":{i},\"name\":{n},\"uuid\":{u}}}",
                i = e.index,
                n = json::quoted(&e.name),
                u = json::quoted(&e.uuid),
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

/// Enumerate all NVML-visible GPUs. Does not run architecture validation.
pub fn list_all() -> Result<Vec<DeviceListing>> {
    let count = device_get_count()?;
    let mut out = Vec::with_capacity(count as usize);
    for i in 0..count {
        let dev = device_get_handle_by_index(i)?;
        let name = device_get_name(dev).unwrap_or_else(|_| "unknown".into());
        let uuid = device_get_uuid(dev).unwrap_or_else(|_| String::new());
        out.push(DeviceListing {
            index: i,
            name,
            uuid,
        });
    }
    Ok(out)
}

/// Resolve a `Devices` spec into ordered `(index, handle)` pairs, deduplicating.
///
/// Strict semantics: every selector must resolve to at least one device. Out-of-range
/// indices, unknown UUIDs, and zero-match name patterns are all hard errors.
pub fn resolve_devices(
    spec: &Devices,
) -> std::result::Result<Vec<(u32, NvmlDevice)>, crate::AppError> {
    let count = device_get_count().map_err(|e| crate::AppError::new("device", e))?;

    let mut pairs: Vec<(u32, NvmlDevice)> = Vec::new();

    match spec {
        Devices::All => {
            for i in 0..count {
                let h =
                    device_get_handle_by_index(i).map_err(|e| crate::AppError::new("device", e))?;
                pairs.push((i, h));
            }
        }
        Devices::List(refs) => {
            for r in refs {
                match r {
                    DeviceRef::Index(i) => {
                        if *i >= count {
                            return Err(crate::AppError::msg(
                                "device",
                                format!("device index {i} out of range (have {count})"),
                            ));
                        }
                        let h = device_get_handle_by_index(*i)
                            .map_err(|e| crate::AppError::new("device", e))?;
                        pairs.push((*i, h));
                    }
                    DeviceRef::Uuid(uuid) => {
                        let h = device_get_handle_by_uuid(uuid)
                            .map_err(|e| crate::AppError::new("device", e))?;
                        let i =
                            device_get_index(h).map_err(|e| crate::AppError::new("device", e))?;
                        pairs.push((i, h));
                    }
                    DeviceRef::Name(pattern) => {
                        let matches = resolve_name(pattern, count)
                            .map_err(|e| crate::AppError::new("device", e))?;
                        if matches.is_empty() {
                            return Err(crate::AppError::msg(
                                "device",
                                format!("no devices matched name '{pattern}'"),
                            ));
                        }
                        pairs.extend(matches);
                    }
                    DeviceRef::Regex(re) => {
                        let matches = resolve_regex(re, count)
                            .map_err(|e| crate::AppError::new("device", e))?;
                        if matches.is_empty() {
                            return Err(crate::AppError::msg(
                                "device",
                                format!("no devices matched regex '{}'", re.as_str()),
                            ));
                        }
                        pairs.extend(matches);
                    }
                }
            }
        }
    }

    // Dedup by index. Multiple selectors can resolve to the same device (e.g. an index
    // that also matches a name pattern); keep the first occurrence.
    let mut seen = std::collections::HashSet::new();
    let mut deduped = Vec::with_capacity(pairs.len());
    for (idx, h) in pairs {
        if seen.insert(idx) {
            deduped.push((idx, h));
        }
    }
    Ok(deduped)
}

/// Find all devices whose name contains `pattern` (case-insensitive substring).
/// Returns matches in NVML index order.
fn resolve_name(pattern: &str, count: u32) -> Result<Vec<(u32, NvmlDevice)>> {
    let needle = pattern.to_ascii_lowercase();
    let mut out = Vec::new();
    for i in 0..count {
        let h = device_get_handle_by_index(i)?;
        let name = device_get_name(h)?;
        if name.to_ascii_lowercase().contains(&needle) {
            out.push((i, h));
        }
    }
    Ok(out)
}

/// Find all devices whose name matches `re` (case-sensitive regex search).
/// Returns matches in NVML index order.
fn resolve_regex(re: &regex::Regex, count: u32) -> Result<Vec<(u32, NvmlDevice)>> {
    let mut out = Vec::new();
    for i in 0..count {
        let h = device_get_handle_by_index(i)?;
        let name = device_get_name(h)?;
        if re.is_match(&name) {
            out.push((i, h));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(i: u32, name: &str, uuid: &str) -> DeviceListing {
        DeviceListing {
            index: i,
            name: name.into(),
            uuid: uuid.into(),
        }
    }

    #[test]
    fn render_list_json_empty() {
        assert_eq!(render_list_json(&[]), "[]");
    }

    #[test]
    fn render_list_json_single_entry() {
        let s = render_list_json(&[listing(0, "NVIDIA GeForce RTX 5090", "GPU-abc")]);
        assert_eq!(
            s,
            "[{\"index\":0,\"name\":\"NVIDIA GeForce RTX 5090\",\"uuid\":\"GPU-abc\"}]"
        );
    }

    #[test]
    fn render_list_json_multiple_entries_have_correct_separators() {
        let s = render_list_json(&[
            listing(0, "GPU A", "GPU-aaa"),
            listing(1, "GPU B", "GPU-bbb"),
            listing(2, "GPU C", "GPU-ccc"),
        ]);
        // Three objects, two commas between them.
        assert_eq!(s.matches("},{").count(), 2, "got {s}");
        assert_eq!(s.matches("\"index\":").count(), 3, "got {s}");
        assert!(s.starts_with('['));
        assert!(s.ends_with(']'));
    }

    #[test]
    fn render_list_json_escapes_names() {
        let s = render_list_json(&[listing(0, "weird\"name", "GPU-x")]);
        assert!(s.contains("\"name\":\"weird\\\"name\""), "got {s}");
    }
}
