use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::{
    backend::linguist::LinguistState,
    combine_paths,
    vcs::diffs::{do_diff, get_diff_type},
};

use super::{config::User, diffs::DiffType, DCG_DIR, INDEX_DIR, LAST_DIR};

#[derive(Debug, Clone)]
pub(crate) enum ChangeContent {
    Addition,
    Deletion,
    Modification(DiffType, Vec<u8>),
}

#[derive(Debug, Clone)]
pub(crate) struct Change {
    pub(crate) content: ChangeContent,
    pub(crate) left: Vec<u8>,
    pub(crate) path: PathBuf,
}

impl Change {
    pub(crate) fn from(state: LinguistState, f: &Path, dd: &Path) -> Result<Option<Self>> {
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
            Ok(Some(Self {
                path: f.to_path_buf(),
                left: lb,
                content: match (lh, ih) {
                    (None, _) => ChangeContent::Addition,
                    (_, None) => ChangeContent::Deletion,
                    (Some(lh), Some(ih)) => {
                        let dt = get_diff_type(state, &laf, &inf)?;

                        let lhf = laf.with_file_name(hex::encode(lh));
                        let ihf = inf.with_file_name(hex::encode(ih));

                        let d = do_diff(dt, &lhf, &ihf, true)?;

                        ChangeContent::Modification(dt, d)
                    }
                },
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CommitObject {
    author: User,
    message: String,
    changes: Vec<Change>,
}
