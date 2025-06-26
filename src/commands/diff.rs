use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use anyhow::Result;
use flate2::write::GzDecoder;
use glob::glob;

use crate::{
    backend::{linear, linguist::LinguistState},
    combine_paths,
    commands::visit_dirs,
    debug,
    vcs::{
        config::Config,
        diffs::{deserialise_everything, do_diff, get_diff_type, DiffType},
        find_repo, DCG_DIR, INDEX_DIR, LAST_DIR,
    },
    NotificationLevel,
};

fn diff_file(
    state: LinguistState,
    f: &Path,
    wd: &Path,
    dd: &Path,
    lvl: NotificationLevel,
) -> Result<()> {
    // basically what `compute_status` does but for only one file
    let last = combine_paths!(dd, DCG_DIR, LAST_DIR);
    let index = combine_paths!(dd, DCG_DIR, INDEX_DIR);

    let laf = last.join(f).into_boxed_path();
    let inf = index.join(f).into_boxed_path();

    let mut lb = Vec::new();

    let lh: Option<[u8; 32]> = if laf.exists() {
        let mut h = String::new();

        File::open(&laf)?.read_to_string(&mut h)?;

        let hp = laf.with_file_name(h.trim());
        File::open(&hp)?.read_to_end(&mut lb)?;

        Some(hex::decode(&h)?.try_into().unwrap())
    } else {
        None
    };

    let ih: Option<[u8; 32]> = if inf.exists() {
        let mut h = String::new();

        File::open(&inf)?.read_to_string(&mut h)?;
        Some(hex::decode(&h)?.try_into().unwrap())
    } else {
        None
    };

    if lh != ih {
        print!("{}:", f.display());
        match (lh, ih) {
            (None, _) => println!(" file was created"),
            (_, None) => println!(" file was deleted"),
            (Some(lh), Some(ih)) => {
                let dt = get_diff_type(state, &laf, &inf)?;

                let lhf = laf.with_file_name(hex::encode(lh));
                let ihf = inf.with_file_name(hex::encode(ih));

                let d = do_diff(dt, &lhf, &ihf, true)?;
                let mut writer = Vec::new();

                let text = if !matches!(dt, DiffType::FromBinary(_) | DiffType::Binary) {
                    let mut decoder = GzDecoder::new(writer);

                    decoder.write_all(&lb)?;
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
                visit_dirs(&p, &mut |x| diff_file(state, x, &wd, dd, lvl))?;
            } else {
                diff_file(state, &p, &wd, dd, lvl)?;
            }
        }
    }

    Ok(())
}
