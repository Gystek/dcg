use std::{env, fs};

use anyhow::Result;
use time::{format_description::parse, OffsetDateTime};

use crate::{
    backend::linguist::LinguistState,
    vcs::{
        commit::{fetch_head, get_branch, get_parent, CommitObject},
        config::Config,
        find_repo,
    },
    NotificationLevel,
};

pub(crate) fn log(
    state: LinguistState,
    cfg: &Config,
    lvl: NotificationLevel,
    one_line: bool,
) -> Result<()> {
    let wd = env::current_dir().map(fs::canonicalize)??.into_boxed_path();
    let dd = find_repo(&wd)?.to_path_buf();

    let branch = get_branch(&dd)?;
    let mut head = fetch_head(&dd, &branch)?;

    while let Some(h) = head {
        let commit = CommitObject::read(&dd, h)?;

        if one_line {
            let sh = hex::encode(&h[..4]);
            println!(
                "\x1b[0;33m{}\x1b[0m {}",
                sh,
                commit.message.lines().next().unwrap()
            );
        } else {
            let date_fmt = parse(
                "[weekday repr:short] [month repr:short] [day padding:zero] \
		 [hour]:[minute]:[second] [year] [offset_hour sign:mandatory][offset_minute]",
            )?;
            println!("\x1b[0;33m{}\x1b[0m", hex::encode(&h));
            println!(
                "Author: {} <{}>",
                commit.author.name.unwrap(),
                commit.author.email.unwrap()
            );
            println!(
                "Date:   {}\n",
                OffsetDateTime::from_unix_timestamp(commit.date as i64)?.format(&date_fmt)?
            );

            println!("\t{}\n", commit.message);
        }

        head = get_parent(&dd, h)?;
    }

    Ok(())
}
