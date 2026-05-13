//! GPU information display

use crate::gpu::domain::{get_power_info, get_power_usage_watts};
use crate::json;
use crate::nvml::{
    device_get_clock_info, device_get_clock_offsets, device_get_name, device_get_temperature,
    device_get_uuid, NvmlClockType, NvmlDevice, Result,
};

/// Snapshot of all per-GPU info fields. `None` means the underlying NVML query failed
/// (renders as `n/a` in human output and `null` in JSON).
#[derive(Debug)]
pub struct GpuInfoSnapshot {
    pub index: u32,
    pub name: String,
    pub uuid: String,
    pub gpu_clock_mhz: Option<u32>,
    pub gpu_offset_mhz: Option<i32>,
    pub mem_clock_mhz: Option<u32>,
    pub mem_offset_mhz: Option<i32>,
    pub temp_c: Option<u32>,
    pub power_w: Option<u32>,
    pub power_limit_w: Option<u32>,
    pub power_limit_percent: Option<u32>,
    pub power_min_w: Option<u32>,
    pub power_default_w: Option<u32>,
    pub power_max_w: Option<u32>,
}

/// Read all info fields from a device. The `name` field is required and bubbles up
/// errors; everything else degrades to `None`.
pub fn collect_gpu_info(device: NvmlDevice, index: u32) -> Result<GpuInfoSnapshot> {
    let name = device_get_name(device)?;
    let uuid = device_get_uuid(device).unwrap_or_default();
    let power = get_power_info(device).ok();

    Ok(GpuInfoSnapshot {
        index,
        name,
        uuid,
        gpu_clock_mhz: device_get_clock_info(device, NvmlClockType::Graphics).ok(),
        gpu_offset_mhz: device_get_clock_offsets(device, NvmlClockType::Graphics)
            .ok()
            .map(|o| o.clockOffsetMHz),
        mem_clock_mhz: device_get_clock_info(device, NvmlClockType::Memory).ok(),
        mem_offset_mhz: device_get_clock_offsets(device, NvmlClockType::Memory)
            .ok()
            .map(|o| o.clockOffsetMHz),
        temp_c: device_get_temperature(device).ok(),
        power_w: get_power_usage_watts(device).ok(),
        power_limit_w: power.as_ref().map(|p| p.limit_watts),
        power_limit_percent: power.as_ref().map(|p| p.current_percentage()),
        power_min_w: power.as_ref().map(|p| p.min_watts),
        power_default_w: power.as_ref().map(|p| p.default_watts),
        power_max_w: power.as_ref().map(|p| p.max_watts),
    })
}

fn show_field<T: std::fmt::Display>(label: &str, unit: &str, val: Option<T>) {
    match val {
        Some(v) => println!("{label}: {v}{unit}"),
        None => println!("{label}: n/a"),
    }
}

pub fn show_human(snap: &GpuInfoSnapshot) {
    println!("gpu {}: {}", snap.index, snap.name);
    show_field("gpu clock", "MHz", snap.gpu_clock_mhz);
    show_field("gpu offset", "MHz", snap.gpu_offset_mhz);
    show_field("mem clock", "MHz", snap.mem_clock_mhz);
    show_field("mem offset", "MHz", snap.mem_offset_mhz);
    show_field("temp", "°C", snap.temp_c);
    show_field("power", "W", snap.power_w);

    match (snap.power_limit_w, snap.power_limit_percent) {
        (Some(w), Some(pct)) => println!("power limit: {w}W ({pct}%)"),
        _ => println!("power limit: n/a"),
    }
    if let (Some(mn), Some(def), Some(mx)) =
        (snap.power_min_w, snap.power_default_w, snap.power_max_w)
    {
        println!("power range: {mn}W-{def}W ({mx}W hard limit)");
    }
}

pub fn render_json(snap: &GpuInfoSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"index\":{idx},",
            "\"name\":{name},",
            "\"uuid\":{uuid},",
            "\"gpu_clock_mhz\":{gc},",
            "\"gpu_offset_mhz\":{go},",
            "\"mem_clock_mhz\":{mc},",
            "\"mem_offset_mhz\":{mo},",
            "\"temp_c\":{t},",
            "\"power_w\":{p},",
            "\"power_limit_w\":{plw},",
            "\"power_limit_percent\":{plp},",
            "\"power_min_w\":{pmin},",
            "\"power_default_w\":{pdef},",
            "\"power_max_w\":{pmax}",
            "}}"
        ),
        idx = snap.index,
        name = json::quoted(&snap.name),
        uuid = json::quoted(&snap.uuid),
        gc = json::opt_num(snap.gpu_clock_mhz),
        go = json::opt_num(snap.gpu_offset_mhz),
        mc = json::opt_num(snap.mem_clock_mhz),
        mo = json::opt_num(snap.mem_offset_mhz),
        t = json::opt_num(snap.temp_c),
        p = json::opt_num(snap.power_w),
        plw = json::opt_num(snap.power_limit_w),
        plp = json::opt_num(snap.power_limit_percent),
        pmin = json::opt_num(snap.power_min_w),
        pdef = json::opt_num(snap.power_default_w),
        pmax = json::opt_num(snap.power_max_w),
    )
}

/// Render a device that failed validation or handle resolution as a JSON entry.
pub fn render_json_error(index: u32, name: &str, message: &str) -> String {
    format!(
        "{{\"index\":{idx},\"name\":{name},\"error\":{err}}}",
        idx = index,
        name = json::quoted(name),
        err = json::quoted(message),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_snapshot(index: u32) -> GpuInfoSnapshot {
        GpuInfoSnapshot {
            index,
            name: format!("NVIDIA GeForce RTX 5090 #{index}"),
            uuid: format!("GPU-abc-{index}"),
            gpu_clock_mhz: Some(1080),
            gpu_offset_mhz: Some(856),
            mem_clock_mhz: Some(810),
            mem_offset_mhz: Some(2000),
            temp_c: Some(51),
            power_w: Some(39),
            power_limit_w: Some(600),
            power_limit_percent: Some(104),
            power_min_w: Some(400),
            power_default_w: Some(575),
            power_max_w: Some(600),
        }
    }

    fn empty_snapshot(index: u32) -> GpuInfoSnapshot {
        GpuInfoSnapshot {
            index,
            name: "GPU".into(),
            uuid: "GPU-x".into(),
            gpu_clock_mhz: None,
            gpu_offset_mhz: None,
            mem_clock_mhz: None,
            mem_offset_mhz: None,
            temp_c: None,
            power_w: None,
            power_limit_w: None,
            power_limit_percent: None,
            power_min_w: None,
            power_default_w: None,
            power_max_w: None,
        }
    }

    /// Naive JSON validator: each `{` must be balanced by `}`, `[`/`]` likewise,
    /// and quoted strings are skipped. Returns true if the document is balanced.
    fn json_balanced(s: &str) -> bool {
        let mut depth_curly = 0i32;
        let mut depth_square = 0i32;
        let mut in_str = false;
        let mut escaped = false;
        for c in s.chars() {
            if in_str {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_str = false;
                }
                continue;
            }
            match c {
                '"' => in_str = true,
                '{' => depth_curly += 1,
                '}' => depth_curly -= 1,
                '[' => depth_square += 1,
                ']' => depth_square -= 1,
                _ => {}
            }
            if depth_curly < 0 || depth_square < 0 {
                return false;
            }
        }
        depth_curly == 0 && depth_square == 0 && !in_str
    }

    #[test]
    fn render_json_balanced_with_full_snapshot() {
        let s = render_json(&full_snapshot(0));
        assert!(json_balanced(&s), "unbalanced: {s}");
    }

    #[test]
    fn render_json_balanced_with_all_none_fields() {
        let s = render_json(&empty_snapshot(1));
        assert!(json_balanced(&s), "unbalanced: {s}");
    }

    #[test]
    fn render_json_emits_null_for_none_fields() {
        let s = render_json(&empty_snapshot(0));
        assert!(s.contains("\"gpu_clock_mhz\":null"), "got {s}");
        assert!(s.contains("\"temp_c\":null"), "got {s}");
        assert!(s.contains("\"power_limit_percent\":null"), "got {s}");
    }

    #[test]
    fn render_json_emits_numbers_for_some_fields() {
        let s = render_json(&full_snapshot(0));
        assert!(s.contains("\"gpu_clock_mhz\":1080"), "got {s}");
        assert!(s.contains("\"gpu_offset_mhz\":856"), "got {s}");
        assert!(s.contains("\"temp_c\":51"), "got {s}");
    }

    #[test]
    fn render_json_includes_all_expected_keys() {
        let s = render_json(&full_snapshot(0));
        for key in [
            "\"index\"",
            "\"name\"",
            "\"uuid\"",
            "\"gpu_clock_mhz\"",
            "\"gpu_offset_mhz\"",
            "\"mem_clock_mhz\"",
            "\"mem_offset_mhz\"",
            "\"temp_c\"",
            "\"power_w\"",
            "\"power_limit_w\"",
            "\"power_limit_percent\"",
            "\"power_min_w\"",
            "\"power_default_w\"",
            "\"power_max_w\"",
        ] {
            assert!(s.contains(key), "missing {key} in {s}");
        }
    }

    #[test]
    fn render_json_quotes_string_fields() {
        let s = render_json(&full_snapshot(2));
        assert!(s.contains("\"name\":\"NVIDIA GeForce RTX 5090 #2\""), "got {s}");
        assert!(s.contains("\"uuid\":\"GPU-abc-2\""), "got {s}");
    }

    #[test]
    fn render_json_escapes_quotes_in_name() {
        let snap = GpuInfoSnapshot {
            name: "weird\"name".into(),
            ..full_snapshot(0)
        };
        let s = render_json(&snap);
        assert!(s.contains("\"name\":\"weird\\\"name\""), "got {s}");
        assert!(json_balanced(&s), "unbalanced: {s}");
    }

    #[test]
    fn render_json_uses_correct_index() {
        let s = render_json(&full_snapshot(7));
        assert!(s.contains("\"index\":7"), "got {s}");
    }

    #[test]
    fn render_json_error_balanced_and_well_formed() {
        let s = render_json_error(3, "unknown", "not supported by this gpu");
        assert!(json_balanced(&s), "unbalanced: {s}");
        assert!(s.contains("\"index\":3"));
        assert!(s.contains("\"name\":\"unknown\""));
        assert!(s.contains("\"error\":\"not supported by this gpu\""));
    }

    #[test]
    fn multi_gpu_array_concatenation_stays_balanced() {
        // Simulates how main.rs joins per-device JSON for `info --json`. This is the
        // path that runs on systems with >1 GPU; we can't exercise it on hardware here
        // but we can verify the building blocks compose into valid JSON.
        let entries: Vec<String> = (0..4).map(|i| render_json(&full_snapshot(i))).collect();
        let doc = format!(
            "{{\"driver\":\"595.71.05\",\"gpus\":[{}]}}",
            entries.join(",")
        );
        assert!(json_balanced(&doc), "unbalanced multi-gpu doc: {doc}");
        // each index appears exactly once
        for i in 0..4 {
            let needle = format!("\"index\":{i}");
            assert_eq!(
                doc.matches(&needle).count(),
                1,
                "index {i} not unique in {doc}"
            );
        }
    }

    #[test]
    fn multi_gpu_array_with_mixed_success_and_error_entries() {
        let mut entries = Vec::new();
        entries.push(render_json(&full_snapshot(0)));
        entries.push(render_json_error(1, "Some GPU", "not supported by this gpu"));
        entries.push(render_json(&empty_snapshot(2)));
        let doc = format!(
            "{{\"driver\":\"595.71.05\",\"gpus\":[{}]}}",
            entries.join(",")
        );
        assert!(json_balanced(&doc), "unbalanced: {doc}");
        assert!(doc.contains("\"index\":0"));
        assert!(doc.contains("\"index\":1"));
        assert!(doc.contains("\"index\":2"));
        assert!(doc.contains("\"error\":\"not supported by this gpu\""));
    }
}
