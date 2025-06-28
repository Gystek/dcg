use std::{
    env,
    fs::{self, copy, create_dir_all, remove_dir_all, File},
    io::{self, BufWriter, Read, Write},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use content_inspector::ContentType;
use flate2::write::GzDecoder;
use sha2::{Digest, Sha256};

use crate::{
    backend::linguist::LinguistState,
    combine_paths,
    vcs::diffs::{do_diff, get_diff_type},
};

use super::{
    config::User, diffs::DiffType, find_repo, index::get_fname, DcgError, BASE_DIR, BLOBS_DIR,
    BRANCHES_DIR, DCG_DIR, INDEX_DIR, LAST_DIR, REFS_DIR, TREE_DIR,
};

#[derive(Debug, Clone)]
pub(crate) enum ChangeContent {
    Addition([u8; 32]),
    Deletion,
    Modification(DiffType, [u8; 32], Vec<u8>),
}

#[derive(Debug, Clone)]
pub(crate) struct Change {
    pub(crate) content: ChangeContent,
    pub(crate) file: Vec<u8>,
    pub(crate) path: PathBuf,
}

impl Change {
    pub(crate) fn from<P: AsRef<Path>>(state: LinguistState, f: P, dd: P) -> Result<Option<Self>> {
        let f = f.as_ref();
        let dd = dd.as_ref();

        // basically what `compute_status` does but for only one file
        let last = combine_paths!(dd, DCG_DIR, LAST_DIR);
        let index = combine_paths!(dd, DCG_DIR, INDEX_DIR);

        let laf = last.join(f).into_boxed_path();
        let inf = index.join(f).into_boxed_path();

        let mut lb = Vec::new();
        let mut ib = Vec::new();

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

            let hp = inf.with_file_name(h.trim());
            File::open(&hp)?.read_to_end(&mut ib)?;

            Some(hex::decode(&h)?.try_into().unwrap())
        } else {
            None
        };

        if lh != ih {
            Ok(Some(Self {
                path: f.to_path_buf(),
                file: if lh.is_some() { lb } else { ib },
                content: match (lh, ih) {
                    (None, _) => ChangeContent::Addition(ih.unwrap()),
                    (_, None) => ChangeContent::Deletion,
                    (Some(lh), Some(ih)) => {
                        let dt = get_diff_type(state, &laf, &inf)?;

                        let lhf = laf.with_file_name(hex::encode(lh));
                        let ihf = inf.with_file_name(hex::encode(ih));

                        let d = do_diff(dt, &lhf, &ihf, true)?;

                        ChangeContent::Modification(dt, ih, d)
                    }
                },
            }))
        } else {
            Ok(None)
        }
    }

    fn serialise_entry(&self) -> Vec<u8> {
        let mut base = Vec::new();

        match &self.content {
            ChangeContent::Deletion => {
                base.push(b'd');
            }
            ChangeContent::Addition(h) => {
                base.push(b'a');
                base.extend(h);
            }
            ChangeContent::Modification(dt, h, _) => {
                base.push(b'm');
                base.extend(dt.serialise());
                base.extend(h);
            }
        }

        base.extend(self.path.as_os_str().as_bytes());

        base
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CommitObject {
    author: User,
    message: String,
    changes: Vec<Change>,
}

impl CommitObject {
    pub(crate) fn new(author: User, message: String, changes: Vec<Change>) -> Self {
        Self {
            author,
            message,
            changes,
        }
    }

    fn hash(&self) -> Result<[u8; 32]> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        if let User {
            name: Some(name),
            email: Some(email),
        } = &self.author
        {
            let k = format!("{}{}{}", now, name, email);

            Ok(Sha256::digest(k.as_bytes()).into())
        } else {
            Err(DcgError::NoAuthor.into())
        }
    }

    pub(crate) fn write(&self, state: LinguistState) -> Result<[u8; 32]> {
        let wd = env::current_dir().map(fs::canonicalize)??.into_boxed_path();
        let dd = find_repo(&wd)?;

        let h = self.hash()?;

        let cf = combine_paths!(dd, DCG_DIR, TREE_DIR, hash_to_commit_path(h));

        fs::create_dir_all(&cf)?;

        let mp = combine_paths!(&cf, "message");
        File::create(&mp)?.write_all(self.message.as_bytes())?;

        /* should have been previously checked */
        let name = self.author.name.as_ref().unwrap();
        let email = self.author.email.as_ref().unwrap();

        let ap = combine_paths!(&cf, "author");
        File::create(&ap)?.write_all(format!("{} <{}>", name, email).as_bytes())?;

        let branch = get_branch(dd)?;
        let parent = fetch_head(dd, &branch)?;

        let mut dir = BufWriter::new(File::create(combine_paths!(&cf, "directory"))?);
        for change in &self.changes {
            dir.write_all(&change.serialise_entry())?;

            match &change.content {
                ChangeContent::Addition(ch) => {
                    make_base_file(&change.path, &change.file, h, *ch, dd)?
                }
                ChangeContent::Modification(dt, ch, d) => {
                    handle_modification(*dt, &cf, h, *ch, d, &change.file, &change.path, dd)?
                }
                ChangeContent::Deletion => {}
            }
        }

        /* write parent if applicable */
        if let Some(parent) = parent {
            File::create(combine_paths!(&cf, "parent"))?
                .write_all(hex::encode(parent).as_bytes())?;
        }

        /* update branch head */
        File::create(combine_paths!(&dd, DCG_DIR, BRANCHES_DIR, branch))?
            .write_all(hex::encode(h).as_bytes())?;

        let idp = combine_paths!(dd, DCG_DIR, INDEX_DIR);
        let ltp = combine_paths!(dd, DCG_DIR, LAST_DIR);

        /* move index to last */
        remove_dir_all(&ltp)?;
        copy_dir_all(&idp, ltp)?;

        /* free index */
        remove_dir_all(&idp)?;
        create_dir_all(idp)?;

        Ok(h)
    }
}

fn handle_modification<P: AsRef<Path>>(
    dt: DiffType,
    commit: P,
    commit_h: [u8; 32],
    h: [u8; 32],
    d: &[u8],
    contents: &[u8],
    from: P,
    dd: &Path,
) -> Result<()> {
    let virtual_parent = combine_paths!(
        commit.as_ref(),
        from.as_ref()
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default()
    );
    let hs = hex::encode(h);

    match dt {
        DiffType::Binary => {
            make_blob_from_bytes(contents, &hs, dd)?;
        }
        DiffType::FromBinary(_) | DiffType::Tree(_) | DiffType::Linear(_, _) => {
            if !virtual_parent.exists() {
                fs::create_dir_all(&virtual_parent)?;
            }

            let df = combine_paths!(&virtual_parent, get_fname(from.as_ref()));

            File::create(df)?.write_all(&contents)?;
        }
    }

    Ok(())
}

fn make_base_file<P: AsRef<Path>>(
    p: P,
    contents: &[u8],
    h: [u8; 32],
    commit_h: [u8; 32],
    dd: &Path,
) -> Result<()> {
    let virtual_parent = combine_paths!(
        dd,
        DCG_DIR,
        BASE_DIR,
        p.as_ref()
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default()
    );

    if !virtual_parent.exists() {
        fs::create_dir_all(&virtual_parent)?;
    }

    /* files are named <base commit hash><file name> to enable
     * mulitple addition/deletions
     */
    let symlink = combine_paths!(
        &virtual_parent,
        format!("{}-{}", get_fname(p.as_ref()), hex::encode(commit_h))
    );

    File::create(symlink)?.write_all(&h)?;

    let mut writer = Vec::new();
    let mut decoder = GzDecoder::new(writer);
    decoder.write_all(contents)?;
    writer = decoder.finish()?;

    let hs = hex::encode(h);

    /* if the file is binary, it is placed in the blobs instead of the base/ directory */
    if !matches!(content_inspector::inspect(&writer), ContentType::BINARY) {
        let virtual_file = combine_paths!(&virtual_parent, hs);

        File::create(virtual_file)?.write_all(contents)?;
    } else {
        let full_path = combine_paths!(dd, DCG_DIR, INDEX_DIR, &p);
        make_blob(full_path, &hs, dd)?;
    }

    Ok(())
}

fn make_blob<P: AsRef<Path>>(from: P, hs: &str, dd: &Path) -> Result<PathBuf> {
    let bf = combine_paths!(dd, DCG_DIR, BLOBS_DIR, hs);

    if !bf.exists() {
        copy(from, &bf)?;
    }

    Ok(bf)
}

fn make_blob_from_bytes(bytes: &[u8], hs: &str, dd: &Path) -> Result<PathBuf> {
    let bf = combine_paths!(dd, DCG_DIR, BLOBS_DIR, hs);

    if !bf.exists() {
        File::create(&bf)?.write_all(bytes)?;
    }

    Ok(bf)
}

fn hash_to_commit_path(h: [u8; 32]) -> String {
    let ph = h[0];
    let sh = &h[1..];

    format!("{:02x}/{}/", ph, hex::encode(sh))
}

pub(crate) fn get_branch<P: AsRef<Path>>(dd: P) -> Result<String> {
    let refs = combine_paths!(dd.as_ref(), DCG_DIR, REFS_DIR);

    let mut branch = String::new();

    File::open(combine_paths!(&refs, "HEAD"))?.read_to_string(&mut branch)?;

    Ok(branch)
}

fn fetch_head<P: AsRef<Path>>(dd: P, branch: &str) -> Result<Option<[u8; 32]>> {
    let branches = combine_paths!(dd.as_ref(), DCG_DIR, BRANCHES_DIR);
    let mut ch = String::new();

    File::open(combine_paths!(&branches, branch.trim()))?.read_to_string(&mut ch)?;

    let ch = ch.trim();

    if ch.is_empty() {
        Ok(None)
    } else {
        Ok(hex::decode(ch).map(|x| x.try_into().unwrap()).ok())
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
