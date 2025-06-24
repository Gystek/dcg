use std::{
    env,
    fs::{self, File},
    io::{prelude::*, BufReader},
    path::Path,
};

use anyhow::Result;
use glob::{glob, Pattern};

use crate::{
    commands::visit_dirs,
    debug,
    vcs::{config::Config, find_repo, index::Object},
    NotificationLevel,
};

fn get_ignored(dd: &Path) -> std::io::Result<Vec<Pattern>> {
    let ig = dd.join(Path::new(".dcgignore"));

    if ig.exists() {
        BufReader::new(File::open(ig)?)
            .lines()
            .map(|x| x.map(|x| Pattern::new(&x).unwrap_or(Pattern::new("").unwrap())))
            .collect::<std::io::Result<Vec<Pattern>>>()
    } else {
        Ok(vec![])
    }
}

fn add_file(
    ignored: &[Pattern],
    path: &Path,
    wd: &Path,
    dd: &Path,
    lvl: NotificationLevel,
) -> Result<()> {
    for pat in ignored {
        if pat.matches_path(path) {
            debug!(lvl, "ignoring file {:?} (matches against '{}')", path, pat);
            return Ok(());
        }
    }

    debug!(lvl, "adding file {:?}", path);

    let obj = Object::construct(wd, path)?;

    obj.write(dd)?;

    Ok(())
}

pub(crate) fn add(paths: &[String], _cfg: &Config, lvl: NotificationLevel) -> Result<()> {
    let wd = env::current_dir().map(fs::canonicalize)??.into_boxed_path();
    let dd = find_repo(&wd)?;

    let ignored = get_ignored(dd)?;

    for path in paths {
        for entry in glob(path)? {
            let p = entry?.into_boxed_path();

            if p.is_dir() {
                debug!(lvl, "recursively adding directory {:?}", &p);
                visit_dirs(&p, &|x| add_file(&ignored, x, &wd, dd, lvl))?;
            } else {
                add_file(&ignored, &p, &wd, dd, lvl)?;
            }
        }
    }

    Ok(())
}
