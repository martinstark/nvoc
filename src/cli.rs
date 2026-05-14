//! Command-line interface parsing and configuration

use crate::constants::app;
use clap::{Arg, ArgAction, Command};
use regex::Regex;

#[derive(Debug, Clone)]
pub enum DeviceRef {
    Index(u32),
    Uuid(String),
    Name(String),
    Regex(Regex),
}

impl PartialEq for DeviceRef {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DeviceRef::Index(a), DeviceRef::Index(b)) => a == b,
            (DeviceRef::Uuid(a), DeviceRef::Uuid(b)) => a == b,
            (DeviceRef::Name(a), DeviceRef::Name(b)) => a.eq_ignore_ascii_case(b),
            (DeviceRef::Regex(a), DeviceRef::Regex(b)) => a.as_str() == b.as_str(),
            _ => false,
        }
    }
}

impl Eq for DeviceRef {}

#[derive(Debug, Clone)]
pub enum Devices {
    All,
    List(Vec<DeviceRef>),
}

impl Default for Devices {
    fn default() -> Self {
        Devices::List(vec![DeviceRef::Index(0)])
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ListFormat {
    Human,
    UuidOnly,
    Json,
}

#[derive(Debug)]
pub struct OverclockParams {
    pub clocks: Option<(u32, u32)>,
    pub graphics_offset: Option<i32>,
    pub memory_offset: Option<i32>,
    pub power_limit: Option<u32>,
    pub dry_run: bool,
}

#[derive(Debug)]
pub enum Operation {
    Info { json: bool },
    Reset { dry_run: bool },
    Overclock(OverclockParams),
    List(ListFormat),
}

impl Operation {
    pub fn modifies_gpu(&self) -> bool {
        matches!(self, Operation::Reset { .. } | Operation::Overclock(_))
    }
}

#[derive(Debug)]
pub struct Config {
    pub devices: Devices,
    pub operation: Operation,
}

fn parse_clocks(s: &str) -> std::result::Result<(u32, u32), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err("Clock format must be 'min,max'".into());
    }

    let min = parts[0]
        .parse::<u32>()
        .map_err(|_| "Invalid minimum clock value".to_string())?;
    let max = parts[1]
        .parse::<u32>()
        .map_err(|_| "Invalid maximum clock value".to_string())?;

    if min >= max {
        return Err("Minimum clock must be less than maximum clock".into());
    }

    Ok((min, max))
}

fn is_uuid_token(s: &str) -> bool {
    let upper = s.get(..4).map(|p| p.to_ascii_uppercase());
    matches!(upper.as_deref(), Some("GPU-") | Some("MIG-"))
}

/// If `tok` begins with `name:` or `n:` (case-insensitive), return the trimmed pattern.
fn extract_name_pattern(tok: &str) -> Option<&str> {
    if tok.get(..5).is_some_and(|p| p.eq_ignore_ascii_case("name:")) {
        return Some(tok[5..].trim());
    }
    if tok.get(..2).is_some_and(|p| p.eq_ignore_ascii_case("n:")) {
        return Some(tok[2..].trim());
    }
    None
}

/// If `tok` begins with `regex:` or `r:` (case-insensitive), return the trimmed pattern.
fn extract_regex_pattern(tok: &str) -> Option<&str> {
    if tok.get(..6).is_some_and(|p| p.eq_ignore_ascii_case("regex:")) {
        return Some(tok[6..].trim());
    }
    if tok.get(..2).is_some_and(|p| p.eq_ignore_ascii_case("r:")) {
        return Some(tok[2..].trim());
    }
    None
}

fn parse_devices(s: &str) -> std::result::Result<Devices, String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err("device spec is empty".into());
    }

    let tokens: Vec<&str> = trimmed.split(',').map(str::trim).collect();

    if tokens.iter().any(|t| t.eq_ignore_ascii_case("all")) {
        if tokens.len() != 1 {
            return Err("'all' must be the sole -d value".into());
        }
        return Ok(Devices::All);
    }

    let mut refs: Vec<DeviceRef> = Vec::with_capacity(tokens.len());
    for tok in tokens {
        if tok.is_empty() {
            return Err("empty entry in -d list".into());
        }
        let r = if let Some(pattern) = extract_name_pattern(tok) {
            if pattern.is_empty() {
                return Err("empty -d name pattern".into());
            }
            DeviceRef::Name(pattern.to_string())
        } else if let Some(pattern) = extract_regex_pattern(tok) {
            if pattern.is_empty() {
                return Err("empty -d regex pattern".into());
            }
            let re = Regex::new(pattern)
                .map_err(|e| format!("invalid -d regex '{pattern}': {e}"))?;
            DeviceRef::Regex(re)
        } else if is_uuid_token(tok) {
            let canon = canonicalize_uuid(tok);
            DeviceRef::Uuid(canon)
        } else {
            tok.parse::<u32>()
                .map(DeviceRef::Index)
                .map_err(|_| format!("invalid -d value: '{tok}'"))?
        };
        if refs.contains(&r) {
            return Err(format!("duplicate -d entry: '{tok}'"));
        }
        refs.push(r);
    }
    Ok(Devices::List(refs))
}

fn canonicalize_uuid(s: &str) -> String {
    // Uppercase the prefix (GPU-/MIG-) to match NVML's output. Body kept verbatim;
    // NVML's lookup is case-insensitive on the hex portion in practice.
    let mut out = String::with_capacity(s.len());
    for (i, c) in s.chars().enumerate() {
        if i < 4 {
            out.push(c.to_ascii_uppercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn devices_arg() -> Arg {
    Arg::new("device")
        .short('d')
        .long("device")
        .value_name("SPEC")
        .help("GPU spec: index, UUID, 'name:<pat>' / 'n:<pat>', 'regex:<re>' / 'r:<re>', 'all', or a comma-separated mix")
        .default_value("0")
        .value_parser(parse_devices)
}

fn dry_run_arg() -> Arg {
    Arg::new("dry-run")
        .long("dry-run")
        .help("Preview")
        .action(ArgAction::SetTrue)
}

fn json_arg() -> Arg {
    Arg::new("json")
        .long("json")
        .help("Emit JSON")
        .action(ArgAction::SetTrue)
}

impl Config {
    pub fn from_args() -> Result<Self, clap::Error> {
        let matches = Command::new(app::NAME)
            .version(app::VERSION)
            .author(app::AUTHOR)
            .about(app::DESCRIPTION)
            .subcommand_required(false)
            .subcommand(
                Command::new("reset")
                    .about("Reset GPU to defaults")
                    .arg(devices_arg())
                    .arg(dry_run_arg()),
            )
            .subcommand(
                Command::new("info")
                    .about("Show GPU information")
                    .arg(devices_arg())
                    .arg(json_arg()),
            )
            .subcommand(
                Command::new("list")
                    .about("List visible GPUs")
                    .arg(
                        Arg::new("uuid")
                            .long("uuid")
                            .help("Print UUIDs only, one per line")
                            .action(ArgAction::SetTrue)
                            .conflicts_with("json"),
                    )
                    .arg(json_arg()),
            )
            .arg(
                Arg::new("clocks")
                    .short('c')
                    .long("clocks")
                    .value_name("MIN,MAX")
                    .help("GPU clocks MHz")
                    .value_parser(parse_clocks),
            )
            .arg(
                Arg::new("offset")
                    .short('o')
                    .long("offset")
                    .value_name("GRAPHICS_OFFSET")
                    .help("GPU offset MHz")
                    .allow_hyphen_values(true)
                    .value_parser(clap::value_parser!(i32)),
            )
            .arg(
                Arg::new("memory-offset")
                    .short('m')
                    .long("memory-offset")
                    .value_name("MEMORY_OFFSET")
                    .help("Mem offset MHz")
                    .allow_hyphen_values(true)
                    .value_parser(clap::value_parser!(i32)),
            )
            .arg(
                Arg::new("power")
                    .short('p')
                    .long("power")
                    .value_name("PERCENT")
                    .help("Power limit %")
                    .value_parser(clap::value_parser!(u32)),
            )
            .arg(devices_arg())
            .arg(dry_run_arg())
            .get_matches();

        match matches.subcommand() {
            Some(("reset", sub_matches)) => Ok(Config {
                devices: sub_matches.get_one::<Devices>("device").cloned().unwrap_or_default(),
                operation: Operation::Reset {
                    dry_run: sub_matches.get_flag("dry-run"),
                },
            }),
            Some(("info", sub_matches)) => Ok(Config {
                devices: sub_matches.get_one::<Devices>("device").cloned().unwrap_or_default(),
                operation: Operation::Info {
                    json: sub_matches.get_flag("json"),
                },
            }),
            Some(("list", sub_matches)) => {
                let format = if sub_matches.get_flag("uuid") {
                    ListFormat::UuidOnly
                } else if sub_matches.get_flag("json") {
                    ListFormat::Json
                } else {
                    ListFormat::Human
                };
                Ok(Config {
                    devices: Devices::default(),
                    operation: Operation::List(format),
                })
            }
            _ => {
                let clocks = matches.get_one::<(u32, u32)>("clocks").copied();
                let graphics_offset = matches.get_one::<i32>("offset").copied();
                let memory_offset = matches.get_one::<i32>("memory-offset").copied();
                let power_limit = matches.get_one::<u32>("power").copied();

                if clocks.is_none()
                    && graphics_offset.is_none()
                    && memory_offset.is_none()
                    && power_limit.is_none()
                {
                    return Err(Command::new(app::NAME)
                        .error(clap::error::ErrorKind::MissingRequiredArgument, "No operation specified. Use a subcommand (info, list, reset) or provide overclock options (-c, -o, -m, -p)."));
                }

                Ok(Config {
                    devices: matches.get_one::<Devices>("device").cloned().unwrap_or_default(),
                    operation: Operation::Overclock(OverclockParams {
                        clocks,
                        graphics_offset,
                        memory_offset,
                        power_limit,
                        dry_run: matches.get_flag("dry-run"),
                    }),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(refs: &[DeviceRef]) -> Devices {
        Devices::List(refs.to_vec())
    }

    #[test]
    fn parses_single_index() {
        match parse_devices("0").unwrap() {
            Devices::List(v) => assert_eq!(v, vec![DeviceRef::Index(0)]),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parses_multiple_indices() {
        match parse_devices("0,2,3").unwrap() {
            Devices::List(v) => assert_eq!(
                v,
                vec![DeviceRef::Index(0), DeviceRef::Index(2), DeviceRef::Index(3)]
            ),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parses_all_sugar() {
        assert!(matches!(parse_devices("all").unwrap(), Devices::All));
        assert!(matches!(parse_devices("ALL").unwrap(), Devices::All));
        assert!(matches!(parse_devices(" all ").unwrap(), Devices::All));
    }

    #[test]
    fn rejects_all_mixed_with_others() {
        assert!(parse_devices("0,all").is_err());
        assert!(parse_devices("all,1").is_err());
    }

    #[test]
    fn parses_gpu_uuid() {
        let s = "GPU-86c2a1f9-0489-21c0-3fd3-f08768de469d";
        match parse_devices(s).unwrap() {
            Devices::List(v) => {
                assert_eq!(v.len(), 1);
                assert!(matches!(v[0], DeviceRef::Uuid(_)));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parses_mig_uuid() {
        let s = "MIG-abcdef01-2345-6789-abcd-ef0123456789";
        match parse_devices(s).unwrap() {
            Devices::List(v) => {
                assert!(matches!(v[0], DeviceRef::Uuid(ref u) if u.starts_with("MIG-")));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn uuid_prefix_is_case_insensitive() {
        assert!(matches!(
            parse_devices("gpu-abc").unwrap(),
            Devices::List(_)
        ));
        assert!(matches!(
            parse_devices("mig-abc").unwrap(),
            Devices::List(_)
        ));
    }

    #[test]
    fn canonicalizes_uuid_prefix_to_uppercase() {
        match parse_devices("gpu-abc").unwrap() {
            Devices::List(v) => match &v[0] {
                DeviceRef::Uuid(u) => assert!(u.starts_with("GPU-"), "got {u}"),
                _ => panic!("expected Uuid"),
            },
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parses_mixed_index_and_uuid() {
        let s = "0,GPU-abc,2";
        match parse_devices(s).unwrap() {
            Devices::List(v) => {
                assert_eq!(v.len(), 3);
                assert_eq!(v[0], DeviceRef::Index(0));
                assert!(matches!(v[1], DeviceRef::Uuid(_)));
                assert_eq!(v[2], DeviceRef::Index(2));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn rejects_duplicate_index() {
        assert!(parse_devices("0,0").is_err());
        assert!(parse_devices("1,0,1").is_err());
    }

    #[test]
    fn rejects_duplicate_uuid() {
        let s = "GPU-abc,GPU-abc";
        assert!(parse_devices(s).is_err());
    }

    #[test]
    fn duplicate_uuid_check_is_case_insensitive_on_prefix() {
        // Both canonicalize to "GPU-abc", should collide.
        let s = "GPU-abc,gpu-abc";
        assert!(parse_devices(s).is_err());
    }

    #[test]
    fn rejects_invalid_tokens() {
        assert!(parse_devices("abc").is_err());
        assert!(parse_devices("0,abc").is_err());
        assert!(parse_devices("1.5").is_err());
        assert!(parse_devices("-1").is_err());
    }

    #[test]
    fn rejects_empty_entries() {
        assert!(parse_devices("").is_err());
        assert!(parse_devices(",").is_err());
        assert!(parse_devices("0,").is_err());
        assert!(parse_devices(",0").is_err());
    }

    #[test]
    fn trims_whitespace_around_tokens() {
        match parse_devices(" 0 , 1 ").unwrap() {
            Devices::List(v) => assert_eq!(v, vec![DeviceRef::Index(0), DeviceRef::Index(1)]),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn default_devices_is_index_zero() {
        match Devices::default() {
            Devices::List(v) => assert_eq!(v, vec![DeviceRef::Index(0)]),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn is_uuid_token_recognizes_prefixes() {
        assert!(is_uuid_token("GPU-abc"));
        assert!(is_uuid_token("gpu-abc"));
        assert!(is_uuid_token("Gpu-abc"));
        assert!(is_uuid_token("MIG-abc"));
        assert!(is_uuid_token("mig-abc"));
        assert!(!is_uuid_token("0"));
        assert!(!is_uuid_token("GPU"));
        assert!(!is_uuid_token("GPUS-abc"));
        assert!(!is_uuid_token("abc"));
    }

    // Sanity: helper returns the same shape as List(...) for inline construction in
    // future tests. Kept to suppress dead-code on the `list` helper.
    #[test]
    fn list_helper_constructs_list_variant() {
        assert!(matches!(
            list(&[DeviceRef::Index(0)]),
            Devices::List(_)
        ));
    }

    #[test]
    fn parses_name_prefix() {
        match parse_devices("name:5090").unwrap() {
            Devices::List(v) => assert_eq!(v, vec![DeviceRef::Name("5090".into())]),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parses_n_shorthand() {
        match parse_devices("n:5090").unwrap() {
            Devices::List(v) => assert_eq!(v, vec![DeviceRef::Name("5090".into())]),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn name_prefix_is_case_insensitive() {
        for spec in ["NAME:5090", "Name:5090", "naMe:5090", "N:5090", "n:5090"] {
            match parse_devices(spec).unwrap() {
                Devices::List(v) => assert_eq!(
                    v,
                    vec![DeviceRef::Name("5090".into())],
                    "spec {spec}"
                ),
                _ => panic!("expected List for {spec}"),
            }
        }
    }

    #[test]
    fn parses_name_with_space() {
        match parse_devices("name:5060 Ti").unwrap() {
            Devices::List(v) => assert_eq!(v, vec![DeviceRef::Name("5060 Ti".into())]),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parses_n_shorthand_with_space() {
        match parse_devices("n:5060 Ti").unwrap() {
            Devices::List(v) => assert_eq!(v, vec![DeviceRef::Name("5060 Ti".into())]),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn trims_whitespace_around_name_pattern() {
        match parse_devices("name:  5090  ").unwrap() {
            Devices::List(v) => assert_eq!(v, vec![DeviceRef::Name("5090".into())]),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn rejects_empty_name_pattern() {
        assert!(parse_devices("name:").is_err());
        assert!(parse_devices("n:").is_err());
        assert!(parse_devices("name:   ").is_err());
    }

    #[test]
    fn parses_mixed_index_uuid_and_name() {
        match parse_devices("0,name:5090,GPU-abc").unwrap() {
            Devices::List(v) => {
                assert_eq!(v.len(), 3);
                assert_eq!(v[0], DeviceRef::Index(0));
                assert_eq!(v[1], DeviceRef::Name("5090".into()));
                assert!(matches!(v[2], DeviceRef::Uuid(_)));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn rejects_duplicate_name_pattern_case_insensitive() {
        assert!(parse_devices("name:5090,name:5090").is_err());
        assert!(parse_devices("n:5090,N:5090").is_err());
        assert!(parse_devices("name:5090,n:5090").is_err());
        assert!(parse_devices("name:5060 Ti,n:5060 ti").is_err());
    }

    #[test]
    fn name_and_uuid_with_same_text_are_not_dupes() {
        // A name pattern and a UUID never collide since their variants differ.
        match parse_devices("name:GPU-abc,GPU-abc").unwrap() {
            Devices::List(v) => assert_eq!(v.len(), 2),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn name_token_takes_priority_over_uuid_dispatch() {
        // "name:GPU-..." should be a Name selector, not a UUID, even though the body
        // starts with the UUID prefix.
        match parse_devices("name:GPU-5090").unwrap() {
            Devices::List(v) => {
                assert_eq!(v.len(), 1);
                assert!(matches!(v[0], DeviceRef::Name(ref p) if p == "GPU-5090"));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn bare_name_or_n_without_colon_falls_through_to_invalid() {
        assert!(parse_devices("name").is_err());
        assert!(parse_devices("n").is_err());
    }

    #[test]
    fn extract_name_pattern_recognizes_prefixes() {
        assert_eq!(extract_name_pattern("name:5090"), Some("5090"));
        assert_eq!(extract_name_pattern("NAME:5090"), Some("5090"));
        assert_eq!(extract_name_pattern("n:5090"), Some("5090"));
        assert_eq!(extract_name_pattern("N:5090"), Some("5090"));
        assert_eq!(extract_name_pattern("name: 5060 Ti "), Some("5060 Ti"));
        assert_eq!(extract_name_pattern("name:"), Some(""));
        assert_eq!(extract_name_pattern("n:"), Some(""));
        assert_eq!(extract_name_pattern("GPU-abc"), None);
        assert_eq!(extract_name_pattern("0"), None);
        assert_eq!(extract_name_pattern("name"), None);
        assert_eq!(extract_name_pattern("n"), None);
    }

    #[test]
    fn parses_regex_prefix() {
        match parse_devices("regex:50[89]0").unwrap() {
            Devices::List(v) => {
                assert_eq!(v.len(), 1);
                assert!(matches!(v[0], DeviceRef::Regex(ref re) if re.as_str() == "50[89]0"));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parses_r_shorthand() {
        match parse_devices("r:5090").unwrap() {
            Devices::List(v) => {
                assert!(matches!(v[0], DeviceRef::Regex(ref re) if re.as_str() == "5090"));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn regex_prefix_is_case_insensitive() {
        for spec in ["REGEX:5090", "Regex:5090", "reGEx:5090", "R:5090", "r:5090"] {
            match parse_devices(spec).unwrap() {
                Devices::List(v) => assert!(
                    matches!(v[0], DeviceRef::Regex(ref re) if re.as_str() == "5090"),
                    "spec {spec}"
                ),
                _ => panic!("expected List for {spec}"),
            }
        }
    }

    #[test]
    fn parses_regex_with_space() {
        match parse_devices("regex:RTX 50[89]0").unwrap() {
            Devices::List(v) => {
                assert!(matches!(v[0], DeviceRef::Regex(ref re) if re.as_str() == "RTX 50[89]0"));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn trims_whitespace_around_regex_pattern() {
        match parse_devices("regex:  5090  ").unwrap() {
            Devices::List(v) => {
                assert!(matches!(v[0], DeviceRef::Regex(ref re) if re.as_str() == "5090"));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn rejects_empty_regex_pattern() {
        assert!(parse_devices("regex:").is_err());
        assert!(parse_devices("r:").is_err());
        assert!(parse_devices("regex:   ").is_err());
    }

    #[test]
    fn rejects_invalid_regex() {
        assert!(parse_devices("r:5[09").is_err());
        assert!(parse_devices("regex:(unclosed").is_err());
    }

    #[test]
    fn rejects_duplicate_regex_pattern() {
        assert!(parse_devices("r:5090,r:5090").is_err());
        assert!(parse_devices("regex:5090,r:5090").is_err());
    }

    #[test]
    fn regex_dup_check_is_case_sensitive() {
        // Unlike name patterns, regex dedup is an exact string compare: differing
        // case is two distinct selectors, not a duplicate.
        match parse_devices("r:RTX,r:rtx").unwrap() {
            Devices::List(v) => assert_eq!(v.len(), 2),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn regex_token_takes_priority_over_uuid_dispatch() {
        match parse_devices("r:GPU-5090").unwrap() {
            Devices::List(v) => {
                assert_eq!(v.len(), 1);
                assert!(matches!(v[0], DeviceRef::Regex(ref re) if re.as_str() == "GPU-5090"));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parses_mixed_index_uuid_name_and_regex() {
        match parse_devices("0,name:5090,r:50[89]0,GPU-abc").unwrap() {
            Devices::List(v) => {
                assert_eq!(v.len(), 4);
                assert_eq!(v[0], DeviceRef::Index(0));
                assert_eq!(v[1], DeviceRef::Name("5090".into()));
                assert!(matches!(v[2], DeviceRef::Regex(ref re) if re.as_str() == "50[89]0"));
                assert!(matches!(v[3], DeviceRef::Uuid(_)));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn bare_regex_or_r_without_colon_falls_through_to_invalid() {
        assert!(parse_devices("regex").is_err());
        assert!(parse_devices("r").is_err());
    }

    #[test]
    fn extract_regex_pattern_recognizes_prefixes() {
        assert_eq!(extract_regex_pattern("regex:5090"), Some("5090"));
        assert_eq!(extract_regex_pattern("REGEX:5090"), Some("5090"));
        assert_eq!(extract_regex_pattern("r:5090"), Some("5090"));
        assert_eq!(extract_regex_pattern("R:5090"), Some("5090"));
        assert_eq!(extract_regex_pattern("regex: RTX 50[89]0 "), Some("RTX 50[89]0"));
        assert_eq!(extract_regex_pattern("regex:"), Some(""));
        assert_eq!(extract_regex_pattern("r:"), Some(""));
        assert_eq!(extract_regex_pattern("GPU-abc"), None);
        assert_eq!(extract_regex_pattern("0"), None);
        assert_eq!(extract_regex_pattern("regex"), None);
        assert_eq!(extract_regex_pattern("r"), None);
        assert_eq!(extract_regex_pattern("name:5090"), None);
    }
}
