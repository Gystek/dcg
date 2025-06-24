use std::{cmp::Ordering, process::exit};

use anyhow::Result;
use clap::Parser;
use vcs::config::read_config;

mod backend;
mod commands;
mod vcs;

use crate::commands::Commands;

#[derive(Parser)]
#[command(version, about)]
struct Dcg {
    #[arg(short, long)]
    quiet: bool,

    #[arg(short, long)]
    silent: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NotificationLevel {
    All,
    Warnings,
    Errors,
}

impl Ord for NotificationLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::All, Self::Warnings) => Ordering::Greater,
            (Self::All, Self::Errors) => Ordering::Greater,
            (Self::Warnings, Self::All) => Ordering::Less,
            (Self::Warnings, Self::Errors) => Ordering::Greater,
            (Self::Errors, Self::All) => Ordering::Less,
            (Self::Errors, Self::Warnings) => Ordering::Less,
            _ => Ordering::Equal,
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

    let lvl = match (args.silent, args.quiet) {
        (true, _) => NotificationLevel::Errors,
        (_, true) => NotificationLevel::Warnings,
        _ => NotificationLevel::All,
    };

    let cfg = read_config()?;

    match &args.command {
        Commands::Init {
            initial_branch,
            directory,
        } => commands::init::init(initial_branch, directory, cfg, lvl),
    }
}

fn main() {
    match try_main() {
        Ok(_) => exit(0),
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}
