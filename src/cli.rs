//! Command-line interface parsing and configuration

use crate::constants::{app, cli};
use clap::{Arg, Command};

fn device_arg() -> Arg {
    Arg::new("device")
        .short('d')
        .long("device")
        .value_name("INDEX")
        .help("GPU index")
        .default_value("0")
        .value_parser(clap::value_parser!(u32))
}

fn dry_run_arg() -> Arg {
    Arg::new("dry-run")
        .long("dry-run")
        .help("Preview")
        .action(clap::ArgAction::SetTrue)
}

/// Configuration structure for command-line arguments
#[derive(Debug)]
pub struct Config {
    /// GPU locked clocks (min, max) in MHz
    pub clocks: Option<(u32, u32)>,
    /// Graphics clock offset in MHz
    pub graphics_offset: Option<i32>,
    /// Memory VF offset in MHz
    pub memory_offset: Option<i32>,
    /// Power limit percentage (e.g., 104 for 104%)
    pub power_limit: Option<u32>,
    /// Target GPU device index (default: 0)
    pub device: u32,
    /// Dry run mode (show what would be done)
    pub dry_run: bool,
    /// Reset to defaults
    pub reset: bool,
    /// Show detailed GPU information
    pub info: bool,
}

fn parse_clocks(s: &str) -> std::result::Result<(u32, u32), &'static str> {
    let parts: Vec<&str> = s.split(cli::CLOCK_SEPARATOR).collect();
    if parts.len() != cli::CLOCK_PARTS_COUNT {
        return Err("Clock format must be 'min,max'");
    }

    let min = parts[0]
        .parse::<u32>()
        .map_err(|_| "Invalid minimum clock value")?;
    let max = parts[1]
        .parse::<u32>()
        .map_err(|_| "Invalid maximum clock value")?;

    if min >= max {
        return Err("Minimum clock must be less than maximum clock");
    }

    Ok((min, max))
}

impl Config {
    pub fn from_args() -> Self {
        let matches = Command::new(app::NAME)
            .version(app::VERSION)
            .author(app::AUTHOR)
            .about(app::DESCRIPTION)
            .subcommand_required(false)
            .subcommand(
                Command::new("reset")
                    .about("Reset GPU to defaults")
                    .arg(device_arg())
                    .arg(dry_run_arg()),
            )
            .subcommand(
                Command::new("info")
                    .about("Show GPU information")
                    .arg(device_arg()),
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
            .arg(device_arg())
            .arg(dry_run_arg())
            .get_matches();

        // Handle subcommands
        match matches.subcommand() {
            Some(("reset", sub_matches)) => Config {
                clocks: None,
                graphics_offset: None,
                memory_offset: None,
                power_limit: None,
                device: *sub_matches.get_one::<u32>("device").unwrap(),
                dry_run: sub_matches.get_flag("dry-run"),
                reset: true,
                info: false,
            },
            Some(("info", sub_matches)) => Config {
                clocks: None,
                graphics_offset: None,
                memory_offset: None,
                power_limit: None,
                device: *sub_matches.get_one::<u32>("device").unwrap(),
                dry_run: false,
                reset: false,
                info: true,
            },
            _ => Config {
                clocks: matches.get_one::<(u32, u32)>("clocks").copied(),
                graphics_offset: matches.get_one::<i32>("offset").copied(),
                memory_offset: matches.get_one::<i32>("memory-offset").copied(),
                power_limit: matches.get_one::<u32>("power").copied(),
                device: *matches.get_one::<u32>("device").unwrap(),
                dry_run: matches.get_flag("dry-run"),
                reset: false,
                info: false,
            },
        }
    }

    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Require at least one operation
        if self.clocks.is_none()
            && self.graphics_offset.is_none()
            && self.memory_offset.is_none()
            && self.power_limit.is_none()
            && !self.reset
            && !self.info
        {
            return Err("No operation specified.".into());
        }

        Ok(())
    }
}
