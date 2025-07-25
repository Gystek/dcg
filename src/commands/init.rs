use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::Result;

use crate::{
    combine_paths, debug, info,
    vcs::{
        config::Config, BASE_DIR, BLOBS_DIR, BRANCHES_DIR, DCG_DIR, INDEX_DIR, LAST_DIR, REFS_DIR,
        TAGS_DIR, TREE_DIR,
    },
    NotificationLevel,
};

const REPO_DIRECTORIES: [&str; 8] = [
    INDEX_DIR,
    TREE_DIR,
    LAST_DIR,
    BASE_DIR,
    BLOBS_DIR,
    REFS_DIR,
    BRANCHES_DIR,
    TAGS_DIR,
];

pub(crate) fn init(
    initial_branch: &Option<String>,
    directory: &Option<String>,
    cfg: &Config,
    lvl: NotificationLevel,
) -> Result<()> {
    let initial_branch = initial_branch
        .as_ref()
        .or(cfg.init.as_ref().and_then(|x| x.default_branch.as_ref()))
        .map_or("master", String::as_str);
    let p_directory = combine_paths!(
        directory
            .as_ref()
            .map_or(env::current_dir()?, |x| Path::new(&x).to_path_buf()),
        DCG_DIR
    );

    let reinit = if !p_directory.exists() {
        false
    } else {
        debug!(lvl, "deleting existing .dcg directory");
        fs::remove_dir_all(&p_directory)?;
        true
    };

    fs::create_dir_all(&p_directory)?;
    debug!(lvl, "creating .dcg directory in {:?}", &p_directory);

    for dir in REPO_DIRECTORIES {
        let pd = combine_paths!(&p_directory, dir);

        debug!(lvl, "created directory {:?}", pd);

        fs::create_dir_all(pd)?;
    }

    File::create(combine_paths!(&p_directory, REFS_DIR, "HEAD"))?
        .write_all(initial_branch.as_bytes())?;

    debug!(
        lvl,
        "created '.dcg/{}HEAD' pointing to '{}'", REFS_DIR, initial_branch
    );

    File::create(combine_paths!(&p_directory, BRANCHES_DIR, initial_branch))?;

    debug!(lvl, "created '.dcg/{}{}'", BRANCHES_DIR, initial_branch);

    info!(
        lvl,
        "{} dcg repository in '{}'",
        if reinit {
            "Reinitialized"
        } else {
            "Initialized new"
        },
        directory.as_ref().unwrap_or(&".".to_string())
    );

    Ok(())
}
