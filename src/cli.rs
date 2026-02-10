//! Command-line interface parsing and configuration

use crate::constants::app;
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

#[derive(Debug)]
pub enum Operation {
    Info,
    Reset { dry_run: bool },
    Overclock {
        clocks: Option<(u32, u32)>,
        graphics_offset: Option<i32>,
        memory_offset: Option<i32>,
        power_limit: Option<u32>,
        dry_run: bool,
    },
}

impl Operation {
    pub fn modifies_gpu(&self) -> bool {
        matches!(self, Operation::Reset { .. } | Operation::Overclock { .. })
    }
}

#[derive(Debug)]
pub struct Config {
    pub device: u32,
    pub operation: Operation,
}

fn parse_clocks(s: &str) -> std::result::Result<(u32, u32), &'static str> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
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
    pub fn from_args() -> Result<Self, clap::Error> {
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

        match matches.subcommand() {
            Some(("reset", sub_matches)) => Ok(Config {
                device: *sub_matches.get_one::<u32>("device").unwrap(),
                operation: Operation::Reset {
                    dry_run: sub_matches.get_flag("dry-run"),
                },
            }),
            Some(("info", sub_matches)) => Ok(Config {
                device: *sub_matches.get_one::<u32>("device").unwrap(),
                operation: Operation::Info,
            }),
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
                        .error(clap::error::ErrorKind::MissingRequiredArgument, "No operation specified. Use a subcommand (info, reset) or provide overclock options (-c, -o, -m, -p)."));
                }

                Ok(Config {
                    device: *matches.get_one::<u32>("device").unwrap(),
                    operation: Operation::Overclock {
                        clocks,
                        graphics_offset,
                        memory_offset,
                        power_limit,
                        dry_run: matches.get_flag("dry-run"),
                    },
                })
            }
        }
    }
}
