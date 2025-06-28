use std::{
    env,
    fs::{self},
    io::Write,
    path::Path,
};

use anyhow::Result;
use flate2::write::GzDecoder;
use glob::glob;

use crate::{
    backend::{linear, linguist::LinguistState},
    commands::visit_dirs,
    debug,
    vcs::{
        commit::{Change, ChangeContent},
        config::Config,
        diffs::{deserialise_everything, DiffType},
        find_repo,
    },
    NotificationLevel,
};

fn diff_file(state: LinguistState, f: &Path, dd: &Path) -> Result<()> {
    // basically what `compute_status` does but for only one file
    let ch = Change::from(state, f, dd)?;

    if let Some(ch) = ch {
        print!("{}:", ch.path.display());

        match ch.content {
            ChangeContent::Addition(_) => println!(" file was created"),
            ChangeContent::Deletion => println!(" file was deleted"),
            ChangeContent::Modification(dt, _, d) => {
                let mut writer = Vec::new();

                let text = if !matches!(dt, DiffType::FromBinary(_) | DiffType::Binary) {
                    let mut decoder = GzDecoder::new(writer);

                    decoder.write_all(&ch.file)?;
                    writer = decoder.finish()?;

                    str::from_utf8(&writer)?
                } else {
                    ""
                };

                match dt {
                    DiffType::FromBinary(_) => {
                        println!("\n{}", String::from_utf8(d)?);
                    }
                    DiffType::Binary => println!(" file is binary"),
                    DiffType::Linear(_, _) => {
                        let mut ll = text.lines().collect::<Vec<&str>>();
                        ll.push("");

                        println!();
                        linear::pretty_print(&ll, &linear::deserialise(&d));
                    }
                    DiffType::Tree(_) => {
                        let d = deserialise_everything(&d, text)?;

                        println!();
                        println!("{:?}", d);
                    }
                }
            }
        }
    }

    Ok(())
}

pub(crate) fn diff(
    files: &[String],
    state: LinguistState,
    _cfg: &Config,
    lvl: NotificationLevel,
) -> Result<()> {
    let wd = env::current_dir().map(fs::canonicalize)??.into_boxed_path();
    let dd = find_repo(&wd)?;

    let all_glob = [String::from("*")];

    let files = if files.is_empty() { &all_glob } else { files };

    for path in files {
        for entry in glob(path)? {
            let p = entry?.into_boxed_path();

            if p.is_dir() {
                debug!(lvl, "recursively removing directory {:?}", &p);
                visit_dirs(&p, &mut |x| diff_file(state, x, dd))?;
            } else {
                diff_file(state, &p, dd)?;
            }
        }
    }

    Ok(())
}
