use anyhow::Result;

use crate::{
    vcs::index::{compute_status, ObjStatus},
    NotificationLevel,
};

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
