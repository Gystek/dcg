use std::{env, fs};

use anyhow::Result;

use crate::{
    vcs::{
        commit::{fetch_head, get_branch, make_tag},
        config::Config,
        find_repo, DcgError,
    },
    NotificationLevel,
};

pub(crate) fn tag(
    tag: &str,
    commit: &Option<String>,
    _cfg: &Config,
    _lvl: NotificationLevel,
) -> Result<()> {
    let wd = env::current_dir().map(fs::canonicalize)??.into_boxed_path();
    let dd = find_repo(&wd)?.to_path_buf();

    let commit = if let Some(c) = commit {
        let c = hex::decode(c)?;

        match c.try_into() {
            Ok(h) => Ok(h),
            _ => Err(DcgError::InvalidCommit),
        }
    } else {
        let b = get_branch(&dd)?;

        match fetch_head(&dd, &b)? {
            Some(h) => Ok(h),
            None => Err(DcgError::EmptyTree),
        }
    }?;

    make_tag(dd, commit, tag)
}
