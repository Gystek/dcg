use anyhow::Result;
use glob::glob;
use std::{env, fs, path::Path};

use crate::{
    commands::visit_dirs,
    debug,
    vcs::{config::Config, find_repo, index::Object},
    NotificationLevel,
};

fn rm_file(path: &Path, _wd: &Path, dd: &Path, lvl: NotificationLevel) -> Result<()> {
    debug!(lvl, "removing file {:?}", path);

    Object::delete(dd, path)?;

    Ok(())
}

pub(crate) fn rm(paths: &[String], _cfg: &Config, lvl: NotificationLevel) -> Result<()> {
    let wd = env::current_dir().map(fs::canonicalize)??.into_boxed_path();
    let dd = find_repo(&wd)?;

    for path in paths {
        for entry in glob(path)? {
            let p = entry?.into_boxed_path();

            if p.is_dir() {
                debug!(lvl, "recursively removing directory {:?}", &p);
                visit_dirs(&p, &mut |x| rm_file(x, &wd, dd, lvl))?;
            } else {
                rm_file(&p, &wd, dd, lvl)?;
            }
        }
    }

    Ok(())
}
