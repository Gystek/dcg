use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::Result;

use crate::{combine_paths, vcs::config::Config, NotificationLevel};

const REPO_DIRECTORIES: [&str; 7] = [
    "index/",
    "tree/",
    "last/",
    "base/",
    "refs/",
    "refs/branches/",
    "refs/tags/",
];

pub(crate) fn init(
    initial_branch: &Option<String>,
    directory: &Option<String>,
    cfg: Config,
    lvl: NotificationLevel,
) -> Result<()> {
    let initial_branch = initial_branch
        .as_ref()
        .or(cfg.init.default_branch.as_ref())
        .unwrap();
    let p_directory = combine_paths!(
        directory
            .as_ref()
            .map_or(env::current_dir()?, |x| Path::new(&x).to_path_buf()),
        Path::new(".dcg")
    );

    let reinit = if !p_directory.exists() {
        false
    } else {
        fs::remove_dir_all(&p_directory)?;
        true
    };

    fs::create_dir_all(&p_directory)?;

    for dir in REPO_DIRECTORIES {
        let pd = combine_paths!(&p_directory, Path::new(dir));

        fs::create_dir_all(pd)?;
    }

    File::create(combine_paths!(&p_directory, "refs/HEAD"))?
        .write_all(initial_branch.as_bytes())?;

    File::create(combine_paths!(
        &p_directory,
        "refs/branches/",
        initial_branch
    ))?;

    if lvl >= NotificationLevel::All {
        println!(
            "{} dcg repository in '{}'\n",
            if reinit {
                "Reinitialized"
            } else {
                "Initialized new"
            },
            directory.as_ref().unwrap_or(&".".to_string())
        );
    }

    Ok(())
}
