mod index;

pub(crate) const DCG_DIR: &str = ".dcg/";
pub(crate) const INDEX_DIR: &str = "index/";
pub(crate) const TREE_DIR: &str = "tree/";
pub(crate) const LAST_DIR: &str = "last/";
pub(crate) const BASE_DIR: &str = "base/";

#[macro_export]
macro_rules! combine_paths {
    ($($path:expr),+) => {{
	let mut path = std::path::PathBuf::new();

	$(
	    path = path.join($path);
	)*

	    path
    }}
}
