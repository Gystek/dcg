use std::{
    fs::{self, File},
    io::{self, prelude::*, Read, Write},
    path::{Path, PathBuf},
};

use flate2::{
    write::{GzDecoder, GzEncoder},
    Compression,
};
use sha2::{Digest, Sha256};

use crate::combine_paths;

use super::{DCG_DIR, INDEX_DIR};

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

        let hash = Sha256::digest(&contents).into();

        Ok(Self {
            path,
            hash,
            contents,
        })
    }

    pub(crate) fn read(wd: &'a Path, path: &'a Path) -> io::Result<Option<([u8; 32], Vec<u8>)>> {
        let index = combine_paths!(wd, Path::new(DCG_DIR), Path::new(INDEX_DIR));

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

    pub(crate) fn write(&self, wd: &'a Path) -> io::Result<usize> {
        let index = combine_paths!(wd, Path::new(DCG_DIR), Path::new(INDEX_DIR));

        let parent = self
            .path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let fname = self.path.file_name().and_then(|x| x.to_str()).unwrap_or("");

        if fname.is_empty() {
            return Ok(0);
        }

        let virtual_parent = combine_paths!(index, parent);

        if !virtual_parent.exists() {
            fs::create_dir_all(&virtual_parent)?;
        }

        let symlink = combine_paths!(&virtual_parent, Path::new(fname));
        let hash_s = hex::encode(self.hash);
        let virtual_file = combine_paths!(&virtual_parent, Path::new(&hash_s));

        File::create(symlink)?.write_all(hash_s.as_bytes())?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&self.contents)?;

        let gz_contents = encoder.finish()?;

        File::create(virtual_file)?.write_all(&gz_contents)?;

        Ok(gz_contents.len())
    }
}
