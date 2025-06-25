use core::fmt;
use std::{
    collections::BTreeMap,
    env,
    ffi::OsStr,
    fmt::Formatter,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::{
    combine_paths,
    commands::visit_dirs,
    vcs::{find_repo, DCG_DIR, INDEX_DIR, LAST_DIR},
    NotificationLevel,
};

#[derive(Debug, Copy, Clone)]
pub(crate) enum ObjStatus {
    Added,
    Deleted,
    Modified,
    Kept,
}

impl fmt::Display for ObjStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Added => write!(f, "A"),
            Self::Deleted => write!(f, "D"),
            Self::Modified => write!(f, "M"),
            Self::Kept => write!(f, "K"),
        }
    }
}

fn archive_one(p: &Path, pref: &Path, m: &mut BTreeMap<PathBuf, [u8; 32]>) -> Result<()> {
    let fname = p.file_name().and_then(OsStr::to_str).unwrap_or("");

    /* file name is not a hash */
    if hex::decode(fname).is_err() {
        let mut h = String::new();

        File::open(p)?.read_to_string(&mut h)?;

        let h = hex::decode(h)?.try_into().unwrap();
        m.insert(p.strip_prefix(pref)?.to_path_buf(), h);
    }

    Ok(())
}

pub(crate) fn compute_status() -> Result<Vec<(PathBuf, ObjStatus)>> {
    let wd = env::current_dir().map(fs::canonicalize)??.into_boxed_path();
    let dd = find_repo(&wd)?;

    /* hashmap file -> hash for index/ and last/ and then it's a diff */
    let mut last = BTreeMap::new();
    let mut index = BTreeMap::new();

    /* identify files which name is not a hash and push their path clipped of `dd` onto the
     * map
     */
    let last_path = combine_paths!(dd, DCG_DIR, LAST_DIR).into_boxed_path();
    let index_path = combine_paths!(dd, DCG_DIR, INDEX_DIR).into_boxed_path();

    visit_dirs(&last_path, &mut |p| archive_one(p, &last_path, &mut last))?;
    visit_dirs(&index_path, &mut |p| {
        archive_one(p, &index_path, &mut index)
    })?;

    let mut status = Vec::new();

    for (p, h0) in index {
        if let Some(&h1) = last.get(&p) {
            last.remove(&p);
            status.push((
                p,
                if h0 == h1 {
                    ObjStatus::Kept
                } else {
                    ObjStatus::Modified
                },
            ));
        } else {
            status.push((p, ObjStatus::Added));
        }
    }

    for p in last.into_keys() {
        status.push((p, ObjStatus::Deleted));
    }

    Ok(status)
}

pub(crate) fn status(_lvl: NotificationLevel) -> Result<()> {
    let status = compute_status()?;

    for (path, s) in status
        .into_iter()
        .filter(|&(_, x)| !matches!(x, ObjStatus::Kept))
    {
        println!(
            "{}\t{}",
            s,
            path.to_str().unwrap_or("failed to display path")
        );
    }

    Ok(())
}
