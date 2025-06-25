use std::{cmp::Ordering, process::exit};

use anyhow::Result;
use backend::languages::{
    compile_filenames_map, compile_heuristics_map, compile_modelines_map, compile_shebang_map,
    init_all_maps,
};
use clap::Parser;
use vcs::config::read_config;

mod backend;
mod commands;
mod vcs;

use crate::commands::Commands;

#[derive(Parser)]
#[command(version, about)]
struct Dcg {
    /// display debug messages
    #[arg(short, long)]
    debug: bool,

    /// display only warning and error messages
    #[arg(short, long)]
    quiet: bool,

    /// display only error messages
    #[arg(short, long)]
    silent: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NotificationLevel {
    Debug,
    Normal,
    Warnings,
    Errors,
}

#[macro_export]
macro_rules! info {
    ($lvl:expr, $($arg:tt)*) => {{
	if ($lvl >= NotificationLevel::Normal) {
	    println!($($arg)*);
	}
    }};
}

#[macro_export]
macro_rules! debug {
    ($lvl:expr, $($arg:tt)*) => {{
	if ($lvl >= NotificationLevel::Debug) {
	    print!("DEBUG\t");
	    println!($($arg)*);
	}
    }};
}

#[macro_export]
macro_rules! warning {
    ($lvl:expr, $($arg:tt)*) => {{
	if ($lvl >= NotificationLevel::Warnings) {
	    print!("\x1b[0;33mWARNING\x1b[0m\t");
	    println!($($arg)*);
	}
    }};
}

#[macro_export]
macro_rules! error {
    ($lvl:expr, $($arg:tt)*) => {{
	if ($lvl >= NotificationLevel::Errors) {
	    print!("\x1b[0;1mERROR\x1b[0m\t");
	    println!($($arg)*);
	}
    }};
}

impl Ord for NotificationLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else {
            match (self, other) {
                (Self::Debug, _) => Ordering::Greater,
                (_, Self::Debug) => Ordering::Less,
                (Self::Normal, Self::Warnings) => Ordering::Greater,
                (Self::Normal, Self::Errors) => Ordering::Greater,
                (Self::Warnings, Self::Normal) => Ordering::Less,
                (Self::Warnings, Self::Errors) => Ordering::Greater,
                (Self::Errors, _) => Ordering::Less,
                _ => unreachable!(),
            }
        }
    }
}

impl PartialOrd for NotificationLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn try_main() -> Result<()> {
    let args = Dcg::parse();

    let lvl = match (args.debug, args.silent, args.quiet) {
        (true, _, _) => NotificationLevel::Debug,
        (_, true, _) => NotificationLevel::Errors,
        (_, _, true) => NotificationLevel::Warnings,
        _ => NotificationLevel::Normal,
    };

    let cfg = read_config()?;

    init_all_maps();

    let filenames = compile_filenames_map();
    let shebang = compile_shebang_map();
    let modelines = compile_modelines_map();
    let heuristics = compile_heuristics_map();

    let state = (&filenames, &shebang, &modelines, &heuristics);

    match &args.command {
        Commands::Init {
            initial_branch,
            directory,
        } => commands::init::init(initial_branch, directory, &cfg, lvl),
        Commands::Add { paths } => commands::add::add(paths, &cfg, lvl),
        Commands::Rm { paths } => commands::rm::rm(paths, &cfg, lvl),
        Commands::Status => commands::status::status(lvl),
        Commands::Diff { files } => commands::diff::diff(files, state, &cfg, lvl),
    }
}

fn main() {
    match try_main() {
        Ok(_) => exit(0),
        Err(e) => {
            error!(NotificationLevel::Errors, "{}", e);
            exit(1);
        }
    }
}
