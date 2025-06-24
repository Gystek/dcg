use std::{fs, path::Path};

use anyhow::Result;
use clap::Subcommand;

pub(crate) mod add;
pub(crate) mod init;
pub(crate) mod rm;

pub(crate) fn visit_dirs(dir: &Path, cb: &dyn Fn(&Path) -> Result<()>) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path().into_boxed_path();

            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&path)?;
            }
        }
    }
    Ok(())
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// initialize a new dcg repository
    Init {
        /// the name to use for the initial branch
        /// instead of 'master' (by default) or the
        /// branch name as defined in the user
        /// configuration.
        #[arg(short = 'b', long)]
        initial_branch: Option<String>,

        directory: Option<String>,
    },
    /// add files to the dcg index
    Add {
        /// the paths to add to the index.  these can
        /// contain globs (such as '*.rs') to add
        /// multiple files.  folders are added recursively
        paths: Vec<String>,
    },
    /// remove files from the dcg index
    Rm {
        /// the paths to remove from the index.  these can
        /// contain globs (such as '*.rs') to remove
        /// multiple files.  folders are removed recursively.
        paths: Vec<String>,
    },
}
