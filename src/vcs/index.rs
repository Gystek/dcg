use std::{
    collections::BTreeMap,
    env,
    ffi::OsStr,
    fmt::{self, Formatter},
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use flate2::{
    write::{GzDecoder, GzEncoder},
    Compression,
};
use sha2::{Digest, Sha256};

use crate::{combine_paths, commands::visit_dirs};

use super::{find_repo, DCG_DIR, INDEX_DIR, LAST_DIR};

#[derive(Clone, Debug)]
pub(crate) struct Object<'a> {
    path: &'a Path,
    hash: [u8; 32],
    contents: Vec<u8>,
}

impl<'a> Object<'a> {
    pub(crate) fn construct(wd: &'a Path, path: &'a Path) -> io::Result<Self> {
        let mut f = File::open(wd.join(path))?;

        let meta = f.metadata()?;
        let size = meta.len() as usize;

        let mut contents = Vec::with_capacity(size);
        f.read_to_end(&mut contents)?;

        let mut hash: [u8; 32] = Sha256::digest(&contents).into();

        let fname = path.strip_prefix(wd).unwrap_or(path);

        for (i, byte) in fname.as_os_str().as_encoded_bytes().iter().enumerate() {
            hash[i % 32] ^= byte;
        }

        Ok(Self {
            path,
            hash,
            contents,
        })
    }

    pub(crate) fn read(wd: &'a Path, path: &'a Path) -> io::Result<Option<([u8; 32], Vec<u8>)>> {
        let index = combine_paths!(wd, DCG_DIR, INDEX_DIR);

        let hash_p = combine_paths!(&index, path);
        let mut hash_s = String::new();

        if !hash_p.exists() {
            return Ok(None);
        }

        File::open(hash_p)?.read_to_string(&mut hash_s)?;

        let hash = hex::decode(hash_s).unwrap_or(vec![0; 32]);

        let mut gz_contents = Vec::new();

        File::open(combine_paths!(
            &index,
            path.parent().map(Path::to_path_buf).unwrap_or_default()
        ))?
        .read_to_end(&mut gz_contents)?;

        let contents = Vec::new();
        let mut decoder = GzDecoder::new(contents);
        decoder.write_all(&gz_contents)?;

        decoder
            .finish()
            .map(|x| Some((hash.try_into().unwrap(), x)))
    }

    pub(crate) fn delete(wd: &'a Path, path: &'a Path) -> io::Result<()> {
        let fname = get_fname(path);

        if fname.is_empty() {
            return Ok(());
        }

        let virtual_parent = get_virtual_parent(wd, path);

        if !virtual_parent.exists() {
            return Ok(());
        }

        let symlink = combine_paths!(&virtual_parent, fname);

        if !symlink.exists() {
            return Ok(());
        }

        let mut hash_s = String::new();

        File::open(&symlink)?.read_to_string(&mut hash_s)?;

        let virtual_file = combine_paths!(&virtual_parent, hash_s.trim());

        fs::remove_file(virtual_file)?;
        fs::remove_file(symlink)
    }

    pub(crate) fn write(&self, wd: &'a Path) -> io::Result<usize> {
        let fname = get_fname(self.path);

        if fname.is_empty() {
            return Ok(0);
        }

        let virtual_parent = get_virtual_parent(wd, self.path);

        if !virtual_parent.exists() {
            fs::create_dir_all(&virtual_parent)?;
        }

        let symlink = combine_paths!(&virtual_parent, fname);

        let hash_s = hex::encode(self.hash);
        let virtual_file = combine_paths!(&virtual_parent, &hash_s);

        File::create(symlink)?.write_all(hash_s.as_bytes())?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&self.contents)?;

        let gz_contents = encoder.finish()?;

        File::create(virtual_file)?.write_all(&gz_contents)?;

        Ok(gz_contents.len())
    }
}

fn get_virtual_parent(wd: &Path, path: &Path) -> PathBuf {
    let index = combine_paths!(wd, DCG_DIR, INDEX_DIR);

    let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();

    combine_paths!(index, parent)
}

pub(crate) fn get_fname(path: &Path) -> &str {
    path.file_name().and_then(|x| x.to_str()).unwrap_or("")
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum ObjStatus {
    Added,
    Deleted,
    Modified,
    Kept,
}

impl fmt::Display for ObjStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Added => write!(f, "A"),
            Self::Deleted => write!(f, "D"),
            Self::Modified => write!(f, "M"),
            Self::Kept => write!(f, "K"),
        }
    }
}

fn archive_one(p: &Path, pref: &Path, m: &mut BTreeMap<PathBuf, [u8; 32]>) -> Result<()> {
    if index_true_file(p) {
        let mut h = String::new();

        File::open(p)?.read_to_string(&mut h)?;

        let h = hex::decode(h)?.try_into().unwrap();
        m.insert(p.strip_prefix(pref)?.to_path_buf(), h);
    }

    Ok(())
}

fn index_true_file(p: &Path) -> bool {
    let fname = p.file_name().and_then(OsStr::to_str).unwrap_or("");

    /* file name is not a hash */
    hex::decode(fname).is_err()
}

pub(crate) fn get_indexed_files<P: AsRef<Path>>(dd: P) -> Result<Vec<PathBuf>> {
    let dd = dd.as_ref();
    let idx = combine_paths!(dd, DCG_DIR, INDEX_DIR);
    let mut paths = Vec::new();

    visit_dirs(&idx, &mut |p| {
        if index_true_file(p) {
            paths.push(p.strip_prefix(&idx)?.to_path_buf());
        }
        Ok(())
    })?;

    Ok(paths)
}

pub(crate) fn compute_status() -> Result<Vec<(PathBuf, ObjStatus)>> {
    let wd = env::current_dir().map(fs::canonicalize)??.into_boxed_path();
    let dd = find_repo(&wd)?;

    /* hashmap file -> hash for index/ and last/ and then it's a diff */
    let mut last = BTreeMap::new();
    let mut index = BTreeMap::new();

    /* identify files which name is not a hash and push their path clipped of `dd` onto the
     * map
     */
    let last_path = combine_paths!(dd, DCG_DIR, LAST_DIR).into_boxed_path();
    let index_path = combine_paths!(dd, DCG_DIR, INDEX_DIR).into_boxed_path();

    visit_dirs(&last_path, &mut |p| archive_one(p, &last_path, &mut last))?;
    visit_dirs(&index_path, &mut |p| {
        archive_one(p, &index_path, &mut index)
    })?;

    let mut status = Vec::new();

    for (p, h0) in index {
        if let Some(&h1) = last.get(&p) {
            last.remove(&p);
            status.push((
                p,
                if h0 == h1 {
                    ObjStatus::Kept
                } else {
                    ObjStatus::Modified
                },
            ));
        } else {
            status.push((p, ObjStatus::Added));
        }
    }

    for p in last.into_keys() {
        status.push((p, ObjStatus::Deleted));
    }

    Ok(status)
}
