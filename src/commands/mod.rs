use std::{fs, path::Path};

use anyhow::Result;
use clap::Subcommand;

pub(crate) mod add;
pub(crate) mod commit;
pub(crate) mod diff;
pub(crate) mod init;
pub(crate) mod log;
pub(crate) mod rm;
pub(crate) mod status;
pub(crate) mod tag;

pub(crate) fn visit_dirs<F: FnMut(&Path) -> Result<()>>(dir: &Path, cb: &mut F) -> Result<()> {
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
        #[arg(short = 'b', long = "initial-branch")]
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
    /// display the status of each file in the index
    Status,
    /// display the diff between the last commit and the index
    /// files
    Diff {
        /// the files to diff.  if empty, diff all files.  folders
        /// are diff-ed recursively.
        files: Vec<String>,
    },
    /// commit the changes contained in the index to the revision tree.
    Commit {
        /// the message to associate with the commit.
        message: Option<String>,
    },
    /// list commits for the current branch
    Log {
        /// display each commit on one line
        #[arg(long = "oneline")]
        one_line: bool,
    },
    /// create a tag referencing a commit
    Tag {
        /// the tag name
        tag: String,
        /// the commit to reference (by default the head of the current branch)
        commit: Option<String>,
    },
}
