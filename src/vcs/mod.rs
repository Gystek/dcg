pub(crate) mod config;
pub(crate) mod index;

pub(crate) const DCG_DIR: &str = ".dcg/";
pub(crate) const INDEX_DIR: &str = "index/";
pub(crate) const TREE_DIR: &str = "tree/";
pub(crate) const LAST_DIR: &str = "last/";
pub(crate) const BASE_DIR: &str = "base/";

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
