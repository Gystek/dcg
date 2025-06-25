use core::fmt;
use std::{error::Error, fmt::Formatter, path::Path};

use anyhow::Result;
use const_format::concatcp;

pub(crate) mod config;
pub(crate) mod diffs;
pub(crate) mod index;

pub(crate) const DCG_DIR: &str = ".dcg/";
pub(crate) const INDEX_DIR: &str = "index/";
pub(crate) const TREE_DIR: &str = "tree/";
pub(crate) const LAST_DIR: &str = "last/";
pub(crate) const BASE_DIR: &str = "base/";
pub(crate) const BLOBS_DIR: &str = "blobs/";
pub(crate) const REFS_DIR: &str = "refs/";
pub(crate) const BRANCHES_DIR: &str = concatcp!(REFS_DIR, "branches/");
pub(crate) const TAGS_DIR: &str = concatcp!(REFS_DIR, "tags/");

#[macro_export]
macro_rules! combine_paths {
    ($first:expr $(,$path:expr)+) => {{
	let mut path = std::path::Path::new(&$first).to_path_buf();

	$(
	    path = path.join($path);
	)*

	    path
    }}
}

/// Find a dcg repository in the file hierarchy
pub(crate) fn find_repo(start: &Path) -> Result<&Path> {
    if start.join(Path::new(DCG_DIR)).exists() {
        Ok(start)
    } else {
        match start.parent() {
            Some(x) => find_repo(x),
            None => Err(DcgError::NoRepository.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum DcgError {
    NoRepository,
}

impl fmt::Display for DcgError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::NoRepository => write!(f, "no dcg repository found in the file hierarchy"),
        }
    }
}

impl Error for DcgError {}
