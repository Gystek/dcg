use std::{
    env,
    fs::{self, File},
    io::Read,
    process::Command,
};

use anyhow::Result;
use mktemp::Temp;

use crate::{
    backend::linguist::LinguistState,
    error, info,
    vcs::{
        commit::{get_branch, Change, ChangeContent, CommitObject},
        config::Config,
        find_repo,
        index::{compute_status, get_indexed_files},
        DcgError,
    },
    NotificationLevel,
};

pub(crate) fn commit(
    message: &Option<String>,
    state: LinguistState,
    cfg: &Config,
    lvl: NotificationLevel,
) -> Result<()> {
    let wd = env::current_dir().map(fs::canonicalize)??.into_boxed_path();
    let dd = find_repo(&wd)?.to_path_buf();

    if message.is_none() && (cfg.commit.as_ref().and_then(|c| c.editor.as_ref())).is_none() {
        return Err(DcgError::NoEditor.into());
    }

    if (cfg
        .user
        .as_ref()
        .and_then(|u| u.name.as_ref().and(u.email.as_ref())))
    .is_none()
    {
        return Err(DcgError::NoAuthor.into());
    }

    let files = compute_status()?.into_iter().map(|x| x.0);
    let mut added = 0;
    let mut modified = 0;
    let mut deleted = 0;

    let mut changes = Vec::new();
    for file in files {
        let change = Change::from(state, &file, &dd)?;

        if let Some(ch) = change {
            match ch.content {
                ChangeContent::Addition(_) => added += 1,
                ChangeContent::Modification(_, _, _) => modified += 1,
                ChangeContent::Deletion => deleted += 1,
            }

            changes.push(ch);
        }
    }

    if changes.is_empty() {
        return Err(DcgError::NoChanges.into());
    }

    let message = if let Some(msg) = message {
        msg.to_string()
    } else {
        let mut s = String::new();
        let tmp = Temp::new_file()?;

        let tmppb = tmp.to_path_buf();
        let fname = tmppb.as_os_str().to_str().unwrap_or("");

        let cmd = cfg.commit.as_ref().unwrap().editor.as_ref().unwrap();

        let st = Command::new(cmd).arg(fname).status()?;

        if !st.success() {
            return Err(DcgError::FailedToWriteMessage.into());
        }

        File::open(tmp)?.read_to_string(&mut s)?;

        s
    };

    if message.is_empty() {
        info!(lvl, "empty commit message. aborting.");

        return Ok(());
    }

    let commit = CommitObject::new(cfg.user.clone().unwrap(), message.clone(), changes);

    let h = commit.write(state)?;

    info!(
        lvl,
        "[{} {}] {}",
        get_branch(dd)?,
        hex::encode(&h[..4]),
        message.lines().next().unwrap()
    );
    info!(
        lvl,
        "  {} files created, {} files deleted and {} files modified", added, deleted, modified
    );

    Ok(())
}
