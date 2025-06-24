use std::{
    env,
    fs::{self, File},
    io::{prelude::*, BufReader},
    path::Path,
};

use anyhow::Result;
use glob::{glob, Pattern};

use crate::{
    debug,
    vcs::{config::Config, find_repo, index::Object},
    NotificationLevel,
};

fn visit_dirs(dir: &Path, cb: &dyn Fn(&Path) -> Result<()>) -> Result<()> {
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

pub(crate) fn add(paths: &[String], cfg: &Config, lvl: NotificationLevel) -> Result<()> {
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
